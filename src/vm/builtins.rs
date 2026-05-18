use crate::vm::exec::{VM, Socket};
use crate::vm::value::Value;
use std::ffi::{CStr, CString};
use std::rc::Rc;

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
    vm.register_builtin("sys_listen", builtin_sys_listen);
    vm.register_builtin("sys_accept", builtin_sys_accept);
    vm.register_builtin("sys_read", builtin_sys_read);
    vm.register_builtin("sys_write", builtin_sys_write);
    vm.register_builtin("sys_close", builtin_sys_close);
    vm.register_builtin("sys_http_parse_path", builtin_http_parse_path);
    vm.register_builtin("sys_http_ok", builtin_http_ok);
    vm.register_builtin("sys_http_not_found", builtin_http_not_found);
    vm.register_builtin("sys_http_parse_request", builtin_http_parse_request);
    vm.register_builtin("sys_http_response", builtin_http_response);
    vm.register_builtin("sys_url_decode", builtin_url_decode);
    vm.register_builtin("sys_read_http", builtin_sys_read_http);
    vm.register_builtin("sys_args", builtin_sys_args);
    vm.register_builtin("sys_string_find", builtin_string_find);
    vm.register_builtin("sys_string_length", builtin_string_length);
    vm.register_builtin("sys_string_substr", builtin_string_substr);
    vm.register_builtin("sys_string_concat", builtin_string_concat);
    vm.register_builtin("sys_string_split", builtin_string_split);
    vm.register_builtin("sys_string_replace", builtin_string_replace);
    vm.register_builtin("sys_string_trim", builtin_string_trim);
}

fn builtin_string_find(_vm: &mut VM, args: &[Value]) -> Value {
    let haystack = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Number(-1),
    };
    let needle = match &args[1] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Number(-1),
    };
    match haystack.find(needle) {
        Some(i) => Value::Number(i as i64),
        None => Value::Number(-1),
    }
}

fn builtin_string_length(_vm: &mut VM, args: &[Value]) -> Value {
    let s = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Number(0),
    };
    Value::Number(s.len() as i64)
}

fn builtin_string_substr(_vm: &mut VM, args: &[Value]) -> Value {
    let s = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };
    let start = args[1].as_number() as usize;
    if start >= s.len() {
        return Value::String("".into());
    }
    let end = args.get(2).map(|v| v.as_number() as usize).unwrap_or(s.len());
    let end = end.min(s.len());
    if start >= end {
        return Value::String("".into());
    }
    Value::String(s[start..end].into())
}

fn builtin_string_concat(_vm: &mut VM, args: &[Value]) -> Value {
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::String(s) | Value::Symbol(s) => result.push_str(s),
            Value::Number(n) => result.push_str(&n.to_string()),
            _ => {}
        }
    }
    Value::String(result.into())
}

fn builtin_string_split(_vm: &mut VM, args: &[Value]) -> Value {
    let s = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };
    let delimiter = match &args[1] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };
    let parts: Vec<&str> = s.split(delimiter).collect();
    let mut list = Value::Nil;
    for part in parts.iter().rev() {
        list = Value::Cons(Rc::new((Value::String((*part).into()), list)));
    }
    list
}

fn builtin_string_replace(_vm: &mut VM, args: &[Value]) -> Value {
    let s = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };
    let from = match &args[1] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };
    let to = match &args[2] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };
    Value::String(s.replace(from, to).into())
}

fn builtin_string_trim(_vm: &mut VM, args: &[Value]) -> Value {
    let s = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };
    Value::String(s.trim().into())
}

