use std::env;
use std::io::{self, BufRead, Write};

fn main() {
    let mut print_sexpr = false;
    let mut do_prompt = true;
    let mut prompt_str = "> ".to_string();
    let mut stack_depth = 100_000;
    let mut files = Vec::new();

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
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
            "-depth" => {
                i += 1;
                if i < args.len() {
                    stack_depth = args[i].parse().unwrap_or(100_000);
                }
            }
            file => files.push(file.to_string()),
        }
        i += 1;
    }

    mita::config(print_sexpr);
    let mut context = mita::Context::new(stack_depth as usize);

    for file in &files {
        load(&mut context, file);
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
        let mut parser = mita::Parser::new(&line);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let expr = context.eval_toplevel(parser.list());
            println!("{}", expr);
        }));
        if let Err(e) = result {
            if let Some(err) = e.downcast_ref::<mita::Error>() {
                eprintln!("{}", err.0);
                context.pop_stack();
            } else if e.downcast_ref::<mita::EOF>().is_some() {
                break;
            } else {
                panic_any(e);
            }
        }
    }
}

fn load(context: &mut mita::Context, file: &str) {
    let content = std::fs::read_to_string(file).expect("Failed to read file");
    let mut parser = mita::Parser::new(&content);
    input(context, &mut parser, "");
}

fn input(context: &mut mita::Context, parser: &mut mita::Parser, _prompt: &str) {
    loop {
        match parser.skip_space() {
            '\n' => continue,
            '\0' => return,
            _ => {}
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let expr = context.eval_toplevel(parser.list());
            println!("{}", expr);
        }));
        match result {
            Ok(_) => {}
            Err(e) => {
                if let Some(err) = e.downcast_ref::<mita::Error>() {
                    eprintln!("{}", err.0);
                    context.pop_stack();
                    parser.skip_to_end_of_line();
                } else if e.downcast_ref::<mita::EOF>().is_some() {
                    std::process::exit(0);
                } else {
                    panic_any(e);
                }
            }
        }
        parser.skip_space();
    }
}

fn panic_any<T: std::any::Any + Send>(x: T) -> ! {
    std::panic::panic_any(x);
}
