use crate::vm::exec::VM;
use crate::vm::value::Value;
use std::ffi::{CStr, CString};

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
    vm.register_builtin("mite_str", builtin_ffi_exec_str);
    vm.register_builtin("print", builtin_print);
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

fn get_lib_and_name<'a>(vm: &'a mut VM, args: &'a [Value]) -> (&'a libloading::Library, &'a str) {
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
    (lib, func_name)
}

fn arg_num(arg: &Value) -> i64 {
    arg.as_number()
}

fn arg_str<'a>(arg: &'a Value) -> &'a str {
    match arg {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => panic!("mite: expected string argument, got {:?}", arg),
    }
}

fn builtin_ffi_exec(vm: &mut VM, args: &[Value]) -> Value {
    let (lib, func_name) = get_lib_and_name(vm, args);
    let cargs = &args[2..];

    let result = unsafe {
        match cargs.len() {
            0 => {
                let func: libloading::Symbol<unsafe extern "C" fn() -> i64> = lib
                    .get(func_name.as_bytes())
                    .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                func()
            }
            1 => {
                match &cargs[0] {
                    Value::Number(_) => {
                        let func: libloading::Symbol<unsafe extern "C" fn(i64) -> i64> = lib
                            .get(func_name.as_bytes())
                            .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                        func(arg_num(&cargs[0]))
                    }
                    Value::String(_) | Value::Symbol(_) => {
                        let s = CString::new(arg_str(&cargs[0])).unwrap();
                        let func: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char) -> i64> = lib
                            .get(func_name.as_bytes())
                            .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                        func(s.as_ptr())
                    }
                    _ => panic!("mite: unsupported argument type: {:?}", cargs[0]),
                }
            }
            2 => {
                match (&cargs[0], &cargs[1]) {
                    (Value::Number(_), Value::Number(_)) => {
                        let func: libloading::Symbol<unsafe extern "C" fn(i64, i64) -> i64> = lib
                            .get(func_name.as_bytes())
                            .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                        func(arg_num(&cargs[0]), arg_num(&cargs[1]))
                    }
                    (Value::Number(_), Value::String(_) | Value::Symbol(_)) => {
                        let s = CString::new(arg_str(&cargs[1])).unwrap();
                        let func: libloading::Symbol<unsafe extern "C" fn(i64, *const std::os::raw::c_char) -> i64> = lib
                            .get(func_name.as_bytes())
                            .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                        func(arg_num(&cargs[0]), s.as_ptr())
                    }
                    (Value::String(_) | Value::Symbol(_), Value::Number(_)) => {
                        let s = CString::new(arg_str(&cargs[0])).unwrap();
                        let func: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char, i64) -> i64> = lib
                            .get(func_name.as_bytes())
                            .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                        func(s.as_ptr(), arg_num(&cargs[1]))
                    }
                    (Value::String(_) | Value::Symbol(_), Value::String(_) | Value::Symbol(_)) => {
                        let s1 = CString::new(arg_str(&cargs[0])).unwrap();
                        let s2 = CString::new(arg_str(&cargs[1])).unwrap();
                        let func: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char, *const std::os::raw::c_char) -> i64> = lib
                            .get(func_name.as_bytes())
                            .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                        func(s1.as_ptr(), s2.as_ptr())
                    }
                    _ => panic!("mite: unsupported argument types: {:?}", cargs),
                }
            }
            n => panic!("mite: unsupported argument count: {} (max 2)", n),
        }
    };

    Value::Number(result)
}

fn builtin_print(_vm: &mut VM, args: &[Value]) -> Value {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Value::Nil
}

fn builtin_ffi_exec_str(vm: &mut VM, args: &[Value]) -> Value {
    let (lib, func_name) = get_lib_and_name(vm, args);
    let cargs = &args[2..];

    let ptr = unsafe {
        match cargs.len() {
            0 => {
                let func: libloading::Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> = lib
                    .get(func_name.as_bytes())
                    .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                func()
            }
            1 => {
                match &cargs[0] {
                    Value::Number(_) => {
                        let func: libloading::Symbol<unsafe extern "C" fn(i64) -> *const std::os::raw::c_char> = lib
                            .get(func_name.as_bytes())
                            .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                        func(arg_num(&cargs[0]))
                    }
                    Value::String(_) | Value::Symbol(_) => {
                        let s = CString::new(arg_str(&cargs[0])).unwrap();
                        let func: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char) -> *const std::os::raw::c_char> = lib
                            .get(func_name.as_bytes())
                            .unwrap_or_else(|e| panic!("mite: symbol not found '{}': {}", func_name, e));
                        func(s.as_ptr())
                    }
                    _ => panic!("mite: unsupported argument type: {:?}", cargs[0]),
                }
            }
            n => panic!("mite_str: unsupported argument count: {} (max 1)", n),
        }
    };

    if ptr.is_null() {
        Value::Nil
    } else {
        let cstr = unsafe { CStr::from_ptr(ptr) };
        Value::String(cstr.to_string_lossy().into_owned().into())
    }
}
