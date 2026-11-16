//! MLIR LLVM-dialect code generation for MITA.
//!
//! Lowers a list of top-level expressions to a single `module { ... }` of
//! LLVM dialect IR. Every MITA value is `!llvm.struct<(i64, i64)>` (the
//! `mita_value { int64_t tag; int64_t payload; }` C ABI). Scalars are built
//! with `llvm.insertvalue`; heap-backed values (strings/symbols) are built by
//! calling the C runtime constructors.
//!
//! Task 3 scope: integer literals, `celi` (mita_add), `print`
//! (mita_print_value), top-level sequencing, and an automatic print of the
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
    "llvm.func @mita_print_value(!llvm.struct<(i64, i64)>) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_string(!llvm.ptr, i64) -> !llvm.struct<(i64, i64)>",
    "llvm.func @mita_symbol(!llvm.ptr, i64) -> !llvm.struct<(i64, i64)>",
];

/// Builds the `module { ... }` text for the given top-level expressions.
pub fn compile(exprs: &[Rc<Expr>]) -> Result<String, String> {
    let mut gen = Codegen::new();
    let mut body = String::new();

    let mut last: Option<String> = None;
    for expr in exprs {
        last = Some(gen.gen_expr(expr, &mut body)?);
    }

    // Auto-print the final top-level result, like the interpreter's `load()`.
    if let Some(result) = last {
        let printed = gen.fresh();
        body.push_str(&format!(
            "    {} = llvm.call @mita_print_value({}) : ({}) -> {}\n",
            printed, result, VALUE_TY, VALUE_TY
        ));
    }

    let zero = gen.fresh();
    body.push_str(&format!("    {} = llvm.mlir.constant(0 : i32) : i32\n", zero));
    body.push_str(&format!("    llvm.return {} : i32\n", zero));

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
    out.push_str(&body);
    out.push_str("  }\n");
    out.push_str("}\n");

    Ok(out)
}

struct Codegen {
    next: usize,
    globals: Vec<String>,
    seen_strings: HashMap<String, String>,
}

impl Codegen {
    fn new() -> Self {
        Codegen {
            next: 0,
            globals: Vec::new(),
            seen_strings: HashMap::new(),
        }
    }

