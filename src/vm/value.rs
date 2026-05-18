use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub enum Value {
    Nil,
    Number(i64),
    Bool(bool),
    String(Rc<str>),
    Symbol(Rc<str>),
    Cons(Rc<(Value, Value)>),
    Closure(Rc<Closure>),
    Builtin(u16),
}

#[derive(Clone)]
pub struct Closure {
    pub func: Rc<FuncObject>,
    pub upvalues: Vec<Rc<RefCell<Value>>>,
}

#[derive(Clone)]
pub struct FuncObject {
    pub name: String,
    pub bytecode: Vec<u32>,
    pub const_pool: Vec<Value>,
    pub num_params: u8,
    pub num_locals: u8,
    pub used_regs: u8,
    pub variadic: bool,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn as_number(&self) -> i64 {
        match self {
            Value::Number(n) => *n,
            Value::Nil => 0,
            _ => panic!("expect number, got {:?}", self),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Number(_) => "number",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::Symbol(_) => "symbol",
            Value::Cons(_) => "cons",
            Value::Closure(_) => "closure",
            Value::Builtin(_) => "builtin",
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", if *b { "da" } else { "nye" }),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Cons(pair) => {
                write!(f, "(")?;
                fmt_cons(&pair.0, &pair.1, f)?;
                write!(f, ")")
            }
            Value::Closure(c) => write!(f, "<closure {}:u003e", c.func.name),
            Value::Builtin(idx) => write!(f, "<builtin {}:u003e", idx),
        }
    }
}

fn fmt_cons(car: &Value, cdr: &Value, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", car)?;
    match cdr {
        Value::Nil => {},
        Value::Cons(pair) => {
            write!(f, " ")?;
            fmt_cons(&pair.0, &pair.1, f)?;
        }
        other => write!(f, " . {:?}", other)?,
    }
    Ok(())
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
