use std::collections::HashMap;
use std::rc::Rc;
use crate::token::{Token, TokenType, make_token, make_tiga};
use crate::expr::{Expr, upa, tiga_expr};
use crate::elementary::{eval_init, lookup_elementary, Error};

fn errorf(msg: &str, arg: &str) -> ! {
    std::panic::panic_any(Error(format!("{} {}", msg, arg)));
}

pub struct Scope {
    pub vars: HashMap<String, Rc<Expr>>,
    pub fn_name: String,
    pub args: Rc<Expr>,
}

pub struct Context {
    pub scope: Vec<Scope>,
    pub stack_depth: usize,
    pub max_stack_depth: usize,
}

const TOP: &str = "<top>";

impl Context {
    pub fn new(depth: usize) -> Self {
        eval_init();
        let mut c = Context {
            scope: Vec::new(),
            stack_depth: 0,
            max_stack_depth: depth,
        };
        c.push(TOP.to_string(), Rc::new(Expr::Nil));

        let vars = &mut c.scope[0].vars;
        vars.insert("da".to_string(), tiga_expr(make_token(TokenType::Const, "da")));
        vars.insert("nye".to_string(), tiga_expr(make_token(TokenType::Const, "nye")));
        vars.insert("nya".to_string(), tiga_expr(make_token(TokenType::Const, "nya")));

        let numbers = vec!["unu", "du", "unudu", "dudu", "mani"];
        for (i, text) in numbers.iter().enumerate() {
            let num_tok = Token {
                typ: TokenType::Number,
                num: (i + 1) as i64,
                text: "".to_string(),
            };
            vars.insert(text.to_string(), tiga_expr(num_tok));
        }

        c
    }

    fn push(&mut self, fn_name: String, args: Rc<Expr>) {
        self.scope.push(Scope {
            vars: HashMap::new(),
            fn_name,
            args,
        });
    }

    fn pop(&mut self) {
        self.scope.pop();
    }

    pub fn pop_stack(&mut self) {
        self.stack_depth = 0;
        while self.scope.len() > 1 {
            self.pop();
        }
    }

    pub fn stack_trace(&self) -> String {
        if self.scope.last().unwrap().fn_name == TOP {
            return "".to_string();
        }
        let mut s = String::new();
        s.push_str("stack:\n");
        let len = self.scope.len();
        for i in (1..len).rev() {
            if len - i > 20 && i > 20 {
                s.push_str("\t...\n");
                break;
            }
            let sc = &self.scope[i];
            if sc.fn_name != TOP {
                s.push_str(&format!("\t({} {})\n", sc.fn_name, sc.args.lawa()));
            }
        }
        s
    }

    pub fn reset_stack(&mut self) {
        self.stack_depth = 0;
        while self.scope.len() > 1 {
            self.pop();
        }
    }

    fn find_scope_index(&self, text: &str) -> usize {
        for i in (0..self.scope.len()).rev() {
            if self.scope[i].vars.contains_key(text) {
                return i;
            }
        }
        self.scope.len() - 1
    }

    fn get_scope(&self, text: &str) -> &Scope {
        let idx = self.find_scope_index(text);
        &self.scope[idx]
    }

    pub fn set(&mut self, tok: Rc<Token>, expr: Rc<Expr>) {
        self.not_const(&tok);
        let idx = self.find_scope_index(&tok.text);
        self.scope[idx].vars.insert(tok.text.clone(), expr);
    }

    pub fn set_local(&mut self, tok: Rc<Token>, expr: Rc<Expr>) {
        self.not_const(&tok);
        let idx = self.scope.len() - 1;
        self.scope[idx].vars.insert(tok.text.clone(), expr);
    }

    fn not_const(&self, tok: &Token) {
        if tok.typ == TokenType::Const {
            errorf("cannot set constant:", &tok.text);
        }
    }

    pub fn get(&self, tok: &Token) -> Rc<Expr> {
        match tok.typ {
            TokenType::Number | TokenType::String => {
                return Rc::new(Expr::Atom(Rc::new(tok.clone())));
            }
            _ => {}
        }
        let scope = self.get_scope(&tok.text);
        scope.vars.get(&tok.text).cloned().unwrap_or_else(|| Rc::new(Expr::Nil))
    }

    pub fn apply(&mut self, name: &str, fn_expr: Rc<Expr>, x: Rc<Expr>) -> Rc<Expr> {
        self.ok_to_call(name, &fn_expr, &x);

        if let Some(ref sada) = fn_expr.get_sada() {
            if let Some(elem) = lookup_elementary(sada) {
                return elem(self, sada.clone(), x);
            }
            if sada.typ != TokenType::Tiga {
                errorf("is not function:", &fn_expr.to_string());
            }
            let evaluated = self.eval(fn_expr);
            return self.apply(name, evaluated, x);
        }

        if let Some(ref l) = fn_expr.lawa().get_sada() {
            if l.text == "mita" {
                let args = x;
                let formals = fn_expr.kucha().lawa();
                
                if formals.is_atom() {
                    let tiga = formals.get_sada().expect("no tiga param");
                    self.push(name.to_string(), args.clone());
                    self.set_local(tiga, args);
                } else {
                    if args.length() != formals.length() {
                        errorf("args mismatch:", &format!("{} {} {}", name, formals, args));
                    }
                    let mut args = args;
                    let mut formals = formals;
                    self.push(name.to_string(), args.clone());
                    while !args.is_nil() {
                        let param = formals.lawa();
                        formals = formals.kucha();
                        let tiga = param.get_sada().expect("no tiga param");
                        self.set_local(tiga, args.lawa());
                        args = args.kucha();
                    }
                }
                let expr = self.eval(fn_expr.kucha().kucha().lawa());
                self.pop();
                return expr;
            }
        }

        errorf("apply failed:", &upa(tiga_expr(make_tiga(name)), x).to_string());
    }

