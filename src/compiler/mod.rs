use std::collections::HashMap;
use std::rc::Rc;
use crate::expr::Expr;
use crate::vm::value::{Value, Closure, FuncObject};
use crate::vm::isa::*;

pub struct Compiler {
    func: FuncObject,
    locals: HashMap<String, u8>,
    next_reg: u8,
    label_counter: u32,
    pending_jumps: Vec<(usize, String)>,
    labels: HashMap<String, usize>,
    is_toplevel: bool,
}

impl Compiler {
    pub fn new(name: &str, is_toplevel: bool) -> Self {
        Compiler {
            func: FuncObject {
                name: name.to_string(),
                bytecode: Vec::new(),
                const_pool: Vec::new(),
                num_params: 0,
                num_locals: 0,
                used_regs: 0,
            },
            locals: HashMap::new(),
            next_reg: X10,
            label_counter: 0,
            pending_jumps: Vec::new(),
            labels: HashMap::new(),
            is_toplevel,
        }
    }

    pub fn compile_toplevel(exprs: &[Rc<Expr>]) -> (FuncObject, HashMap<String, Value>) {
        let mut globals = HashMap::new();
        let mut compiler = Compiler::new("__toplevel__", true);

        for expr in exprs {
            if let Expr::Cons { lawa, kucha } = expr.as_ref() {
                if let Some(tok) = lawa.get_sada() {
                    if tok.text == "muhe" {
                        Self::compile_muhe_into(&mut compiler, kucha.clone(), &mut globals);
                        continue;
                    }
                }
            }
            let result_reg = compiler.compile_expr(expr.clone(), true);
            compiler.emit_ret(result_reg);
        }

        if compiler.func.bytecode.is_empty() {
            compiler.emit_ret(X0);
        }
        compiler.resolve_labels();
        (compiler.func, globals)
    }

    fn compile_muhe_into(compiler: &mut Compiler, body: Rc<Expr>, globals: &mut HashMap<String, Value>) {
        let mut current = body.lawa();
        while !current.is_nil() {
            let def = current.lawa();
            let name_tok = def.lawa().get_sada().expect("muhe: expected symbol");
            let name = name_tok.text.clone();
            let lambda = def.kucha().lawa();

            let func_compiler = Compiler::compile_lambda(&name, lambda);
            let idx = compiler.func.const_pool.len();
            compiler.func.const_pool.push(Value::Closure(Rc::new(Closure {
                func: Rc::new(func_compiler),
                upvalues: Vec::new(),
            })));

            let target_reg = compiler.alloc_reg();
            compiler.emit_li(target_reg, idx as u32);
            compiler.locals.insert(name.clone(), target_reg);

            let name_idx = compiler.add_const(Value::Symbol(name.clone().into()));
            compiler.func.bytecode.push(encode_u(OP_SETGLOBAL, target_reg, name_idx as u32));

            globals.insert(name, Value::Nil);

            current = current.kucha();
        }
    }

    fn compile_lambda(name: &str, lambda: Rc<Expr>) -> FuncObject {
        let mut compiler = Compiler::new(name, false);

        let formals = lambda.kucha().lawa();
        let body = lambda.kucha().kucha().lawa();
        let num_params = if formals.is_atom() {
            let tok = formals.get_sada().expect("lambda: expected param");
            compiler.emit_mv(X18, X10);
            compiler.locals.insert(tok.text.clone(), X18);
            1
        } else {
            let mut count = 0;
            let mut f = formals;
            while !f.is_nil() {
                let tok = f.lawa().get_sada().expect("lambda: expected param");
                let local_reg = X18 + count;
                compiler.emit_mv(local_reg, X10 + count);
                compiler.locals.insert(tok.text.clone(), local_reg);
                count += 1;
                f = f.kucha();
            }
            count
        };
        compiler.func.num_params = num_params;
        compiler.next_reg = X18 + num_params;

        let result_reg = compiler.compile_expr(body, true);
        compiler.emit_ret(result_reg);
        compiler.resolve_labels();
        compiler.func.num_locals = compiler.locals.len() as u8;
        compiler.func.used_regs = compiler.next_reg;
        compiler.func
    }

