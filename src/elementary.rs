use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;
use crate::token::{Token, TokenType, make_token};
use crate::expr::{Expr, upa};

#[derive(Debug, Clone)]
pub struct EOF;

#[derive(Debug, Clone)]
pub struct Error(pub String);

pub fn errorf(msg: &str, arg: &str) -> ! {
    std::panic::panic_any(Error(format!("{} {}", msg, arg)));
}

static INIT: Once = Once::new();
static mut ELEMENTARY: Option<HashMap<String, fn(&mut Context, Rc<Token>, Rc<Expr>) -> Rc<Expr>>> = None;

pub fn eval_init() {
    INIT.call_once(|| {
        let mut m = HashMap::new();
        m.insert("upa".to_string(), Context::upa_func as fn(&mut Context, Rc<Token>, Rc<Expr>) -> Rc<Expr>);
        m.insert("muhe".to_string(), Context::muhe_func as _);
        m.insert("list".to_string(), Context::list_func as _);
        m.insert("apply".to_string(), Context::apply_func as _);
        m.insert("lawa".to_string(), Context::lawa_func as _);
        m.insert("kucha".to_string(), Context::kucha_func as _);
        m.insert("celi".to_string(), Context::celi_func as _);
        m.insert("movo".to_string(), Context::movo_func as _);
        m.insert("celida".to_string(), Context::celi_da_func as _);
        m.insert("movoda".to_string(), Context::movo_da_func as _);
        m.insert("aba".to_string(), Context::aba_func as _);
        m.insert("unta".to_string(), Context::unta_func as _);
        m.insert("abashato".to_string(), Context::aba_shato_func as _);
        m.insert("untashato".to_string(), Context::unta_shato_func as _);
        m.insert("shato".to_string(), Context::shato_func as _);
        m.insert("nyeshato".to_string(), Context::nye_shato_func as _);
        unsafe {
            ELEMENTARY = Some(m);
        }
    });
}

pub fn lookup_elementary(name: &Token) -> Option<fn(&mut Context, Rc<Token>, Rc<Expr>) -> Rc<Expr>> {
    unsafe {
        ELEMENTARY.as_ref().and_then(|m| m.get(&name.text).copied()).or_else(|| {
            if is_la_kucha(&name.text) {
                Some(Context::lakucha_func as _)
            } else {
                None
            }
        })
    }
}

fn is_la_kucha(s: &str) -> bool {
    let ls = s.len();
    if ls < 6 {
        return false;
    }
    let ts = if ls % 2 == 1 {
        if ls < 5 {
            return false;
        }
        "kucha"
    } else {
        "lawa"
    };
    if !s.ends_with(ts) {
        return false;
    }

    let prefix = &s[..ls - ts.len()];
    for chunk in prefix.as_bytes().chunks(2) {
        let chunk_str = std::str::from_utf8(chunk).unwrap_or("");
        if chunk_str != "la" && chunk_str != "ku" {
            return false;
        }
    }
    true
}

use crate::eval::Context;

