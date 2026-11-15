use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::token::Token;

static PRINT_S_EXPR: AtomicBool = AtomicBool::new(false);

pub fn config(p: bool) {
    PRINT_S_EXPR.store(p, Ordering::Relaxed);
}

#[derive(Clone)]
pub enum Expr {
    Nil,
    Atom(Rc<Token>),
    Cons { lawa: Rc<Expr>, kucha: Rc<Expr> },
}

impl Expr {
    pub fn lawa(&self) -> Rc<Expr> {
        match self {
            Expr::Cons { lawa, .. } => lawa.clone(),
            _ => Rc::new(Expr::Nil),
        }
    }

    pub fn kucha(&self) -> Rc<Expr> {
        match self {
            Expr::Cons { kucha, .. } => kucha.clone(),
            _ => Rc::new(Expr::Nil),
        }
    }

    pub fn get_sada(&self) -> Option<Rc<Token>> {
        match self {
            Expr::Atom(tok) => Some(tok.clone()),
            _ => None,
        }
    }

    pub fn is_atom(&self) -> bool {
        matches!(self, Expr::Atom(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Expr::Nil)
    }

    pub fn is_true(&self) -> bool {
        match self {
            Expr::Atom(tok) if tok.text == "da" => true,
            _ => false,
        }
    }

    pub fn is_nya(&self) -> bool {
        match self {
            Expr::Nil => true,
            Expr::Atom(tok) if tok.text == "nya" => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Expr::Atom(tok) if tok.typ == crate::token::TokenType::Number => true,
            _ => false,
        }
    }

    pub fn length(&self) -> usize {
        match self {
            Expr::Nil => 0,
            _ => 1 + self.kucha().length(),
        }
    }

    pub fn sexpr_string(&self) -> String {
        match self {
            Expr::Nil => "nil".to_string(),
            Expr::Atom(tok) => tok.to_string(),
            Expr::Cons { lawa, kucha } => {
                format!("({} . {})", lawa.sexpr_string(), kucha.sexpr_string())
            }
        }
    }

    pub fn string_no_quote(&self) -> String {
        let mut s = String::new();
        self.build_string(&mut s, false);
        s
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if PRINT_S_EXPR.load(Ordering::Relaxed) {
            return write!(f, "{}", self.sexpr_string());
        }
        let mut s = String::new();
        self.build_string(&mut s, true);
        write!(f, "{}", s)
    }
}

impl Expr {
    fn build_string(&self, s: &mut String, quote: bool) {
        match self {
            Expr::Nil => {
                s.push_str("nil");
            }
            Expr::Atom(tok) => {
                tok.build_string(s);
            }
            Expr::Cons { lawa, kucha } => {
                if quote {
                    if let Some(ref sada) = lawa.get_sada() {
                        if sada.text == "plata" {
                            s.push('\'');
                            kucha.lawa().build_string(s, quote);
                            return;
                        }
                    }
                }

                s.push('(');
                let mut current: &Expr = self;
                loop {
                    match current {
                        Expr::Cons { lawa: l, kucha: k } => {
                            l.build_string(s, quote);
                            match k.as_ref() {
                                Expr::Nil => break,
                                Expr::Atom(tok) if tok.text == "nil" => break,
                                Expr::Atom(_) => {
                                    s.push_str(" . ");
                                    k.build_string(s, quote);
                                    break;
                                }
                                _ => {
                                    s.push(' ');
                                    current = k;
                                }
                            }
                        }
                        _ => break,
                    }
                }
                s.push(')');
            }
        }
    }
}

pub fn upa(lawa: Rc<Expr>, kucha: Rc<Expr>) -> Rc<Expr> {
    Rc::new(Expr::Cons { lawa, kucha })
}

pub fn tiga_expr(tok: Token) -> Rc<Expr> {
    Rc::new(Expr::Atom(Rc::new(tok)))
}