    fn compile_expr(&mut self, expr: Rc<Expr>, is_tail: bool) -> u8 {
        match expr.as_ref() {
            Expr::Nil => {
                let reg = self.alloc_temp();
                self.emit_mv(reg, X0);
                reg
            }
            Expr::Atom(tok) => {
                match tok.typ {
                    crate::token::TokenType::Number => {
                        let reg = self.alloc_temp();
                        let idx = self.add_const(Value::Number(tok.num));
                        self.emit_li(reg, idx as u32);
                        reg
                    }
                    crate::token::TokenType::String => {
                        let reg = self.alloc_temp();
                        let idx = self.add_const(Value::String(tok.text.clone().into()));
                        self.emit_li(reg, idx as u32);
                        reg
                    }
                    _ => {
                        let text = tok.text.clone();
                        if let Some(&reg) = self.locals.get(&text) {
                            reg
                        } else {
                            let reg = self.alloc_temp();
                            let val = Self::predefined_value(&text)
                                .unwrap_or_else(|| Value::Symbol(text.into()));
                            let idx = self.add_const(val);
                            self.emit_li(reg, idx as u32);
                            reg
                        }
                    }
                }
            }
            Expr::Cons { lawa, kucha } => {
                if let Some(op_tok) = lawa.get_sada() {
                    match op_tok.text.as_str() {
                        "plata" => {
                            let quoted = kucha.lawa();
                            self.compile_expr(quoted, false)
                        }
                        "ka" => {
                            self.compile_if(kucha.clone(), is_tail)
                        }
                        "tido" => {
                            self.compile_let(kucha.clone(), is_tail)
                        }
                        "in" => {
                            self.compile_progn(kucha.clone(), is_tail)
                        }
                        "dala" => {
                            self.compile_cond(kucha.clone(), is_tail)
                        }
                        "plama" => {
                            self.compile_setq(kucha.clone(), is_tail)
                        }
                        "mita" => {
                            let reg = self.alloc_temp();
                            let idx = self.func.const_pool.len();
                            let func = Self::compile_lambda("lambda", expr);
                            self.func.const_pool.push(Value::Closure(Rc::new(Closure {
                                func: Rc::new(func),
                                upvalues: Vec::new(),
                            })));
                            self.emit_li(reg, idx as u32);
                            reg
                        }
                        _ => {
                            self.compile_call(&op_tok.text, kucha.clone(), is_tail)
                        }
                    }
                } else {
                    let mut argc = 0;
                    let mut current = kucha.clone();
                    let mut arg_regs = Vec::new();
                    while !current.is_nil() {
                        let arg_reg = self.compile_expr(current.lawa(), false);
                        arg_regs.push(arg_reg);
                        argc += 1;
                        current = current.kucha();
                    }

                    let func_reg = self.compile_expr(lawa.clone(), false);

                    let call_reg = if func_reg >= X10 && func_reg < X10 + argc as u8 {
                        self.emit_mv(X3, func_reg);
                        X3
                    } else {
                        func_reg
                    };

                    for (i, arg_reg) in arg_regs.iter().enumerate() {
                        if *arg_reg != X10 + i as u8 {
                            self.emit_mv(X10 + i as u8, *arg_reg);
                        }
                    }

                    let result_reg = self.alloc_temp();
                    if is_tail && !self.is_toplevel {
                        self.emit_tail(call_reg, argc as u8);
                    } else {
                        self.emit_call(result_reg, call_reg, argc as u8);
                    }
                    result_reg
                }
            }
        }
    }

    fn compile_if(&mut self, args: Rc<Expr>, is_tail: bool) -> u8 {
        let test_expr = args.lawa();
        let rest = args.kucha();
        let then_expr = rest.lawa();
        let else_expr = rest.kucha().lawa();

        let test_reg = self.compile_expr(test_expr, false);
        let else_label = self.new_label();

        self.emit_beq(test_reg, X0, &else_label);
        let then_reg = self.compile_expr(then_expr, is_tail);

        if is_tail && !self.is_toplevel {
            let then_has_ret = self.func.bytecode.last()
                .map(|i| {
                    let op = decode_op(*i);
                    op == OP_TAIL || op == OP_RET
                })
                .unwrap_or(false);
            if !then_has_ret {
                self.emit_ret(then_reg);
            }
            self.emit_label(&else_label);
            let else_reg = self.compile_expr(else_expr, is_tail);
            let else_has_ret = self.func.bytecode.last()
                .map(|i| {
                    let op = decode_op(*i);
                    op == OP_TAIL || op == OP_RET
                })
                .unwrap_or(false);
            if !else_has_ret {
                self.emit_ret(else_reg);
            }
            return then_reg;
        }

        let end_label = self.new_label();
        self.emit_j(&end_label);
        self.emit_label(&else_label);
        let else_reg = self.compile_expr(else_expr, is_tail);
        if then_reg != else_reg {
            self.emit_mv(then_reg, else_reg);
        }
        self.emit_label(&end_label);
        then_reg
    }

