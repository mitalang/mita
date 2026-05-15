use std::collections::HashMap;
use std::rc::Rc;
use crate::vm::value::{Value, Closure};
use crate::vm::isa::*;

pub struct VM {
    regs: [Value; 32],
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    builtins: Vec<fn(&mut VM, &[Value]) -> Value>,
}

struct CallFrame {
    closure: Rc<Closure>,
    return_pc: usize,
    saved_regs: Vec<Value>,
    result_reg: u8,
}

impl VM {
    pub fn new() -> Self {
        VM {
            regs: std::array::from_fn(|_| Value::Nil),
            stack: Vec::new(),
            frames: Vec::new(),
            globals: HashMap::new(),
            builtins: Vec::new(),
        }
    }

    pub fn register_builtin(&mut self, name: &str, f: fn(&mut VM, &[Value]) -> Value) {
        let idx = self.builtins.len() as u16;
        self.builtins.push(f);
        self.globals.insert(name.to_string(), Value::Builtin(idx));
    }

    pub fn define_global(&mut self, name: &str, val: Value) {
        self.globals.insert(name.to_string(), val);
    }

    pub fn run(&mut self, closure: Rc<Closure>) -> Value {
        self.frames.push(CallFrame {
            closure: closure.clone(),
            return_pc: 0,
            saved_regs: Vec::new(),
            result_reg: X10,
        });

        let mut pc: usize = 0;

        loop {
            let inst = self.frame().closure.func.bytecode[pc];
            pc += 1;

            match decode_op(inst) {
                OP_NOP => {},
                OP_MV => {
                    let val = self.reg(decode_rs1(inst)).clone();
                    self.set_reg(decode_rd(inst), val);
                }
                OP_LI => {
                    let idx = decode_imm18(inst) as usize;
                    let val = self.frame().closure.func.const_pool[idx].clone();
                    self.set_reg(decode_rd(inst), val);
                }
                OP_ADD => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    self.set_reg(decode_rd(inst), Value::Number(a + b));
                }
                OP_SUB => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    self.set_reg(decode_rd(inst), Value::Number(a - b));
                }
                OP_MUL => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    self.set_reg(decode_rd(inst), Value::Number(a * b));
                }
                OP_DIV => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    if b == 0 {
                        panic!("div 0");
                    }
                    self.set_reg(decode_rd(inst), Value::Number(a / b));
                }
                OP_REM => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    if b == 0 {
                        panic!("div 0");
                    }
                    self.set_reg(decode_rd(inst), Value::Number(a % b));
                }
                OP_ADDI => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = decode_imm12(inst) as i64;
                    self.set_reg(decode_rd(inst), Value::Number(a + b));
                }
                OP_SEQ => {
                    let a = self.reg(decode_rs1(inst));
                    let b = self.reg(decode_rs2(inst));
                    self.set_reg(decode_rd(inst), Value::Bool(values_equal(a, b)));
                }
                OP_SNE => {
                    let a = self.reg(decode_rs1(inst));
                    let b = self.reg(decode_rs2(inst));
                    self.set_reg(decode_rd(inst), Value::Bool(!values_equal(a, b)));
                }
                OP_SLT => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    self.set_reg(decode_rd(inst), Value::Bool(a < b));
                }
                OP_SGT => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    self.set_reg(decode_rd(inst), Value::Bool(a > b));
                }
                OP_SLE => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    self.set_reg(decode_rd(inst), Value::Bool(a <= b));
                }
                OP_SGE => {
                    let a = self.reg(decode_rs1(inst)).as_number();
                    let b = self.reg(decode_rs2(inst)).as_number();
                    self.set_reg(decode_rd(inst), Value::Bool(a >= b));
                }
                OP_CONS => {
                    let car = self.reg(decode_rs1(inst)).clone();
                    let cdr = self.reg(decode_rs2(inst)).clone();
                    self.set_reg(decode_rd(inst), Value::Cons(Rc::new((car, cdr))));
                }
                OP_CAR => {
                    let val = self.reg(decode_rs1(inst));
                    let car = match val {
                        Value::Cons(pair) => pair.0.clone(),
                        Value::Nil => Value::Nil,
                        _ => panic!("car: expected cons, got {:?}", val),
                    };
                    self.set_reg(decode_rd(inst), car);
                }
                OP_CDR => {
                    let val = self.reg(decode_rs1(inst));
                    let cdr = match val {
                        Value::Cons(pair) => pair.1.clone(),
                        Value::Nil => Value::Nil,
                        _ => panic!("cdr: expected cons, got {:?}", val),
                    };
                    self.set_reg(decode_rd(inst), cdr);
                }
                OP_LW => {
                    let base = decode_rs1(inst);
                    let offset = decode_imm12(inst) as i64;
                    let idx = self.reg(base).as_number() + offset;
                    let val = self.stack[idx as usize].clone();
                    self.set_reg(decode_rd(inst), val);
                }
                OP_SW => {
                    let base = decode_rs1(inst);
                    let offset = decode_imm12(inst) as i64;
                    let idx = self.reg(base).as_number() + offset;
                    let val = self.reg(decode_rs2(inst)).clone();
                    let i = idx as usize;
                    if i >= self.stack.len() {
                        self.stack.resize(i + 1, Value::Nil);
                    }
                    self.stack[i] = val;
                }
                OP_BEQ => {
                    let a = self.reg(decode_b_rs1(inst));
                    let b = self.reg(decode_b_rs2(inst));
                    if values_equal(a, b) {
                        pc = (pc as i64 + decode_imm12(inst) as i64) as usize;
                    }
                }
                OP_BNE => {
                    let a = self.reg(decode_b_rs1(inst));
                    let b = self.reg(decode_b_rs2(inst));
                    if !values_equal(a, b) {
                        pc = (pc as i64 + decode_imm12(inst) as i64) as usize;
                    }
                }
                OP_BLT => {
                    let a = self.reg(decode_b_rs1(inst)).as_number();
                    let b = self.reg(decode_b_rs2(inst)).as_number();
                    if a < b {
                        pc = (pc as i64 + decode_imm12(inst) as i64) as usize;
                    }
                }
                OP_BGE => {
                    let a = self.reg(decode_b_rs1(inst)).as_number();
                    let b = self.reg(decode_b_rs2(inst)).as_number();
                    if a >= b {
                        pc = (pc as i64 + decode_imm12(inst) as i64) as usize;
                    }
                }
                OP_JAL => {
                    let ret_addr = pc;
                    pc = (pc as i64 + decode_imm18(inst) as i64) as usize;
                    self.set_reg(decode_rd(inst), Value::Number(ret_addr as i64));
                }
                OP_J => {
                    pc = (pc as i64 + decode_imm18(inst) as i64) as usize;
                }
                OP_CALL => {
                    let rd = decode_rd(inst);
                    let func_reg = decode_rs1(inst);
                    let argc = decode_funct3(inst) as usize;
                    let func_val = self.reg(func_reg).clone();
                    match func_val {
                        Value::Closure(closure) => {
                            self.enter_closure(closure, pc, rd);
                            pc = 0;
                        }
                        Value::Builtin(idx) => {
                            let mut args = Vec::with_capacity(argc);
                            for i in 0..argc {
                                args.push(self.regs[X10 as usize + i].clone());
                            }
                            let result = (self.builtins[idx as usize])(self, &args);
                            self.set_reg(rd, result);
                        }
                        Value::Symbol(name) => {
                            if let Some(global) = self.globals.get(&*name).cloned() {
                                match global {
                                    Value::Closure(closure) => {
                                        self.enter_closure(closure, pc, rd);
                                        pc = 0;
                                    }
                                    Value::Builtin(idx) => {
                                        let mut args = Vec::with_capacity(argc);
                                        for i in 0..argc {
                                            args.push(self.regs[X10 as usize + i].clone());
                                        }
                                        let result = (self.builtins[idx as usize])(self, &args);
                                        self.set_reg(rd, result);
                                    }
                                    _ => panic!("call: global is not a function: {:?}", global),
                                }
                            } else {
                                panic!("call: undefined function: {}", name);
                            }
                        }
                        _ => panic!("call: expected function, got {:?}", func_val),
                    }
                }
                OP_TAIL => {
                    let func_reg = decode_rs1(inst);
                    let argc = decode_funct3(inst) as usize;
                    if let Some(val) = self.tail_call(func_reg, argc) {
                        return val;
                    }
                    pc = 0;
                }
                OP_RET => {
                    let val = self.reg(decode_rs1(inst)).clone();
                    if self.frames.len() == 1 {
                        self.frames.pop();
                        return val;
                    }
                    let frame = self.frames.pop().unwrap();
                    pc = frame.return_pc;
                    for (idx, saved) in frame.saved_regs.iter().enumerate() {
                        self.regs[8 + idx] = saved.clone();
                    }
                    self.set_reg(frame.result_reg, val);
                }
                OP_BUILTIN => {
                    let rd = decode_rd(inst);
                    let idx = decode_imm18(inst) as usize;
                    let argc = self.reg(X10).as_number() as usize;
                    let mut args = Vec::with_capacity(argc);
                    for i in 0..argc {
                        args.push(self.regs[X10 as usize + i].clone());
                    }
                    let result = (self.builtins[idx])(self, &args);
                    self.set_reg(rd, result);
                }
                OP_SETGLOBAL => {
                    let name_idx = decode_imm18(inst) as usize;
                    let name = match &self.frame().closure.func.const_pool[name_idx] {
                        Value::String(s) | Value::Symbol(s) => s.to_string(),
                        _ => panic!("setglobal: expected symbol"),
                    };
                    let val = self.reg(decode_rd(inst)).clone();
                    self.globals.insert(name, val);
                }
                _ => panic!("unknown opcode: 0x{:02X} at pc={}", decode_op(inst), pc - 1),
            }
        }
    }

    fn frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn reg(&self, idx: u8) -> &Value {
        &self.regs[idx as usize]
    }

    fn set_reg(&mut self, idx: u8, val: Value) {
        if idx != X0 {
            self.regs[idx as usize] = val;
        }
    }

    fn enter_closure(&mut self, closure: Rc<Closure>, return_pc: usize, result_reg: u8) {
        let used_regs = closure.func.used_regs;
        let num_s_regs = used_regs.saturating_sub(8).min(20) as usize;
        let mut saved = Vec::with_capacity(num_s_regs);
        for i in 0..num_s_regs {
            saved.push(self.regs[8 + i].clone());
        }

        self.frames.push(CallFrame {
            closure,
            return_pc,
            saved_regs: saved,
            result_reg,
        });

        for i in (used_regs as usize)..32 {
            self.regs[i] = Value::Nil;
        }
    }

    fn tail_call(&mut self, func_reg: u8, argc: usize) -> Option<Value> {
        let func_val = self.reg(func_reg).clone();
        let current_frame = self.frames.pop().unwrap();

        match func_val {
            Value::Closure(closure) => {
                self.enter_closure(closure, current_frame.return_pc, current_frame.result_reg);
                None
            }
            Value::Builtin(idx) => {
                let mut args = Vec::with_capacity(argc);
                for i in 0..argc {
                    args.push(self.regs[X10 as usize + i].clone());
                }
                let result = (self.builtins[idx as usize])(self, &args);

                for (idx, saved) in current_frame.saved_regs.iter().enumerate() {
                    self.regs[8 + idx] = saved.clone();
                }

                self.set_reg(X10, result);

                if self.frames.is_empty() {
                    return Some(self.reg(X10).clone());
                }
                let frame = self.frames.pop().unwrap();
                for (idx, saved) in frame.saved_regs.iter().enumerate() {
                    self.regs[8 + idx] = saved.clone();
                }
                if self.frames.is_empty() {
                    Some(self.reg(X10).clone())
                } else {
                    None
                }
            }
            Value::Symbol(name) => {
                if let Some(global) = self.globals.get(&*name).cloned() {
                    match global {
                        Value::Closure(closure) => {
                            self.enter_closure(closure, current_frame.return_pc, current_frame.result_reg);
                            None
                        }
                        Value::Builtin(idx) => {
                            let mut args = Vec::with_capacity(argc);
                            for i in 0..argc {
                                args.push(self.regs[X10 as usize + i].clone());
                            }
                            let result = (self.builtins[idx as usize])(self, &args);
                            self.set_reg(X10, result);

                            for (idx, saved) in current_frame.saved_regs.iter().enumerate() {
                                self.regs[8 + idx] = saved.clone();
                            }

                            if self.frames.is_empty() {
                                return Some(self.reg(X10).clone());
                            }
                            let frame = self.frames.pop().unwrap();
                            for (idx, saved) in frame.saved_regs.iter().enumerate() {
                                self.regs[8 + idx] = saved.clone();
                            }
                            if self.frames.is_empty() {
                                Some(self.reg(X10).clone())
                            } else {
                                None
                            }
                        }
                        _ => panic!("tail call: global is not a function: {:?}", global),
                    }
                } else {
                    panic!("tail call: undefined function: {}", name);
                }
            }
            _ => panic!("tail call: expected function, got {:?}", func_val),
        }
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Nil, Nil) | (Nil, Bool(false)) | (Bool(false), Nil) => true,
        (Number(a), Number(b)) => a == b,
        (Bool(a), Bool(b)) => a == b,
        (String(a), String(b)) => a == b,
        (Symbol(a), Symbol(b)) => a == b,
        (Cons(a), Cons(b)) => values_equal(&a.0, &b.0) && values_equal(&a.1, &b.1),
        _ => false,
    }
}
