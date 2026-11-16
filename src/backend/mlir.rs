//! MLIR LLVM-dialect code generation for MITA.
//!
//! Lowers a list of top-level expressions to a single `module { ... }` of
//! LLVM dialect IR. Every MITA value is `!llvm.struct<(i64, i64)>` (the
//! `mita_value { int64_t tag; int64_t payload; }` C ABI). Scalars are built
//! with `llvm.insertvalue`; heap-backed values (strings/symbols/lists) are
//! built by calling the C runtime constructors.
//!
//! Scope: integer/string/quote literals, `celi/movo/celida/movoda`,
//! comparisons, `upa/lawa/kucha`, `ka`, `dala`, `tido`, `in`, `plama`,
//! `plata`, `print`, top-level sequencing, and an automatic print of the
//! final top-level result, mirroring the interpreter's `load()`.

use std::collections::HashMap;
use std::rc::Rc;

use crate::expr::Expr;
use crate::token::TokenType;

const VALUE_TY: &str = "!llvm.struct<(i64, i64)>";

/// Runtime functions the current subset can emit calls to. Kept as a fixed
/// list of declarations so the emitted module always lower cleanly; unused
/// declarations are harmless `declare` entries in the resulting LLVM IR.
const DECLS: &[&str] = &[
    "llvm.func @mita_add(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_sub(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_mul(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_div(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_eq(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_ne(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_lt(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_gt(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_le(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_ge(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_cons(!llvm.struct<(i64, i64)>, !llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_car(!llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_cdr(!llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_truthy(!llvm.struct<(i64, i64)>) -> i64",
    "llvm.func @mita_print(!llvm.ptr, i64) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_print_value(!llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_string(!llvm.ptr, i64) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_symbol(!llvm.ptr, i64) -> !llvm.struct<(i64, i64)>",
];

/// Builds the `module { ... }` text for the given top-level expressions.
pub fn compile(exprs: &[Rc<Expr>]) -> Result<String, String> {
    let mut gen = Codegen::new();

    let mut last: Option<String> = None;
    for expr in exprs {
        last = Some(gen.gen_expr(expr)?);
    }

    // Auto-print the final top-level result, like the interpreter's `load()`.
    if let Some(result) = last {
        gen.emit_print_value(&result);
    }

    let zero = gen.fresh();
    gen.body
        .push_str(&format!("    {} = llvm.mlir.constant(0 : i32) : i32\n", zero));
    gen.body
        .push_str(&format!("    llvm.return {} : i32\n", zero));

    let mut out = String::new();
    out.push_str("module {\n");
    for decl in DECLS {
        out.push_str("  ");
        out.push_str(decl);
        out.push('\n');
    }
    for global in &gen.globals {
        out.push_str("  ");
        out.push_str(global);
        out.push('\n');
    }
    out.push_str("  llvm.func @main() -> i32 {\n");
    out.push_str(&gen.prologue);
    out.push_str(&gen.body);
    out.push_str("  }\n");
    out.push_str("}\n");

    Ok(out)
}

struct Codegen {
    next: usize,
    block_next: usize,
    globals: Vec<String>,
    seen_strings: HashMap<String, String>,
    /// Entry-block `llvm.alloca`s, emitted before the instruction stream so
    /// every slot dominates its uses across branches.
    prologue: String,
    /// The main instruction stream (entry block then any branch blocks).
    body: String,
    /// Lexical variable scopes, mapping a name to its alloca slot SSA name.
    scopes: Vec<HashMap<String, String>>,
}

impl Codegen {
    fn new() -> Self {
        Codegen {
            next: 0,
            block_next: 0,
            globals: Vec::new(),
            seen_strings: HashMap::new(),
            prologue: String::new(),
            body: String::new(),
            scopes: vec![HashMap::new()],
        }
    }

    fn fresh(&mut self) -> String {
        let name = format!("%{}", self.next);
        self.next += 1;
        name
    }

    fn fresh_block(&mut self) -> String {
        let name = format!("^bb{}", self.block_next);
        self.block_next += 1;
        name
    }

    /// Returns (or creates) the global name holding `content` as a byte array.
    fn intern_string(&mut self, content: &str) -> String {
        if let Some(name) = self.seen_strings.get(content) {
            return name.clone();
        }
        let name = format!("@str{}", self.globals.len());
        let escaped = escape_mlir_string(content);
        let global = format!(
            "llvm.mlir.global internal constant {}(\"{}\") {{addr_space = 0 : i32}} : !llvm.array<{} x i8>",
            name,
            escaped,
            content.len()
        );
        self.globals.push(global);
        self.seen_strings.insert(content.to_string(), name.clone());
        name
    }