fn builtin_sys_args(vm: &mut VM, _args: &[Value]) -> Value {
    let mut list = Value::Nil;
    for arg in vm.script_args.iter().rev() {
        list = Value::Cons(Rc::new((Value::String(arg.clone().into()), list)));
    }
    list
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

fn builtin_sys_listen(vm: &mut VM, args: &[Value]) -> Value {
    let host = match args.get(1) {
        Some(Value::String(s) | Value::Symbol(s)) => s.as_ref(),
        _ => "0.0.0.0",
    };
    let port = match &args[0] {
        Value::Number(n) => *n as u16,
        Value::String(s) | Value::Symbol(s) => s.parse::<u16>().unwrap_or_else(|_| panic!("sys_listen: invalid port '{}'", s)),
        _ => panic!("sys_listen: expected port number or string, got {:?}", args[0]),
    };
    let listener = std::net::TcpListener::bind((host, port))
        .unwrap_or_else(|e| panic!("sys_listen: failed to bind {}:{}: {}", host, port, e));
    let fd = vm.alloc_fd();
    vm.sockets.insert(fd, Socket::Listener(listener));
    Value::Number(fd)
}

fn builtin_sys_accept(vm: &mut VM, args: &[Value]) -> Value {
    let fd = args[0].as_number();
    let socket = vm.sockets.remove(&fd)
        .unwrap_or_else(|| panic!("sys_accept: invalid fd {}", fd));
    match socket {
        Socket::Listener(listener) => {
            let (stream, _) = listener.accept()
                .unwrap_or_else(|e| panic!("sys_accept: failed to accept: {}", e));
            vm.sockets.insert(fd, Socket::Listener(listener));
            let client_fd = vm.alloc_fd();
            vm.sockets.insert(client_fd, Socket::Stream(stream));
            Value::Number(client_fd)
        }
        _ => panic!("sys_accept: fd {} is not a listener", fd),
    }
}

fn builtin_sys_read(vm: &mut VM, args: &[Value]) -> Value {
    let fd = args[0].as_number();
    let socket = vm.sockets.remove(&fd)
        .unwrap_or_else(|| panic!("sys_read: invalid fd {}", fd));
    match socket {
        Socket::Stream(mut stream) => {
            use std::io::Read;
            let mut buf = [0u8; 65536];
            let result = match stream.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    Value::String(s.into_owned().into())
                }
                _ => Value::Nil,
            };
            vm.sockets.insert(fd, Socket::Stream(stream));
            result
        }
        _ => panic!("sys_read: fd {} is not a stream", fd),
    }
}

fn builtin_sys_write(vm: &mut VM, args: &[Value]) -> Value {
    let fd = args[0].as_number();
    let data = match &args[1] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => panic!("sys_write: expected string data, got {:?}", args[1]),
    };
    let socket = vm.sockets.remove(&fd)
        .unwrap_or_else(|| panic!("sys_write: invalid fd {}", fd));
    match socket {
        Socket::Stream(mut stream) => {
            use std::io::Write;
            let result = match stream.write_all(data.as_bytes()) {
                Ok(()) => {
                    let _ = stream.flush();
                    Value::Number(data.len() as i64)
                }
                Err(_) => Value::Number(-1),
            };
            vm.sockets.insert(fd, Socket::Stream(stream));
            result
        }
        _ => panic!("sys_write: fd {} is not a stream", fd),
    }
}

fn builtin_sys_close(vm: &mut VM, args: &[Value]) -> Value {
    let fd = args[0].as_number();
    vm.sockets.remove(&fd);
    Value::Nil
}

fn builtin_http_parse_path(_vm: &mut VM, args: &[Value]) -> Value {
    let request = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::String("/".into()),
    };
    let path = if request.starts_with("GET ") || request.starts_with("POST ") {
        let start = request.find(' ').map(|i| i + 1).unwrap_or(0);
        let end = request[start..].find(' ').map(|i| i + start).unwrap_or(request.len());
        &request[start..end]
    } else {
        "/"
    };
    Value::String(path.into())
}

fn builtin_http_ok(_vm: &mut VM, args: &[Value]) -> Value {
    let body = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => "",
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    Value::String(response.into())
}

fn builtin_http_not_found(_vm: &mut VM, args: &[Value]) -> Value {
    let body = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => "",
    };
    let response = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    Value::String(response.into())
}

fn builtin_http_parse_request(_vm: &mut VM, args: &[Value]) -> Value {
    let raw = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };

    let normalized = raw.replace("\r\n", "\n");
    let (head, body) = match normalized.find("\n\n") {
        Some(i) => (&normalized[..i], &normalized[i + 2..]),
        None => (normalized.as_str(), ""),
    };

    let lines: Vec<&str> = head.split('\n').collect();
    if lines.is_empty() {
        return Value::Nil;
    }

    let first = lines[0];
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let raw_path = parts.next().unwrap_or("/");
    let version = parts.next().unwrap_or("HTTP/1.1");

    let (path, query) = match raw_path.find('?') {
        Some(i) => (&raw_path[..i], &raw_path[i + 1..]),
        None => (raw_path, ""),
    };

    let mut headers = Value::Nil;
    for line in lines.iter().skip(1).rev() {
        if let Some(i) = line.find(':') {
            let key = line[..i].trim();
            let val = line[i + 1..].trim();
            headers = Value::Cons(Rc::new((
                Value::Cons(Rc::new((
                    Value::String(key.into()),
                    Value::String(val.into()),
                ))),
                headers,
            )));
        }
    }

    let mut result = Value::Nil;
    result = Value::Cons(Rc::new((Value::String(body.into()), result)));
    result = Value::Cons(Rc::new((headers, result)));
    result = Value::Cons(Rc::new((Value::String(version.into()), result)));
    result = Value::Cons(Rc::new((Value::String(query.into()), result)));
    result = Value::Cons(Rc::new((Value::String(path.into()), result)));
    result = Value::Cons(Rc::new((Value::String(method.into()), result)));
    result
}

