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
//! `plata`, `print`, top-level sequencing, an automatic print of the final
//! top-level result (mirroring the interpreter's `load()`), fixed-arity
//! `muhe` functions with two-phase name resolution, non-capturing `mita`
//! lambdas, and closure creation / dynamic calls through `mita_call`.

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
    "llvm.func @mita_make_closure(!llvm.ptr, i64) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_call(!llvm.struct<(i64, i64)>, !llvm.ptr, i64) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_panic(!llvm.ptr)",
];

/// A parsed function definition pending compilation.
struct FnDef {
    /// MLIR symbol name including the leading `@` (e.g. `@fak`, `@lambda0`).
    name: String,
    params: Vec<String>,
    body: Rc<Expr>,
    /// Names of enclosing locals at the definition site. Referencing any of
    /// these from the body is an upvalue capture and a compile error.
    banned: Vec<String>,
}

/// Module-wide state shared across every emitted function.
struct Shared {
    next: usize,
    block_next: usize,
    globals: Vec<String>,
    seen_strings: HashMap<String, String>,
    /// `muhe` function source name (no `@`) -> arity, for direct calls and
    /// closure construction.
    known_fns: HashMap<String, usize>,
    lambda_count: usize,
    /// Lambdas discovered while compiling a body, awaiting their own body.
    lambda_queue: Vec<FnDef>,
}

/// Per-function compilation state.
struct FnCtx<'a> {
    shared: &'a mut Shared,
    /// Entry-block `llvm.alloca`s, emitted before the instruction stream so
    /// every slot dominates its uses across branches.
    prologue: String,
    /// The instruction stream (entry block then any branch blocks).
    body: String,
    /// Lexical variable scopes, mapping a name to its alloca slot SSA name.
    scopes: Vec<HashMap<String, String>>,
    /// Capturing any of these names is a compile error (lambda bodies only).
    banned: Vec<String>,
}

/// Builds the `module { ... }` text for the given top-level expressions.
pub fn compile(exprs: &[Rc<Expr>]) -> Result<String, String> {
    // Phase 1: scan every top-level `muhe` and collect function definitions
    // and names, so bodies (and top-level calls) can forward-reference them.
    let (mut worklist, known_fns) = collect_muhe(exprs)?;

    let mut shared = Shared {
        next: 0,
        block_next: 0,
        globals: Vec::new(),
        seen_strings: HashMap::new(),
        known_fns,
        lambda_count: 0,
        lambda_queue: Vec::new(),
    };

    let mut functions: Vec<String> = Vec::new();

    // Compile `main` first: it may discover lambdas, which are queued and then
    // compiled alongside the `muhe` bodies below. Function definitions may be
    // emitted in any order; MLIR resolves `llvm.call` symbols module-wide.
    let main = compile_main(&mut shared, exprs)?;

    loop {
        let def = match worklist.pop() {
            Some(d) => Some(d),
            None => shared.lambda_queue.pop(),
        };
        let Some(def) = def else { break };
        emit_function(&mut shared, def, &mut functions)?;
    }

    functions.push(main);

    let mut out = String::new();
    out.push_str("module {\n");
    for decl in DECLS {
        out.push_str("  ");
        out.push_str(decl);
        out.push('\n');
    }
    for global in &shared.globals {
        out.push_str("  ");
        out.push_str(global);
        out.push('\n');
    }
    for func in &functions {
        out.push_str(func);
    }
    out.push_str("}\n");

    Ok(out)
}

