use mita::Parser;
use mita::vm::VM;
use mita::compiler::Compiler;

fn eval_vm(code: &str) -> String {
    let mut parser = Parser::new(code);
    let mut exprs = Vec::new();
    loop {
        let c = parser.skip_space();
        if c == '\0' {
            break;
        }
        exprs.push(parser.list());
    }
    let (func, _globals) = Compiler::compile_toplevel(&exprs);
    let closure = std::rc::Rc::new(mita::vm::value::Closure {
        func: std::rc::Rc::new(func),
        upvalues: Vec::new(),
    });
    let mut vm = VM::new();
    mita::vm::builtins::register_all(&mut vm);
    let result = vm.run(closure);
    format!("{}", result)
}

#[test]
fn test_vm_add() {
    assert_eq!(eval_vm("(celi 1 2)"), "3");
}

#[test]
fn test_vm_sub() {
    assert_eq!(eval_vm("(movo 5 3)"), "2");
}

#[test]
fn test_vm_mul() {
    assert_eq!(eval_vm("(celida 4 5)"), "20");
}

#[test]
fn test_vm_div() {
    assert_eq!(eval_vm("(movoda 10 2)"), "5");
}

#[test]
fn test_vm_cmp() {
    assert_eq!(eval_vm("(aba 2 3)"), "da");
    assert_eq!(eval_vm("(unta 2 3)"), "nye");
}

#[test]
fn test_vm_if() {
    assert_eq!(eval_vm("(ka da 1 2)"), "1");
    assert_eq!(eval_vm("(ka nye 1 2)"), "2");
}

#[test]
fn test_vm_let() {
    assert_eq!(eval_vm("(tido ((x 1) (y 2)) (celi x y))"), "3");
}

#[test]
fn test_vm_lambda() {
    assert_eq!(eval_vm("((mita (x) (celi x 1)) 5)"), "6");
}

#[test]
fn test_vm_tail_call() {
    let code = "(muhe ((fak_tco (mita (n acc) (ka (shato n unu) acc (fak_tco (movo n unu) (celida n acc))))))) (fak_tco 5 1)";
    assert_eq!(eval_vm(code), "120");
}

#[test]
fn test_vm_ffi_exec() {
    let lib_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/libmita_test.so");
    
    let code0 = format!("(mite \"{}\" \"mita_test_forty_two\")", lib_path);
    assert_eq!(eval_vm(&code0), "42");

    let code1 = format!("(mite \"{}\" \"mita_test_negate\" 5)", lib_path);
    assert_eq!(eval_vm(&code1), "-5");

    let code2 = format!("(mite \"{}\" \"mita_test_add\" 3 4)", lib_path);
    assert_eq!(eval_vm(&code2), "7");
}

#[test]
fn test_vm_cond() {
    assert_eq!(eval_vm("(dala ((shato 1 1) 'da) (da 'nye))"), "da");
    assert_eq!(eval_vm("(dala ((shato 1 2) 'da) (da 'nye))"), "nye");
}

#[test]
fn test_vm_progn() {
    assert_eq!(eval_vm("(in 1 2 3)"), "3");
}

#[test]
fn test_vm_nested_calls() {
    assert_eq!(eval_vm("(celi (movo 5 3) (celida 2 4))"), "10");
}

#[test]
fn test_vm_higher_order() {
    let code = "((mita (f) (f 5)) (mita (x) (celi x x)))";
    assert_eq!(eval_vm(code), "10");
}