fn builtin_http_response(_vm: &mut VM, args: &[Value]) -> Value {
    let status = match &args[0] {
        Value::Number(n) => *n as u16,
        Value::String(s) | Value::Symbol(s) => s.parse::<u16>().unwrap_or(200),
        _ => 200,
    };
    let body = match args.get(1) {
        Some(Value::String(s) | Value::Symbol(s)) => s.as_ref(),
        _ => "",
    };
    let content_type = match args.get(2) {
        Some(Value::String(s) | Value::Symbol(s)) => s.as_ref(),
        _ => "text/html; charset=utf-8",
    };

    let status_text = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    };

    let date = {
        let now = std::time::SystemTime::now();
        let dur = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        let secs = dur.as_secs() as i64;
        // Simple GMT date formatting for HTTP
        let days = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        // Days since Unix epoch
        let day_count = (secs / 86400) as usize;
        let day_of_week = days[(day_count + 4) % 7]; // Jan 1 1970 was Thursday
        let year = 1970 + (day_count / 365) as i64;
        let leap_years = (year - 1969) / 4 - (year - 1901) / 100 + (year - 1601) / 400;
        let day_of_year = (day_count as i64 - (year - 1970) * 365 - leap_years) as u64;
        let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut month = 0;
        let mut day = day_of_year + 1;
        while month < 12 && day > month_days[month] as u64 {
            day -= month_days[month] as u64;
            month += 1;
        }
        let hour = (secs % 86400) / 3600;
        let minute = (secs % 3600) / 60;
        let second = secs % 60;
        format!(
            "Date: {}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
            day_of_week, day, months[month], year, hour, minute, second
        )
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\n{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        status_text,
        date,
        content_type,
        body.len(),
        body
    );
    Value::String(response.into())
}

fn builtin_url_decode(_vm: &mut VM, args: &[Value]) -> Value {
    let input = match &args[0] {
        Value::String(s) | Value::Symbol(s) => s.as_ref(),
        _ => return Value::Nil,
    };

    let mut result = String::new();
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(a), Some(b)) = (h1, h2) {
                if let Ok(byte) = u8::from_str_radix(&format!("{}{}", a, b), 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push(c);
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    Value::String(result.into())
}

fn builtin_sys_read_http(vm: &mut VM, args: &[Value]) -> Value {
    let fd = args[0].as_number();
    let socket = vm.sockets.remove(&fd)
        .unwrap_or_else(|| panic!("sys_read_http: invalid fd {}", fd));
    match socket {
        Socket::Stream(mut stream) => {
            use std::io::Read;
            let mut buffer = Vec::new();
            let mut temp = [0u8; 4096];
            loop {
                match stream.read(&mut temp) {
                    Ok(0) => break,
                    Ok(n) => {
                        buffer.extend_from_slice(&temp[..n]);
                        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let content_length = {
                let buf_str = String::from_utf8_lossy(&buffer);
                if let Some(end) = buf_str.find("\r\n\r\n") {
                    let headers = &buf_str[..end];
                    headers.lines()
                        .find_map(|line| {
                            let line = line.to_lowercase();
                            if line.starts_with("content-length:") {
                                line[15..].trim().parse::<usize>().ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0)
                } else {
                    0
                }
            };
            let header_end = buffer.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4).unwrap_or(buffer.len());
            let body_received = buffer.len() - header_end;
            if content_length > body_received {
                let mut remaining = content_length - body_received;
                while remaining > 0 {
                    match stream.read(&mut temp) {
                        Ok(0) => break,
                        Ok(n) => {
                            buffer.extend_from_slice(&temp[..n]);
                            remaining -= n;
                        }
                        Err(_) => break,
                    }
                }
            }

            let result = if buffer.is_empty() {
                Value::Nil
            } else {
                let s = String::from_utf8_lossy(&buffer);
                Value::String(s.into_owned().into())
            };

            vm.sockets.insert(fd, Socket::Stream(stream));
            result
        }
        _ => panic!("sys_read_http: fd {} is not a stream", fd),
    }
}