/// Scans top-level `muhe` forms and returns their parsed definitions plus a
/// source-name -> arity map.
fn collect_muhe(exprs: &[Rc<Expr>]) -> Result<(Vec<FnDef>, HashMap<String, usize>), String> {
    let mut fns = Vec::new();
    let mut known = HashMap::new();
    for expr in exprs {
        let Expr::Cons { lawa, kucha } = expr.as_ref() else {
            continue;
        };
        let Some(tok) = lawa.get_sada() else {
            continue;
        };
        if tok.text != "muhe" {
            continue;
        }
        let defs = kucha.lawa();
        let mut current = defs;
        while let Expr::Cons { lawa: def, kucha: rest } = current.as_ref() {
            let name_tok = def
                .lawa()
                .get_sada()
                .ok_or_else(|| "muhe: expected function name".to_string())?;
            let name = name_tok.text.clone();
            let lambda = def.kucha().lawa();
            let (params, body) = parse_lambda(&lambda)?;
            let arity = params.len();
            known.insert(name.clone(), arity);
            fns.push(FnDef {
                name: format!("@{}", name),
                params,
                body,
                banned: Vec::new(),
            });
            current = rest.clone();
        }
    }
    Ok((fns, known))
}

/// Parses `(mita (params...) body)` into its parameter names and body form.
fn parse_lambda(lambda: &Rc<Expr>) -> Result<(Vec<String>, Rc<Expr>), String> {
    let formals = lambda.kucha().lawa();
    let body = lambda.kucha().kucha().lawa();
    let params = parse_formals(&formals)?;
    Ok((params, body))
}

/// Extracts a fixed-arity parameter list from a lambda formal list.
fn parse_formals(formals: &Rc<Expr>) -> Result<Vec<String>, String> {
    match formals.as_ref() {
        Expr::Atom(_) => Err("variadic lambda not supported".to_string()),
        Expr::Nil => Ok(Vec::new()),
        Expr::Cons { .. } => {
            let mut params = Vec::new();
            let mut f = formals.clone();
            while let Expr::Cons { lawa, kucha } = f.as_ref() {
                let tok = lawa
                    .get_sada()
                    .ok_or_else(|| "lambda: expected parameter name".to_string())?;
                params.push(tok.text.clone());
                f = kucha.clone();
            }
            Ok(params)
        }
    }
}

/// Compiles the `main` function: runs each top-level expression in order
/// (skipping `muhe`), auto-prints the final result, and returns 0.
fn compile_main(shared: &mut Shared, exprs: &[Rc<Expr>]) -> Result<String, String> {
    let mut ctx = FnCtx::new(shared, Vec::new());

    let mut last: Option<String> = None;
    for expr in exprs {
        if is_muhe(expr) {
            continue;
        }
        last = Some(ctx.gen_expr(expr)?);
    }

    if let Some(result) = last {
        ctx.emit_print_value(&result);
    }

    let zero = ctx.fresh();
    ctx.body
        .push_str(&format!("    {} = llvm.mlir.constant(0 : i32) : i32\n", zero));
    ctx.body
        .push_str(&format!("    llvm.return {} : i32\n", zero));

    let mut s = String::new();
    s.push_str("  llvm.func @main() -> i32 {\n");
    s.push_str(&ctx.prologue);
    s.push_str(&ctx.body);
    s.push_str("  }\n");
    Ok(s)
}

fn is_muhe(expr: &Rc<Expr>) -> bool {
    match expr.as_ref() {
        Expr::Cons { lawa, .. } => lawa
            .get_sada()
            .map(|t| t.text == "muhe")
            .unwrap_or(false),
        _ => false,
    }
}

/// Compiles one function (a `muhe` def or a lambda) into a full `llvm.func`.
fn emit_function(shared: &mut Shared, def: FnDef, out: &mut Vec<String>) -> Result<(), String> {
    let mut ctx = FnCtx::new(shared, def.banned);

    // Map each parameter to an alloca slot and load it from the args array.
    for (i, pname) in def.params.iter().enumerate() {
        let slot = ctx.alloca_value();
        ctx.scopes[0].insert(pname.clone(), slot.clone());
        let gep = ctx.emit_gep("%args", i);
        let val = ctx.emit_load(&gep);
        ctx.emit_store(&slot, &val);
    }

    let result = ctx.gen_expr(&def.body)?;
    ctx.body
        .push_str(&format!("    llvm.return {} : {}\n", result, VALUE_TY));

    let mut s = String::new();
    s.push_str(&format!(
        "  llvm.func {}(%args: !llvm.ptr, %argc: i64) -> {} {{\n",
        def.name, VALUE_TY
    ));
    s.push_str(&ctx.prologue);
    s.push_str(&ctx.body);
    s.push_str("  }\n");
    out.push(s);
    Ok(())
}