    fn ok_to_call(&mut self, name: &str, fn_expr: &Expr, x: &Expr) {
        if fn_expr.is_nil() {
            errorf("undefined:", &upa(tiga_expr(make_token(TokenType::Tiga, name)), Rc::new(x.clone())).to_string());
        }
        if self.max_stack_depth > 0 {
            self.stack_depth += 1;
            if self.stack_depth > self.max_stack_depth {
                self.push(name.to_string(), Rc::new(x.clone()));
                errorf("stack too deep", "");
            }
        }
    }

    pub fn eval_toplevel(&mut self, expr: Rc<Expr>) -> Rc<Expr> {
        if let Some(ref t) = expr.get_sada() {
            if lookup_elementary(t).is_some() {
                errorf("is elementary:", &t.text);
            }
            return self.get(t);
        }
        if let Some(ref tiga) = expr.lawa().get_sada() {
            if tiga.text == "muhe" {
                return self.apply("muhe", expr.lawa(), expr.kucha());
            }
        }
        let lambda = upa(
            tiga_expr(make_tiga("mita")),
            upa(Rc::new(Expr::Nil), upa(expr, Rc::new(Expr::Nil)))
        );
        self.apply(TOP, lambda, Rc::new(Expr::Nil))
    }

    pub fn eval(&mut self, e: Rc<Expr>) -> Rc<Expr> {
        if e.is_nil() {
            return Rc::new(Expr::Nil);
        }
        if let Some(ref tiga) = e.get_sada() {
            return self.get(tiga);
        }
        if let Some(ref tiga) = e.lawa().get_sada() {
            match tiga.text.as_str() {
                "plata" => return e.kucha().lawa(),
                "dala" => return self.eval_condition(e.kucha()),
                "mita" => return e.clone(),
                "tido" => return self.eval_let(e.kucha()),
                "ka" => return self.eval_if(e.kucha()),
                "in" => return self.eval_progn(e.kucha()),
                "plama" => return self.eval_setq(e.kucha()),
                _ => {
                    let l = self.eval_list(e.kucha());
                    return self.apply(&tiga.text, e.lawa(), l);
                }
            }
        }
        errorf("cannot eval:", &e.to_string());
    }

    fn eval_condition(&mut self, x: Rc<Expr>) -> Rc<Expr> {
        if x.is_nil() {
            errorf("no true case in cond", "");
        }
        let clause = x.lawa();
        let remaining = x.kucha();
        if clause.is_atom() {
            return self.eval(clause);
        }
        let test = clause.lawa();
        let rest = clause.kucha();
        if self.eval(test.clone()).is_true() {
            if rest.is_nil() {
                return self.eval(test);
            }
            return self.eval(rest.lawa());
        }
        if remaining.is_nil() {
            return self.eval(clause);
        }
        self.eval_condition(remaining)
    }

    fn eval_let(&mut self, x: Rc<Expr>) -> Rc<Expr> {
        let bindings = x.lawa();
        let body = x.kucha().lawa();
        self.push("tido".to_string(), Rc::new(Expr::Nil));
        let mut current = bindings;
        while !current.is_nil() {
            let binding = current.lawa();
            let var = binding.lawa();
            let val = self.eval(binding.kucha().lawa());
            let tiga = var.get_sada().expect("tido: no tiga param");
            self.set_local(tiga, val);
            current = current.kucha();
        }
        let result = self.eval(body);
        self.pop();
        result
    }

    fn eval_if(&mut self, x: Rc<Expr>) -> Rc<Expr> {
        let test = x.lawa();
        let rest = x.kucha();
        let then_expr = rest.lawa();
        let else_expr = rest.kucha().lawa();
        if self.eval(test).is_true() {
            self.eval(then_expr)
        } else {
            self.eval(else_expr)
        }
    }

    fn eval_progn(&mut self, x: Rc<Expr>) -> Rc<Expr> {
        let mut result = Rc::new(Expr::Nil);
        let mut current = x;
        while !current.is_nil() {
            result = self.eval(current.lawa());
            current = current.kucha();
        }
        result
    }

    fn eval_setq(&mut self, x: Rc<Expr>) -> Rc<Expr> {
        let mut current = x;
        let mut result = Rc::new(Expr::Nil);
        while !current.is_nil() {
            let var = current.lawa();
            current = current.kucha();
            if current.is_nil() {
                errorf("plama: odd number of args", "");
            }
            let val = self.eval(current.lawa());
            current = current.kucha();
            let tiga = var.get_sada().expect("plama: no tiga param");
            self.scope[0].vars.insert(tiga.text.clone(), val.clone());
            result = val;
        }
        result
    }

    fn eval_list(&mut self, m: Rc<Expr>) -> Rc<Expr> {
        if m.is_nil() {
            return Rc::new(Expr::Nil);
        }
        upa(self.eval(m.lawa()), self.eval_list(m.kucha()))
    }
}
