use std::env;
use std::io::{self, BufRead, Write};

fn main() {
    let mut print_sexpr = false;
    let mut do_prompt = true;
    let mut prompt_str = "> ".to_string();
    let mut files = Vec::new();
    let mut script_args = Vec::new();
    let mut pass_args = false;

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if pass_args {
            script_args.push(args[i].clone());
        } else {
            match args[i].as_str() {
                "-sexpr" => print_sexpr = true,
                "-doprompt" => {
                    i += 1;
                    if i < args.len() {
                        do_prompt = args[i].parse().unwrap_or(true);
                    }
                }
                "-prompt" => {
                    i += 1;
                    if i < args.len() {
                        prompt_str = args[i].clone();
                    }
                }
                "--" => pass_args = true,
                file => files.push(file.to_string()),
            }
        }
        i += 1;
    }

    mita::config(print_sexpr);

    let mut vm = mita::vm::VM::new();
    mita::vm::builtins::register_all(&mut vm);
    vm.script_args = script_args;

    for file in &files {
        load(&mut vm, file);
    }

    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line.unwrap();
        if do_prompt {
            print!("{}", prompt_str);
            io::stdout().flush().unwrap();
        }
        if line.trim().is_empty() {
            continue;
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut parser = mita::Parser::new(&line);
            let mut exprs = Vec::new();
            loop {
                let c = parser.skip_space();
                if c == '\0' {
                    break;
                }
                exprs.push(parser.list());
            }
            let (func, globals) = mita::compiler::Compiler::compile_toplevel(&exprs);
            for (name, val) in globals {
                vm.define_global(&name, val);
            }
            let closure = std::rc::Rc::new(mita::vm::value::Closure {
                func: std::rc::Rc::new(func),
                upvalues: Vec::new(),
            });
            let result = vm.run(closure);
            println!("{}", result);
        }));
        if let Err(e) = result {
            if let Some(err) = e.downcast_ref::<mita::Error>() {
                eprintln!("{}", err.0);
            } else if e.downcast_ref::<mita::EOF>().is_some() {
                break;
            } else {
                panic_any(e);
            }
        }
    }
}

fn load(vm: &mut mita::vm::VM, file: &str) {
    let content = std::fs::read_to_string(file).expect("Failed to read file");
    let mut parser = mita::Parser::new(&content);
    let mut exprs = Vec::new();
    loop {
        match parser.skip_space() {
            '\n' => continue,
            '\0' => break,
            _ => {}
        }
        exprs.push(parser.list());
    }
    let (func, globals) = mita::compiler::Compiler::compile_toplevel(&exprs);
    for (name, val) in globals {
        vm.define_global(&name, val);
    }
    let closure = std::rc::Rc::new(mita::vm::value::Closure {
        func: std::rc::Rc::new(func),
        upvalues: Vec::new(),
    });
    let result = vm.run(closure);
    println!("{}", result);
}

fn panic_any<T: std::any::Any + Send>(x: T) -> ! {
    std::panic::panic_any(x);
}