impl<'a> FnCtx<'a> {
    fn new(shared: &'a mut Shared, banned: Vec<String>) -> Self {
        FnCtx {
            shared,
            prologue: String::new(),
            body: String::new(),
            scopes: vec![HashMap::new()],
            banned,
        }
    }

    fn fresh(&mut self) -> String {
        let name = format!("%{}", self.shared.next);
        self.shared.next += 1;
        name
    }

    fn fresh_block(&mut self) -> String {
        let name = format!("^bb{}", self.shared.block_next);
        self.shared.block_next += 1;
        name
    }

    /// Returns (or creates) the global name holding `content` as a byte array.
    fn intern_string(&mut self, content: &str) -> String {
        if let Some(name) = self.shared.seen_strings.get(content) {
            return name.clone();
        }
        let name = format!("@str{}", self.shared.globals.len());
        // Emit a NUL-terminated array. Length-delimited consumers
        // (`mita_string`/`mita_symbol`) pass an explicit length and ignore the
        // extra byte; C-string consumers (`mita_panic`) rely on the `\00`
        // terminator to avoid an out-of-bounds read.
        let mut escaped = escape_mlir_string(content);
        escaped.push_str("\\00");
        let global = format!(
            "llvm.mlir.global internal constant {}(\"{}\") {{addr_space = 0 : i32}} : !llvm.array<{} x i8>",
            name,
            escaped,
            content.len() + 1
        );
        self.shared.globals.push(global);
        self.shared
            .seen_strings
            .insert(content.to_string(), name.clone());
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

    /// Emits an `i64` constant, returning its SSA name.
    fn emit_const_i64(&mut self, v: i64) -> String {
        let name = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            name, v
        ));
        name
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

