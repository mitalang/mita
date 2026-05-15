use crate::vm::exec::VM;
use crate::vm::value::Value;

pub fn register_all(vm: &mut VM) {
    vm.register_builtin("celi", builtin_add);
    vm.register_builtin("movo", builtin_sub);
    vm.register_builtin("celida", builtin_mul);
    vm.register_builtin("movoda", builtin_div);
    vm.register_builtin("aba", builtin_lt);
    vm.register_builtin("unta", builtin_gt);
    vm.register_builtin("abashato", builtin_le);
    vm.register_builtin("untashato", builtin_ge);
    vm.register_builtin("shato", builtin_eq);
    vm.register_builtin("nyeshato", builtin_ne);
    vm.register_builtin("lawa", builtin_car);
    vm.register_builtin("kucha", builtin_cdr);
    vm.register_builtin("upa", builtin_cons);
    vm.register_builtin("mite", builtin_ffi_exec);
}

fn builtin_add(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Number(args[0].as_number() + args[1].as_number())
}

fn builtin_sub(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Number(args[0].as_number() - args[1].as_number())
}

fn builtin_mul(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Number(args[0].as_number() * args[1].as_number())
}

fn builtin_div(_vm: &mut VM, args: &[Value]) -> Value {
    let b = args[1].as_number();
    if b == 0 {
        panic!("div 0");
    }
    Value::Number(args[0].as_number() / b)
}

fn builtin_lt(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Bool(args[0].as_number() < args[1].as_number())
}

fn builtin_gt(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Bool(args[0].as_number() > args[1].as_number())
}

fn builtin_le(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Bool(args[0].as_number() <= args[1].as_number())
}

fn builtin_ge(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Bool(args[0].as_number() >= args[1].as_number())
}

fn builtin_eq(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Bool(values_eq(&args[0], &args[1]))
}

fn builtin_ne(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Bool(!values_eq(&args[0], &args[1]))
}

fn values_eq(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Nil, Nil) | (Nil, Bool(false)) | (Bool(false), Nil) => true,
        (Number(a), Number(b)) => a == b,
        (Bool(a), Bool(b)) => a == b,
        (String(a), String(b)) => a == b,
        (Symbol(a), Symbol(b)) => a == b,
        (Cons(a), Cons(b)) => values_eq(&a.0, &b.0) && values_eq(&a.1, &b.1),
        _ => false,
    }
}

fn builtin_car(_vm: &mut VM, args: &[Value]) -> Value {
    match &args[0] {
        Value::Cons(pair) => pair.0.clone(),
        Value::Nil => Value::Nil,
        other => panic!("car: expected cons, got {:?}", other),
    }
}

fn builtin_cdr(_vm: &mut VM, args: &[Value]) -> Value {
    match &args[0] {
        Value::Cons(pair) => pair.1.clone(),
        Value::Nil => Value::Nil,
        other => panic!("cdr: expected cons, got {:?}", other),
    }
}

fn builtin_cons(_vm: &mut VM, args: &[Value]) -> Value {
    Value::Cons(std::rc::Rc::new((args[0].clone(), args[1].clone())))
}

fn builtin_ffi_exec(vm: &mut VM, args: &[Value]) -> Value {
    if args.len() < 2 {
        panic!("mite: expected at least library path and function name");
    }
    let lib_path = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => panic!("mite: expected string library path, got {:?}", args[0]),
    };
    let func_name = match &args[1] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => panic!("mite: expected string function name, got {:?}", args[1]),
    };

    let lib = vm.get_library(lib_path)
        .unwrap_or_else(|| panic!("mite: library not found: {}", lib_path));

    let result = unsafe {
        match args.len() - 2 {
            0 => {
                let func: libloading::Symbol<unsafe extern "C" fn() -> i64> = lib
                    .get(func_name.as_bytes())
                    .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                func()
            }
            1 => {
                let func: libloading::Symbol<unsafe extern "C" fn(i64) -> i64> = lib
                    .get(func_name.as_bytes())
                    .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                func(args[2].as_number())
            }
            2 => {
                let func: libloading::Symbol<unsafe extern "C" fn(i64, i64) -> i64> = lib
                    .get(func_name.as_bytes())
                    .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                func(args[2].as_number(), args[3].as_number())
            }
            n => panic!("mite: unsupported argument count: {} (max 2)", n),
        }
    };

    Value::Number(result)
}