    fn compile_let(&mut self, args: Rc<Expr>, is_tail: bool) -> u8 {
        let bindings = args.lawa();
        let body = args.kucha().lawa();

        let mut saved = Vec::new();
        let mut current = bindings;
        while !current.is_nil() {
            let binding = current.lawa();
            let var_tok = binding.lawa().get_sada().expect("let: expected var");
            let val_expr = binding.kucha().lawa();

            let val_reg = self.compile_expr(val_expr, false);
            let local_reg = self.alloc_local();
            self.emit_mv(local_reg, val_reg);
            saved.push((var_tok.text.clone(), local_reg));
            self.locals.insert(var_tok.text.clone(), local_reg);

            current = current.kucha();
        }

        let result = self.compile_expr(body, is_tail);

        for (name, _) in saved {
            self.locals.remove(&name);
        }

        result
    }

    fn compile_cond(&mut self, args: Rc<Expr>, is_tail: bool) -> u8 {
        let mut current = args;
        let end_label = self.new_label();
        let result_reg = self.alloc_temp();
        self.emit_mv(result_reg, X0);

        while !current.is_nil() {
            let clause = current.lawa();
            let remaining = current.kucha();

            if clause.is_atom() {
                let reg = self.compile_expr(clause, is_tail);
                if reg != result_reg {
                    self.emit_mv(result_reg, reg);
                }
                if is_tail && !self.is_toplevel {
                    let has_ret = self.func.bytecode.last()
                        .map(|i| {
                            let op = decode_op(*i);
                            op == OP_TAIL || op == OP_RET
                        })
                        .unwrap_or(false);
                    if !has_ret {
                        self.emit_ret(result_reg);
                    }
                }
                break;
            }

            let test_expr = clause.lawa();
            let rest = clause.kucha();

            let test_reg = self.compile_expr(test_expr, false);
            let next_label = self.new_label();

            self.emit_beq(test_reg, X0, &next_label);

            let then_reg = if rest.is_nil() {
                self.compile_expr(clause, is_tail)
            } else {
                self.compile_expr(rest.lawa(), is_tail)
            };
            if then_reg != result_reg {
                self.emit_mv(result_reg, then_reg);
            }

            if is_tail && !self.is_toplevel {
                let has_ret = self.func.bytecode.last()
                    .map(|i| {
                        let op = decode_op(*i);
                        op == OP_TAIL || op == OP_RET
                    })
                    .unwrap_or(false);
                if !has_ret {
                    self.emit_ret(result_reg);
                }
            } else {
                self.emit_j(&end_label);
            }

            self.emit_label(&next_label);
            current = remaining;
        }

        if !(is_tail && !self.is_toplevel) {
            self.emit_label(&end_label);
        }

        result_reg
    }

    fn compile_setq(&mut self, args: Rc<Expr>, _is_tail: bool) -> u8 {
        let mut current = args;
        let mut result = X0;

        while !current.is_nil() {
            let var = current.lawa();
            current = current.kucha();
            if current.is_nil() {
                panic!("plama: odd number of args");
            }
            let val = self.compile_expr(current.lawa(), false);
            current = current.kucha();

            let var_tok = var.get_sada().expect("plama: no tiga param");
            if let Some(&reg) = self.locals.get(&var_tok.text) {
                self.emit_mv(reg, val);
                result = reg;
            } else {
                panic!("plama: undefined variable {}", var_tok.text);
            }
        }

        result
    }

    fn compile_progn(&mut self, args: Rc<Expr>, is_tail: bool) -> u8 {
        let mut result = X0;
        let mut current = args;
        while !current.is_nil() {
            let is_last = current.kucha().is_nil();
            result = self.compile_expr(current.lawa(), is_tail && is_last);
            current = current.kucha();
        }
        result
    }

    fn builtin_index(name: &str) -> Option<u16> {
        match name {
            "celi" => Some(0),
            "movo" => Some(1),
            "celida" => Some(2),
            "movoda" => Some(3),
            "aba" => Some(4),
            "unta" => Some(5),
            "abashato" => Some(6),
            "untashato" => Some(7),
            "shato" => Some(8),
            "nyeshato" => Some(9),
            "lawa" => Some(10),
            "kucha" => Some(11),
            "upa" => Some(12),
            "mite" => Some(13),
            _ => None,
        }
    }