    /// Creates a closure value wrapping the function `name` of `arity` args.
    fn emit_closure(&mut self, name: &str, arity: usize) -> String {
        let ptr = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.mlir.addressof {} : !llvm.ptr\n",
            ptr, name
        ));
        let arity_val = self.emit_const_i64(arity as i64);
        let result = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @mita_make_closure({}, {}) : (!llvm.ptr, i64) -> {}\n",
            result, ptr, arity_val, VALUE_TY
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
                    // Name resolution: local slot -> known muhe -> capture
                    // error -> unbound symbol (falsy, matching the VM).
                    _ => {
                        if let Some(slot) = self.lookup_var(&tok.text) {
                            Ok(self.emit_load(&slot))
                        } else if let Some(arity) = self.shared.known_fns.get(&tok.text).copied() {
                            Ok(self.emit_closure(&format!("@{}", tok.text), arity))
                        } else if self.banned.contains(&tok.text) {
                            Err(format!(
                                "lambda captures outer variable: {}",
                                tok.text
                            ))
                        } else {
                            Ok(self.emit_symbol(&tok.text))
                        }
                    }
                },
            },
        }
    }

    fn gen_call(&mut self, expr: &Expr) -> Result<String, String> {
        let head = expr.lawa();
        let args = collect_args(expr.kucha());

        let name = match head.as_ref() {
            Expr::Atom(tok) => tok.text.clone(),
            // A non-atom head is a value (e.g. a lambda): dynamic call.
            _ => return self.gen_dynamic_call(head, &args),
        };

        // Special forms are reserved syntax and cannot be shadowed by locals.
        match name.as_str() {
            "plata" => {
                if args.len() != 1 {
                    return Err("plata expects 1 argument".to_string());
                }
                return self.gen_quote(args[0].clone());
            }
            "ka" => {
                if args.len() != 3 {
                    return Err("ka expects 3 arguments".to_string());
                }
                return self.gen_if(args[0].clone(), args[1].clone(), args[2].clone());
            }
            "dala" => return self.gen_cond(expr.kucha()),
            "tido" => {
                if args.len() < 2 {
                    return Err("tido expects bindings and a body".to_string());
                }
                return self.gen_let(args[0].clone(), args[1].clone());
            }
            "in" => return self.gen_progn(expr.kucha()),
            "plama" => return self.gen_setq(expr.kucha()),
            "mita" => {
                if args.len() < 2 {
                    return Err("mita expects formals and a body".to_string());
                }
                return self.gen_lambda(args[0].clone(), args[1].clone());
            }
            _ => {}
        }

        // Ordinary call heads resolve local slot -> builtin -> known muhe ->
        // error. A local shadowing a builtin or `muhe` name wins.
        if self.lookup_var(&name).is_some() {
            return self.gen_dynamic_call(head, &args);
        }
        if let Some(builtin) = builtin_for(&name) {
            return match builtin {
                Builtin::Binary(f) => self.gen_binary(&args, f),
                Builtin::Unary(f) => self.gen_unary(&args, f),
                Builtin::Print => self.gen_print(&args),
            };
        }
        if let Some(arity) = self.shared.known_fns.get(&name).copied() {
            return self.gen_direct_call(&name, arity, &args);
        }
        Err(format!("unsupported function: {}", name))
    }

    /// A compile-time-known `muhe` function: direct `llvm.call @fn(args, argc)`.
    fn gen_direct_call(
        &mut self,
        name: &str,
        arity: usize,
        args: &[Rc<Expr>],
    ) -> Result<String, String> {
        if args.len() != arity {
            return Err(format!(
                "{}: expected {} arguments, got {}",
                name,
                arity,
                args.len()
            ));
        }
        let mut vals = Vec::with_capacity(args.len());
        for arg in args {
            vals.push(self.gen_expr(arg)?);
        }
        let arr = self.alloca_array(vals.len());
        for (i, val) in vals.iter().enumerate() {
            let gep = self.emit_gep(&arr, i);
            self.emit_store(&gep, val);
        }
        let argc = self.emit_const_i64(vals.len() as i64);
        let result = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @{}({}, {}) : (!llvm.ptr, i64) -> {}\n",
            result, name, arr, argc, VALUE_TY
        ));
        Ok(result)
    }

    /// A runtime closure value (local var or expression): `mita_call(fn, args)`.
    fn gen_dynamic_call(&mut self, head: Rc<Expr>, args: &[Rc<Expr>]) -> Result<String, String> {
        // Evaluate args first, then the function position (matches the VM
        // compiler's ordering).
        let mut vals = Vec::with_capacity(args.len());
        for arg in args {
            vals.push(self.gen_expr(arg)?);
        }
        let fn_val = self.gen_expr(head.as_ref())?;

        let arr = self.alloca_array(vals.len());
        for (i, val) in vals.iter().enumerate() {
            let gep = self.emit_gep(&arr, i);
            self.emit_store(&gep, val);
        }
        let argc = self.emit_const_i64(vals.len() as i64);
        let result = self.fresh();
        self.body.push_str(&format!(
            "    {} = llvm.call @mita_call({}, {}, {}) : ({}, !llvm.ptr, i64) -> {}\n",
            result, fn_val, arr, argc, VALUE_TY, VALUE_TY
        ));
        Ok(result)
    }

    /// `mita` lambda: queue its body as a function and build a closure value.
    fn gen_lambda(&mut self, formals: Rc<Expr>, body: Rc<Expr>) -> Result<String, String> {
        let params = parse_formals(&formals)?;
        let arity = params.len();

        // Capture ban: any enclosing local is off-limits.
        let banned: Vec<String> = self
            .scopes
            .iter()
            .flat_map(|s| s.keys().cloned())
            .collect();

        let name = format!("@lambda{}", self.shared.lambda_count);
        self.shared.lambda_count += 1;
        self.shared.lambda_queue.push(FnDef {
            name: name.clone(),
            params,
            body,
            banned,
        });

        Ok(self.emit_closure(&name, arity))
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
        let argc = self.emit_const_i64(vals.len() as i64);
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

    /// Whether a call head can be compiled by `gen_call` under the resolution
    /// rules (local slot → builtin → known `muhe` → special form, or a
    /// non-atom head as a dynamic call). Used by `gen_cond` to decide whether
    /// the last-clause implicit-else fallback is actually compilable: an
    /// uncallable head is a *runtime* error (the fallback may be unreachable),
    /// not a compile error.
    fn head_is_callable(&self, head: &Expr) -> bool {
        match head {
            Expr::Atom(tok) => {
                let name = tok.text.as_str();
                is_special_form(name)
                    || self.lookup_var(name).is_some()
                    || builtin_for(name).is_some()
                    || self.shared.known_fns.contains_key(name)
            }
            // A non-atom head is a value (e.g. a lambda): dynamic call.
            _ => true,
        }
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
        if remaining.is_nil() {
            // The implicit-else fallback re-evaluates the whole clause only
            // when its head is callable. An uncallable head (e.g. `(da 'no)`)
            // would be a compile error under `gen_expr`, but the fallback may
            // be unreachable at runtime; mirror the tree interpreter by only
            // failing if this branch is actually taken.
            if self.head_is_callable(clause.lawa().as_ref()) {
                let else_val = self.gen_expr(clause.as_ref())?;
                self.emit_store(&slot, &else_val);
                self.body.push_str(&format!("    llvm.br {}\n", end_bb));
            } else {
                let msg = self.intern_string("cond: implicit else clause is not callable");
                let ptr = self.fresh();
                self.body.push_str(&format!(
                    "    {} = llvm.mlir.addressof {} : !llvm.ptr\n",
                    ptr, msg
                ));
                self.body.push_str(&format!(
                    "    llvm.call @mita_panic({}) : (!llvm.ptr) -> ()\n",
                    ptr
                ));
                self.body.push_str("    llvm.unreachable\n");
            }
        } else {
            let else_val = self.gen_cond(remaining)?;
            self.emit_store(&slot, &else_val);
            self.body.push_str(&format!("    llvm.br {}\n", end_bb));
        }

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

/// A builtin function the compiler can emit a direct runtime call for.
enum Builtin {
    Binary(&'static str),
    Unary(&'static str),
    Print,
}

/// Whether `name` is a special form (reserved syntax, not shadowable).
fn is_special_form(name: &str) -> bool {
    matches!(
        name,
        "plata" | "ka" | "dala" | "tido" | "in" | "plama" | "mita"
    )
}

/// Maps a builtin call name to its runtime lowering, if it is one.
fn builtin_for(name: &str) -> Option<Builtin> {
    Some(match name {
        "celi" => Builtin::Binary("mita_add"),
        "movo" => Builtin::Binary("mita_sub"),
        "celida" => Builtin::Binary("mita_mul"),
        "movoda" => Builtin::Binary("mita_div"),
        "shato" => Builtin::Binary("mita_eq"),
        "nyeshato" => Builtin::Binary("mita_ne"),
        "aba" => Builtin::Binary("mita_lt"),
        "unta" => Builtin::Binary("mita_gt"),
        "abashato" => Builtin::Binary("mita_le"),
        "untashato" => Builtin::Binary("mita_ge"),
        "upa" => Builtin::Binary("mita_cons"),
        "lawa" => Builtin::Unary("mita_car"),
        "kucha" => Builtin::Unary("mita_cdr"),
        "print" => Builtin::Print,
        _ => return None,
    })
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