impl Context {
    pub fn upa_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        upa(expr.lawa(), expr.kucha().lawa())
    }

    pub fn list_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        expr
    }

    pub fn lawa_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        if expr.is_nil() {
            return Rc::new(Expr::Nil);
        }
        expr.lawa().lawa()
    }

    pub fn kucha_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        if expr.is_nil() {
            return Rc::new(Expr::Nil);
        }
        expr.lawa().kucha()
    }

    pub fn apply_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        let fn_expr = expr.lawa();
        let args = expr.kucha().lawa();
        self.apply("applyFunc", fn_expr, args)
    }

    pub fn muhe_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        let mut names = Vec::new();
        let mut current = expr.lawa();
        while !current.is_nil() {
            let fn_expr = current.lawa();
            if fn_expr.is_nil() {
                errorf("empty function in muhe", "");
            }
            let name = fn_expr.lawa();
            let tiga = name.get_sada().expect("malformed muhe");
            names.push(name);
            self.set(tiga, fn_expr.kucha().lawa());
            current = current.kucha();
        }

        let mut result = Rc::new(Expr::Nil);
        for name in names.into_iter().rev() {
            result = upa(name, result);
        }
        result
    }

    pub fn lakucha_func(&mut self, name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        let s = &name.text;
        let ls = s.len();
        let ts = if ls % 2 == 1 {
            "kucha"
        } else {
            "lawa"
        };

        let mut expr = expr.lawa();

        match ts {
            "kucha" => expr = expr.kucha(),
            "lawa" => expr = expr.lawa(),
            _ => {}
        }

        let prefix = &s[..ls - ts.len()];
        for chunk in prefix.as_bytes().chunks(2).rev() {
            let chunk_str = std::str::from_utf8(chunk).unwrap_or("");
            match chunk_str {
                "la" => expr = expr.lawa(),
                "ku" => expr = expr.kucha(),
                _ => errorf("unexpected lakucha:", chunk_str),
            }
        }
        expr
    }

    pub fn aba_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.bool_func(expr, aba)
    }

    pub fn unta_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.bool_func(expr, unta)
    }

    pub fn aba_shato_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.bool_func(expr, aba_shato)
    }

    pub fn unta_shato_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.bool_func(expr, unta_shato)
    }

    pub fn nye_shato_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.bool_func(expr, nye_shato)
    }

    pub fn shato_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        truth_expr(equal_expr(&expr.lawa(), &expr.kucha().lawa()))
    }

    pub fn celi_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.math_func(expr, celi)
    }

    pub fn movo_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.math_func(expr, movo)
    }

    pub fn celi_da_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.math_func(expr, celida)
    }

    pub fn movo_da_func(&mut self, _name: Rc<Token>, expr: Rc<Expr>) -> Rc<Expr> {
        self.math_func(expr, movoda)
    }
}

fn truth_expr(t: bool) -> Rc<Expr> {
    if t {
        Rc::new(Expr::Atom(Rc::new(make_token(TokenType::Const, "da"))))
    } else {
        Rc::new(Expr::Atom(Rc::new(make_token(TokenType::Const, "nye"))))
    }
}

impl Context {
    fn math_func(&self, expr: Rc<Expr>, f: fn(i64, i64) -> i64) -> Rc<Expr> {
        let a = self.get_number(&expr.lawa());
        let b = self.get_number(&expr.kucha().lawa());
        let result = Token {
            typ: TokenType::Number,
            num: f(a, b),
            text: "".to_string(),
        };
        Rc::new(Expr::Atom(Rc::new(result)))
    }

    fn bool_func(&self, expr: Rc<Expr>, f: fn(i64, i64) -> bool) -> Rc<Expr> {
        let a = self.get_number(&expr.lawa());
        let b = self.get_number(&expr.kucha().lawa());
        truth_expr(f(a, b))
    }

    pub fn get_number(&self, expr: &Expr) -> i64 {
        if expr.is_nya() {
            return 0;
        }
        if !expr.is_number() {
            errorf("expect number; got", &expr.to_string());
        }
        match expr {
            Expr::Atom(tok) => tok.num,
            _ => 0,
        }
    }
}

fn aba(a: i64, b: i64) -> bool { a < b }
fn unta(a: i64, b: i64) -> bool { a > b }
fn aba_shato(a: i64, b: i64) -> bool { a <= b }
fn unta_shato(a: i64, b: i64) -> bool { a >= b }
fn nye_shato(a: i64, b: i64) -> bool { a != b }

fn celi(a: i64, b: i64) -> i64 { a + b }
fn movo(a: i64, b: i64) -> i64 { a - b }
fn celida(a: i64, b: i64) -> i64 { a * b }
fn movoda(a: i64, b: i64) -> i64 {
    if b == 0 {
        errorf("div 0", "");
    }
    a / b
}

fn equal_expr(a: &Expr, b: &Expr) -> bool {
    if a.is_nya() && b.is_nya() {
        return true;
    }
    match (a, b) {
        (Expr::Nil, _) | (_, Expr::Nil) => false,
        (Expr::Atom(tok_a), Expr::Atom(tok_b)) => {
            if tok_a.typ != tok_b.typ {
                return false;
            }
            match tok_a.typ {
                TokenType::Number => tok_a.num == tok_b.num,
                _ => tok_a.text == tok_b.text,
            }
        }
        (Expr::Cons { lawa: la, kucha: ka }, Expr::Cons { lawa: lb, kucha: kb }) => {
            equal_expr(la, lb) && equal_expr(ka, kb)
        }
        _ => false,
    }
}