    fn compile_call(&mut self, name: &str, args: Rc<Expr>, is_tail: bool) -> u8 {
        let mut argc = 0;
        let mut current = args;
        let mut arg_regs = Vec::new();
        while !current.is_nil() {
            let arg_reg = self.compile_expr(current.lawa(), false);
            arg_regs.push(arg_reg);
            argc += 1;
            current = current.kucha();
        }

        let func_reg = if let Some(&reg) = self.locals.get(name) {
            reg
        } else if let Some(builtin_idx) = Self::builtin_index(name) {
            let reg = self.alloc_temp();
            let idx = self.add_const(Value::Builtin(builtin_idx));
            self.emit_li(reg, idx as u32);
            reg
        } else {
            let reg = self.alloc_temp();
            let idx = self.add_const(Value::Symbol(name.into()));
            self.emit_li(reg, idx as u32);
            reg
        };

        let call_reg = if func_reg >= X10 && func_reg < X10 + argc as u8 {
            self.emit_mv(X3, func_reg);
            X3
        } else {
            func_reg
        };

        for (i, arg_reg) in arg_regs.iter().enumerate() {
            if *arg_reg != X10 + i as u8 {
                self.emit_mv(X10 + i as u8, *arg_reg);
            }
        }

        let result_reg = self.alloc_temp();
        if is_tail && !self.is_toplevel {
            self.emit_tail(call_reg, argc as u8);
        } else {
            self.emit_call(result_reg, call_reg, argc as u8);
        }
        result_reg
    }

    fn predefined_value(name: &str) -> Option<Value> {
        match name {
            "da" => Some(Value::Bool(true)),
            "nye" => Some(Value::Bool(false)),
            "nya" => Some(Value::Nil),
            "unu" => Some(Value::Number(1)),
            "du" => Some(Value::Number(2)),
            "unudu" => Some(Value::Number(3)),
            "dudu" => Some(Value::Number(4)),
            "mani" => Some(Value::Number(5)),
            _ => None,
        }
    }

    fn alloc_reg(&mut self) -> u8 {
        let reg = self.next_reg;
        self.next_reg += 1;
        if self.next_reg > self.func.used_regs {
            self.func.used_regs = self.next_reg;
        }
        reg
    }

    fn alloc_temp(&mut self) -> u8 {
        self.alloc_reg()
    }

    fn alloc_local(&mut self) -> u8 {
        self.alloc_reg()
    }

    fn add_const(&mut self, val: Value) -> usize {
        let idx = self.func.const_pool.len();
        self.func.const_pool.push(val);
        idx
    }

    fn new_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn emit_label(&mut self, label: &str) {
        self.labels.insert(label.to_string(), self.func.bytecode.len());
    }

    fn emit_mv(&mut self, rd: u8, rs1: u8) {
        self.func.bytecode.push(encode_r(OP_MV, rd, rs1, X0, 0));
    }

    fn emit_li(&mut self, rd: u8, idx: u32) {
        self.func.bytecode.push(encode_u(OP_LI, rd, idx));
    }

    fn emit_ret(&mut self, rs1: u8) {
        self.func.bytecode.push(encode_r(OP_RET, X0, rs1, X0, 0));
    }

    fn emit_call(&mut self, rd: u8, rs1: u8, argc: u8) {
        self.func.bytecode.push(encode_r(OP_CALL, rd, rs1, X0, argc));
    }

    fn emit_tail(&mut self, rs1: u8, argc: u8) {
        self.func.bytecode.push(encode_r(OP_TAIL, X0, rs1, X0, argc));
    }

    fn emit_beq(&mut self, rs1: u8, rs2: u8, label: &str) {
        let pc = self.func.bytecode.len();
        self.func.bytecode.push(encode_b(OP_BEQ, rs1, rs2, 0));
        self.pending_jumps.push((pc, label.to_string()));
    }

    fn emit_j(&mut self, label: &str) {
        let pc = self.func.bytecode.len();
        self.func.bytecode.push(encode_j(OP_J, X0, 0));
        self.pending_jumps.push((pc, label.to_string()));
    }

    fn resolve_labels(&mut self) {
        for (pc, label) in &self.pending_jumps {
            let target = *self.labels.get(label).expect("undefined label") as i64;
            let offset = target - (*pc as i64 + 1);
            let inst = self.func.bytecode[*pc];
            let op = decode_op(inst);
            let resolved = match op {
                OP_BEQ | OP_BNE | OP_BLT | OP_BGE => {
                    let rs1 = decode_b_rs1(inst);
                    let rs2 = decode_b_rs2(inst);
                    encode_b(op, rs1, rs2, offset as i16)
                }
                OP_JAL | OP_J => {
                    let rd = decode_rd(inst);
                    encode_j(op, rd, offset as i32)
                }
                _ => panic!("unknown jump instruction"),
            };
            self.func.bytecode[*pc] = resolved;
        }
    }
}
