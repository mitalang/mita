use std::rc::Rc;

use mita::compiler::Compiler;
use mita::vm::value::Closure;
use mita::vm::VM;
use mita::Parser;

fn parse_source(source: &str) -> Vec<Rc<mita::Expr>> {
    let mut parser = Parser::new(source);
    let mut exprs = Vec::new();
    loop {
        match parser.skip_space() {
            '\n' => continue,
            '\0' => break,
            _ => {}
        }
        exprs.push(parser.list());
    }
    exprs
}

fn load_file(vm: &mut VM, path: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    let exprs = parse_source(&source);
    let (func, globals) = Compiler::compile_toplevel(&exprs);
    for (name, val) in globals {
        vm.define_global(&name, val);
    }
    vm.run(Rc::new(Closure {
        func: Rc::new(func),
        upvalues: Vec::new(),
    }));
}

fn eval(vm: &mut VM, source: &str) -> String {
    let exprs = parse_source(source);
    let (func, _) = Compiler::compile_toplevel(&exprs);
    let result = vm.run(Rc::new(Closure {
        func: Rc::new(func),
        upvalues: Vec::new(),
    }));
    format!("{}", result)
}

fn new_vm() -> VM {
    let mut vm = VM::new();
    mita::vm::builtins::register_all(&mut vm);
    vm
}

#[test]
fn odomu_excludes_network_functions() {
    let source = std::fs::read_to_string("odomu.mita").unwrap();
    for name in [
        "net_listen",
        "net_accept",
        "net_read",
        "net_write",
        "net_close",
        "serve_once",
    ] {
        assert!(
            !source.contains(&format!("({}", name)),
            "odomu.mita still defines {}",
            name
        );
    }
}

#[test]
fn net_library_loads_after_odomu() {
    let mut vm = new_vm();
    load_file(&mut vm, "odomu.mita");
    load_file(&mut vm, "net.mita");
    assert_eq!(eval(&mut vm, "(net_close 999)"), "nil");
}

#[test]
fn libraries_load_together() {
    let mut vm = new_vm();
    load_file(&mut vm, "odomu.mita");
    load_file(&mut vm, "net.mita");
    load_file(&mut vm, "http.mita");
    assert_eq!(eval(&mut vm, "(net_close 999)"), "nil");
}
