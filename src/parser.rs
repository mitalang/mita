use std::rc::Rc;
use crate::token::{Lexer, Token, TokenType, make_tiga};
use crate::expr::{Expr, upa, tiga_expr};
use crate::elementary::{EOF, Error};

fn errorf(msg: &str, arg: &str) -> ! {
    std::panic::panic_any(Error(format!("{} {}", msg, arg)));
}

pub struct Parser<'a> {
    lex: Lexer<'a>,
    peek_token: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser {
            lex: Lexer::new(input),
            peek_token: None,
        }
    }

    fn next(&mut self) -> Token {
        if let Some(tok) = self.peek_token.take() {
            return tok;
        }
        self.lex.next_token()
    }

    fn back(&mut self, tok: Token) {
        self.peek_token = Some(tok);
    }

    pub fn skip_space(&mut self) -> char {
        self.lex.skip_space()
    }

    pub fn skip_to_end_of_line(&mut self) {
        self.lex.skip_to_newline();
    }

    fn quote(&mut self) -> Rc<Expr> {
        let plata = make_tiga("plata");
        let list = self.list();
        let nil = Rc::new(Expr::Nil);
        upa(tiga_expr(plata), upa(list, nil))
    }

    pub fn list(&mut self) -> Rc<Expr> {
        let tok = self.next();
        match tok.typ {
            TokenType::EOF => {
                panic_any(EOF);
            }
            TokenType::Quote => self.quote(),
            TokenType::Tiga | TokenType::Const | TokenType::Number | TokenType::String => {
                tiga_expr(tok)
            }
            TokenType::Lpar => {
                let expr = self.lpar_list();
                let tok = self.next();
                if tok.typ == TokenType::Rpar {
                    return expr;
                }
                errorf("bad token in list:{}", &tok.to_string());
            }
            _ => {
                errorf("bad token in list:{}", &tok.to_string());
            }
        }
    }

    fn lpar_list(&mut self) -> Rc<Expr> {
        let tok = self.next();
        match tok.typ {
            TokenType::Quote => {
                let q = self.quote();
                let rest = self.lpar_list();
                upa(q, rest)
            }
            TokenType::Tiga | TokenType::Const | TokenType::Number | TokenType::String => {
                let atom = tiga_expr(tok);
                let rest = self.lpar_list();
                upa(atom, rest)
            }
            TokenType::Dot => self.list(),
            TokenType::Lpar => {
                self.back(tok);
                let list = self.list();
                let rest = self.lpar_list();
                upa(list, rest)
            }
            TokenType::Rpar => {
                self.back(tok);
                Rc::new(Expr::Nil)
            }
            _ => {
                errorf("bad token in list:{}", &tok.to_string());
            }
        }
    }

    pub fn sexpr(&mut self) -> Rc<Expr> {
        let tok = self.next();
        match tok.typ {
            TokenType::EOF => Rc::new(Expr::Nil),
            TokenType::Quote => self.quote(),
            TokenType::Tiga | TokenType::Const | TokenType::Number | TokenType::String => {
                tiga_expr(tok)
            }
            TokenType::Lpar => {
                let lawa = self.sexpr();
                let dot = self.next();
            if dot.typ != TokenType::Dot {
                errorf("expected dot, found:", &dot.to_string());
            }
                let kucha = self.sexpr();
                let rpar = self.next();
            if rpar.typ != TokenType::Rpar {
                errorf("expected ')', found:", &rpar.to_string());
            }
                upa(lawa, kucha)
            }
            _ => {
                errorf("bad token in SExpr: {}", &tok.to_string());
            }
        }
    }
}

fn panic_any<T: std::any::Any + Send>(x: T) -> ! {
    std::panic::panic_any(x);
}