    fn fresh(&mut self) -> String {
        let name = format!("%{}", self.next);
        self.next += 1;
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

    /// Integer literal: `{ tag = 2, payload = v }` built with insertvalue.
    fn emit_int(&mut self, v: i64, out: &mut String) -> String {
        let undef = self.fresh();
        out.push_str(&format!("    {} = llvm.mlir.undef : {}\n", undef, VALUE_TY));
        let tag = self.fresh();
        out.push_str(&format!("    {} = llvm.mlir.constant(2 : i64) : i64\n", tag));
        let with_tag = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.insertvalue {}, {}[0] : {}\n",
            with_tag, tag, undef, VALUE_TY
        ));
        let payload = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            payload, v
        ));
        let value = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.insertvalue {}, {}[1] : {}\n",
            value, payload, with_tag, VALUE_TY
        ));
        value
    }

    /// nil / nya: `{ tag = 0, payload = 0 }`.
    fn emit_nil(&mut self, out: &mut String) -> String {
        let undef = self.fresh();
        out.push_str(&format!("    {} = llvm.mlir.undef : {}\n", undef, VALUE_TY));
        let zero = self.fresh();
        out.push_str(&format!("    {} = llvm.mlir.constant(0 : i64) : i64\n", zero));
        let with_tag = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.insertvalue {}, {}[0] : {}\n",
            with_tag, zero, undef, VALUE_TY
        ));
        let value = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.insertvalue {}, {}[1] : {}\n",
            value, zero, with_tag, VALUE_TY
        ));
        value
    }

    /// Boolean literal: `{ tag = 1, payload = v ? 1 : 0 }`.
    fn emit_bool(&mut self, v: bool, out: &mut String) -> String {
        let undef = self.fresh();
        out.push_str(&format!("    {} = llvm.mlir.undef : {}\n", undef, VALUE_TY));
        let tag = self.fresh();
        out.push_str(&format!("    {} = llvm.mlir.constant(1 : i64) : i64\n", tag));
        let with_tag = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.insertvalue {}, {}[0] : {}\n",
            with_tag, tag, undef, VALUE_TY
        ));
        let payload = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            payload,
            if v { 1 } else { 0 }
        ));
        let value = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.insertvalue {}, {}[1] : {}\n",
            value, payload, with_tag, VALUE_TY
        ));
        value
    }

    /// String literal via `mita_string(ptr, len)`.
    fn emit_string(&mut self, content: &str, out: &mut String) -> String {
        let global = self.intern_string(content);
        let ptr = self.fresh();
        out.push_str(&format!("    {} = llvm.mlir.addressof {} : !llvm.ptr\n", ptr, global));
        let len = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            len,
            content.len()
        ));
        let value = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.call @mita_string({}, {}) : (!llvm.ptr, i64) -> {}\n",
            value, ptr, len, VALUE_TY
        ));
        value
    }

    /// Symbol literal via `mita_symbol(ptr, len)`. Reachable only through the
    /// `plata` (quote) special form, which lands in a later task; kept here so
    /// the constant helpers match the plan.
    #[allow(dead_code)]
    fn emit_symbol(&mut self, content: &str, out: &mut String) -> String {
        let global = self.intern_string(content);
        let ptr = self.fresh();
        out.push_str(&format!("    {} = llvm.mlir.addressof {} : !llvm.ptr\n", ptr, global));
        let len = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.mlir.constant({} : i64) : i64\n",
            len,
            content.len()
        ));
        let value = self.fresh();
        out.push_str(&format!(
            "    {} = llvm.call @mita_symbol({}, {}) : (!llvm.ptr, i64) -> {}\n",
            value, ptr, len, VALUE_TY
        ));
        value
    }

    fn gen_expr(&mut self, expr: &Expr, out: &mut String) -> Result<String, String> {
        match expr {
            Expr::Cons { .. } => self.gen_call(expr, out),
            Expr::Nil => Ok(self.emit_nil(out)),
            Expr::Atom(tok) => match tok.typ {
                TokenType::Number => Ok(self.emit_int(tok.num, out)),
                TokenType::String => Ok(self.emit_string(&tok.text, out)),
                _ => match tok.text.as_str() {
                    "nil" | "nya" => Ok(self.emit_nil(out)),
                    "da" => Ok(self.emit_bool(true, out)),
                    "nye" => Ok(self.emit_bool(false, out)),
                    // Hilichurlian number words, matching eval::Context::new().
                    "unu" => Ok(self.emit_int(1, out)),
                    "du" => Ok(self.emit_int(2, out)),
                    "unudu" => Ok(self.emit_int(3, out)),
                    "dudu" => Ok(self.emit_int(4, out)),
                    "mani" => Ok(self.emit_int(5, out)),
                    other => Err(format!("unsupported atom: {}", other)),
                },
            },
        }
    }

    fn gen_call(&mut self, expr: &Expr, out: &mut String) -> Result<String, String> {
        let head = expr.lawa();
        let name = match head.as_ref() {
            Expr::Atom(tok) => tok.text.clone(),
            _ => return Err("call head must be an atom".to_string()),
        };
        let args = collect_args(expr.kucha());
        match name.as_str() {
            "celi" => {
                if args.len() != 2 {
                    return Err("celi expects 2 arguments".to_string());
                }
                let a = self.gen_expr(&args[0], out)?;
                let b = self.gen_expr(&args[1], out)?;
                let result = self.fresh();
                out.push_str(&format!(
                    "    {} = llvm.call @mita_add({}, {}) : ({}, {}) -> {}\n",
                    result, a, b, VALUE_TY, VALUE_TY, VALUE_TY
                ));
                Ok(result)
            }
            "print" => {
                if args.len() != 1 {
                    return Err("print expects 1 argument in this MVP".to_string());
                }
                let a = self.gen_expr(&args[0], out)?;
                let result = self.fresh();
                out.push_str(&format!(
                    "    {} = llvm.call @mita_print_value({}) : ({}) -> {}\n",
                    result, a, VALUE_TY, VALUE_TY
                ));
                Ok(result)
            }
            other => Err(format!("unsupported function: {}", other)),
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