    /// Allocates a single value slot in the entry block, returning its pointer.
    fn alloca_value(&mut self) -> String {
        self.alloca_array(1)
    }

    /// Allocates an array of `n` value slots in the entry block.
    fn alloca_array(&mut self, n: usize) -> String {
        let cnt = self.fresh();
        self.prologue.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            cnt, n
        ));
        let slot = self.fresh();
        self.prologue.push_str(&format!(
            "    {} = llvm.alloca {} x {} {{alignment = 8 : i64}} : (i64) -> !llvm.ptr\n",
            slot, cnt, VALUE_TY
        ));
        slot
    }

    /// Emits `llvm.store` of `value` into slot `slot`.
    fn emit_store(&mut self, slot: &str, value: &str) {
        self.body.push_str(&format!(
            "    llvm.store {}, {} : {}, !llvm.ptr\n",
            value, slot, VALUE_TY
        ));
    }

    /// Emits `llvm.load` of slot `slot`, returning the loaded value name.
    fn emit_load(&mut self, slot: &str) -> String {
        let value = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.load {} : !llvm.ptr -> {}\n",
            value, slot, VALUE_TY
        ));
        value
    }

    /// Emits a `getelementptr` to element `idx` of the array at `base`.
    fn emit_gep(&mut self, base: &str, idx: usize) -> String {
        let i = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            i, idx
        ));
        let ptr = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.getelementptr {}[{}] : (!llvm.ptr, i64) -> !llvm.ptr, {}\n",
            ptr, base, i, VALUE_TY
        ));
        ptr
    }

    fn emit_print_value(&mut self, value: &str) {
        let printed = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @mita_print_value({}) : ({}) -> {}\n",
            printed, value, VALUE_TY, VALUE_TY
        ));
    }

    /// Emits a call to a binary runtime function.
    fn emit_binary(&mut self, fname: &str, a: &str, b: &str) -> String {
        let result = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @{}({}, {}) : ({}, {}) -> {}\n",
            result, fname, a, b, VALUE_TY, VALUE_TY, VALUE_TY
        ));
        result
    }

    /// Emits a call to a unary runtime function.
    fn emit_unary(&mut self, fname: &str, a: &str) -> String {
        let result = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @{}({}) : ({}) -> {}\n",
            result, fname, a, VALUE_TY, VALUE_TY
        ));
        result
    }

    /// Integer literal: `{ tag = 2, payload = v }` built with insertvalue.
    fn emit_int(&mut self, v: i64) -> String {
        self.emit_tagged(2, v)
    }

    /// Boolean literal: `{ tag = 1, payload = v ? 1 : 0 }`.
    fn emit_bool(&mut self, v: bool) -> String {
        self.emit_tagged(1, if v { 1 } else { 0 })
    }

    /// nil / nya: `{ tag = 0, payload = 0 }`.
    fn emit_nil(&mut self) -> String {
        self.emit_tagged(0, 0)
    }

    /// Builds a `{ tag, payload }` value with `llvm.insertvalue`.
    fn emit_tagged(&mut self, tag: i64, payload: i64) -> String {
        let undef = self.fresh();
        self.body
            .push_str(&format!("    {} = llvm.mlir.undef : {}\n", undef, VALUE_TY));
        let tagv = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            tagv, tag
        ));
        let with_tag = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.insertvalue {}, {}[0] : {}\n",
            with_tag, tagv, undef, VALUE_TY
        ));
        let payloadv = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            payloadv, payload
        ));
        let value = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.insertvalue {}, {}[1] : {}\n",
            value, payloadv, with_tag, VALUE_TY
        ));
        value
    }

    /// String literal via `mita_string(ptr, len)`.
    fn emit_string(&mut self, content: &str) -> String {
        let global = self.intern_string(content);
        let ptr = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.addressof {} : !llvm.ptr\n",
            ptr, global
        ));
        let len = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            len,
            content.len()
        ));
        let value = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @mita_string({}, {}) : (!llvm.ptr, i64) -> {}\n",
            value, ptr, len, VALUE_TY
        ));
        value
    }

    /// Symbol literal via `mita_symbol(ptr, len)`.
    fn emit_symbol(&mut self, content: &str) -> String {
        let global = self.intern_string(content);
        let ptr = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.addressof {} : !llvm.ptr\n",
            ptr, global
        ));
        let len = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            len,
            content.len()
        ));
        let value = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @mita_symbol({}, {}) : (!llvm.ptr, i64) -> {}\n",
            value, ptr, len, VALUE_TY
        ));
        value
    }

    /// Looks up a variable's slot in the innermost scope containing it.
    fn lookup_var(&self, name: &str) -> Option<String> {
        for scope in self.scopes.iter().rev() {
            if let Some(slot) = scope.get(name) {
                return Some(slot.clone());
            }
        }
        None
    }

    /// Compiles a constant/symbol atom (used for literals and quote).
    fn constant_value(&mut self, text: &str) -> String {
        match text {
            "nil" | "nya" => self.emit_nil(),
            "da" => self.emit_bool(true),
            "nye" => self.emit_bool(false),
            "unu" => self.emit_int(1),
            "du" => self.emit_int(2),
            "unudu" => self.emit_int(3),
            "dudu" => self.emit_int(4),
            "mani" => self.emit_int(5),
            other => self.emit_symbol(other),
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Cons { .. } => self.gen_call(expr),
            Expr::Nil => Ok(self.emit_nil()),
            Expr::Atom(tok) => match tok.typ {
                TokenType::Number => Ok(self.emit_int(tok.num)),
                TokenType::String => Ok(self.emit_string(&tok.text)),
                _ => match tok.text.as_str() {
                    "nil" | "nya" | "da" | "nye" | "unu" | "du" | "unudu" | "dudu"
                    | "mani" => Ok(self.constant_value(&tok.text)),
                    // Unbound atoms are symbols (falsy), matching the VM
                    // compiler's `predefined_value(...).unwrap_or(Symbol)`.
                    _ => match self.lookup_var(&tok.text) {
                        Some(slot) => Ok(self.emit_load(&slot)),
                        None => Ok(self.emit_symbol(&tok.text)),
                    },
                },
            },
        }
    }

    fn gen_call(&mut self, expr: &Expr) -> Result<String, String> {
        let head = expr.lawa();
        let name = match head.as_ref() {
            Expr::Atom(tok) => tok.text.clone(),
            _ => return Err("call head must be an atom".to_string()),
        };
        let args = collect_args(expr.kucha());
        match name.as_str() {
            // Special forms.
            "plata" => {
                if args.len() != 1 {
                    return Err("plata expects 1 argument".to_string());
                }
                self.gen_quote(args[0].clone())
            }
            "ka" => {
                if args.len() != 3 {
                    return Err("ka expects 3 arguments".to_string());
                }
                self.gen_if(args[0].clone(), args[1].clone(), args[2].clone())
            }
            "dala" => self.gen_cond(expr.kucha()),
            "tido" => {
                // Tree/VM semantics: bindings = first arg, body = second arg;
                // any trailing forms are ignored (tido takes a single body).
                if args.len() < 2 {
                    return Err("tido expects bindings and a body".to_string());
                }
                self.gen_let(args[0].clone(), args[1].clone())
            }
            "in" => self.gen_progn(expr.kucha()),
            "plama" => self.gen_setq(expr.kucha()),
            // Arithmetic.
            "celi" => self.gen_binary(&args, "mita_add"),
            "movo" => self.gen_binary(&args, "mita_sub"),
            "celida" => self.gen_binary(&args, "mita_mul"),
            "movoda" => self.gen_binary(&args, "mita_div"),
            // Comparisons.
            "shato" => self.gen_binary(&args, "mita_eq"),
            "nyeshato" => self.gen_binary(&args, "mita_ne"),
            "aba" => self.gen_binary(&args, "mita_lt"),
            "unta" => self.gen_binary(&args, "mita_gt"),
            "abashato" => self.gen_binary(&args, "mita_le"),
            "untashato" => self.gen_binary(&args, "mita_ge"),
            // Lists.
            "upa" => self.gen_binary(&args, "mita_cons"),
            "lawa" => self.gen_unary(&args, "mita_car"),
            "kucha" => self.gen_unary(&args, "mita_cdr"),
            // Printing.
            "print" => self.gen_print(&args),
            other => Err(format!("unsupported function: {}", other)),
        }
    }

    fn gen_binary(&mut self, args: &[Rc<Expr>], fname: &str) -> Result<String, String> {
        if args.len() != 2 {
            return Err(format!("{} expects 2 arguments", fname));
        }
        let a = self.gen_expr(&args[0])?;
        let b = self.gen_expr(&args[1])?;
        Ok(self.emit_binary(fname, &a, &b))
    }

    fn gen_unary(&mut self, args: &[Rc<Expr>], fname: &str) -> Result<String, String> {
        if args.len() != 1 {
            return Err(format!("{} expects 1 argument", fname));
        }
        let a = self.gen_expr(&args[0])?;
        Ok(self.emit_unary(fname, &a))
    }

    /// `print`: evaluate all args, store them into an entry-block array, and
    /// call `mita_print(args, argc)`.
    fn gen_print(&mut self, args: &[Rc<Expr>]) -> Result<String, String> {
        let mut vals = Vec::with_capacity(args.len());
        for arg in args {
            vals.push(self.gen_expr(arg)?);
        }
        let arr = self.alloca_array(vals.len());
        for (i, val) in vals.iter().enumerate() {
            let gep = self.emit_gep(&arr, i);
            self.emit_store(&gep, val);
        }
        let argc = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            argc,
            vals.len()
        ));
        let result = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @mita_print({}, {}) : (!llvm.ptr, i64) -> {}\n",
            result, arr, argc, VALUE_TY
        ));
        Ok(result)
    }

    /// `ka`: `llvm.cond_br` with then/else/end blocks; branch results are
    /// stored into an alloca slot and loaded after the merge.
    fn gen_if(
        &mut self,
        test_expr: Rc<Expr>,
        then_expr: Rc<Expr>,
        else_expr: Rc<Expr>,
    ) -> Result<String, String> {
        let test_val = self.gen_expr(test_expr.as_ref())?;

        let truth = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @mita_truthy({}) : ({}) -> i64\n",
            truth, test_val, VALUE_TY
        ));
        let zero = self.fresh();
        self.body
            .push_str(&format!("    {} = llvm.mlir.constant(0 : i64) : i64\n", zero));
        let cmp = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.icmp \"ne\" {}, {} : i64\n",
            cmp, truth, zero
        ));

        let slot = self.alloca_value();
        let then_bb = self.fresh_block();
        let else_bb = self.fresh_block();
        let end_bb = self.fresh_block();

        self.body.push_str(&format!(
            "    llvm.cond_br {}, {}, {}\n",
            cmp, then_bb, else_bb
        ));

        self.body.push_str(&format!("  {}:\n", then_bb));
        let then_val = self.gen_expr(then_expr.as_ref())?;
        self.emit_store(&slot, &then_val);
        self.body.push_str(&format!("    llvm.br {}\n", end_bb));

        self.body.push_str(&format!("  {}:\n", else_bb));
        let else_val = self.gen_expr(else_expr.as_ref())?;
        self.emit_store(&slot, &else_val);
        self.body.push_str(&format!("    llvm.br {}\n", end_bb));

        self.body.push_str(&format!("  {}:\n", end_bb));
        Ok(self.emit_load(&slot))
    }

    /// `dala`: a nested `ka` matching the VM/tree semantics exactly.
    fn gen_cond(&mut self, clauses: Rc<Expr>) -> Result<String, String> {
        if clauses.is_nil() {
            return Err("no true case in cond".to_string());
        }
        let clause = clauses.lawa();
        let remaining = clauses.kucha();

        // A bare atom clause evaluates directly.
        if clause.is_atom() {
            return self.gen_expr(clause.as_ref());
        }

        let test_expr = clause.lawa();
        let rest = clause.kucha();

        let test_val = self.gen_expr(test_expr.as_ref())?;

        let truth = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @mita_truthy({}) : ({}) -> i64\n",
            truth, test_val, VALUE_TY
        ));
        let zero = self.fresh();
        self.body
            .push_str(&format!("    {} = llvm.mlir.constant(0 : i64) : i64\n", zero));
        let cmp = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.icmp \"ne\" {}, {} : i64\n",
            cmp, truth, zero
        ));

        let slot = self.alloca_value();
        let then_bb = self.fresh_block();
        let else_bb = self.fresh_block();
        let end_bb = self.fresh_block();

        self.body.push_str(&format!(
            "    llvm.cond_br {}, {}, {}\n",
            cmp, then_bb, else_bb
        ));

        // Truthy: evaluate the test again if the clause has no body, otherwise
        // the first body expression.
        self.body.push_str(&format!("  {}:\n", then_bb));
        let then_val = if rest.is_nil() {
            self.gen_expr(test_expr.as_ref())?
        } else {
            self.gen_expr(rest.lawa().as_ref())?
        };
        self.emit_store(&slot, &then_val);
        self.body.push_str(&format!("    llvm.br {}\n", end_bb));

        // Falsy: the last clause is an implicit else — re-evaluate the whole
        // clause as an expression — otherwise recurse into the next clause.
        self.body.push_str(&format!("  {}:\n", else_bb));
        let else_val = if remaining.is_nil() {
            self.gen_expr(clause.as_ref())?
        } else {
            self.gen_cond(remaining)?
        };
        self.emit_store(&slot, &else_val);
        self.body.push_str(&format!("    llvm.br {}\n", end_bb));

        self.body.push_str(&format!("  {}:\n", end_bb));
        Ok(self.emit_load(&slot))
    }

    /// `tido`: allocate a slot for each binding, store its value, then
    /// evaluate the body in a fresh scope.
    fn gen_let(&mut self, bindings: Rc<Expr>, body: Rc<Expr>) -> Result<String, String> {
        self.scopes.push(HashMap::new());

        let result = (|| -> Result<String, String> {
            let mut current = bindings;
            while let Expr::Cons { lawa, kucha } = current.as_ref() {
                let var_name = lawa
                    .lawa()
                    .get_sada()
                    .map(|t| t.text.clone())
                    .ok_or_else(|| "tido: expected variable name".to_string())?;
                let val_expr = lawa.kucha().lawa();
                let val = self.gen_expr(val_expr.as_ref())?;
                let slot = self.alloca_value();
                self.emit_store(&slot, &val);
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert(var_name, slot);
                }
                current = kucha.clone();
            }
            self.gen_expr(body.as_ref())
        })();

        self.scopes.pop();
        result
    }

    /// `in`: evaluate expressions in order; the last value is the result.
    fn gen_progn(&mut self, exprs: Rc<Expr>) -> Result<String, String> {
        let mut result = self.emit_nil();
        let mut current = exprs;
        while let Expr::Cons { lawa, kucha } = current.as_ref() {
            result = self.gen_expr(lawa.as_ref())?;
            current = kucha.clone();
        }
        Ok(result)
    }

    /// `plama`: store to an existing local slot; an unknown variable is a
    /// compile error. Supports multiple var/value pairs.
    fn gen_setq(&mut self, args: Rc<Expr>) -> Result<String, String> {
        let mut current = args;
        let mut result = self.emit_nil();
        while let Expr::Cons { lawa, kucha } = current.as_ref() {
            let var_name = lawa
                .get_sada()
                .map(|t| t.text.clone())
                .ok_or_else(|| "plama: expected variable name".to_string())?;
            let val_expr = match kucha.as_ref() {
                Expr::Cons { lawa, .. } => lawa.clone(),
                _ => return Err("plama: odd number of args".to_string()),
            };
            let val = self.gen_expr(val_expr.as_ref())?;
            let slot = self
                .lookup_var(&var_name)
                .ok_or_else(|| format!("plama: undefined variable {}", var_name))?;
            self.emit_store(&slot, &val);
            result = val;
            current = match kucha.as_ref() {
                Expr::Cons { kucha, .. } => kucha.clone(),
                _ => break,
            };
        }
        Ok(result)
    }

    /// `plata` (quote): literals become constants; lists are built recursively
    /// with `mita_cons`.
    fn gen_quote(&mut self, expr: Rc<Expr>) -> Result<String, String> {
        match expr.as_ref() {
            Expr::Nil => Ok(self.emit_nil()),
            Expr::Atom(tok) => match tok.typ {
                TokenType::Number => Ok(self.emit_int(tok.num)),
                TokenType::String => Ok(self.emit_string(&tok.text)),
                _ => Ok(self.constant_value(&tok.text)),
            },
            Expr::Cons { .. } => {
                let car = self.gen_quote(expr.lawa())?;
                let cdr = self.gen_quote(expr.kucha())?;
                Ok(self.emit_binary("mita_cons", &car, &cdr))
            }
        }
    }
}

/// Collects the argument list of a call expression (the `kucha` tail).
fn collect_args(mut list: Rc<Expr>) -> Vec<Rc<Expr>> {
    let mut args = Vec::new();
    while let Expr::Cons { lawa, kucha } = list.as_ref() {
        args.push(lawa.clone());
        list = kucha.clone();
    }
    args
}

/// Escapes a Rust string into an MLIR string literal body (for
/// `llvm.mlir.global`). Backslashes and quotes are escaped; non-printable
/// bytes become `\XX` hex escapes (MLIR's string literal convention).
fn escape_mlir_string(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            0x20..=0x7e => out.push(b as char),
            _ => out.push_str(&format!("\\{:02X}", b)),
        }
    }
    out
}
