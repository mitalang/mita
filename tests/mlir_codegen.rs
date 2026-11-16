use mita::Parser;

fn compile_source(source: &str) -> String {
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
    mita::backend::mlir::compile(&exprs).unwrap()
}

#[test]
fn emits_main_and_add() {
    let mlir = compile_source("(print (celi 1 2))");
    assert!(mlir.contains("llvm.func @main"));
    assert!(mlir.contains("@mita_add"));
    assert!(mlir.contains("@mita_print"));
}

#[test]
fn emits_cond_branch_and_runtime_calls() {
    let mlir = compile_source("(ka (shato 1 1) 2 3)");
    assert!(mlir.contains("@mita_eq"));
    assert!(mlir.contains("llvm.cond_br"));
}

#[test]
fn emits_cons_and_list_calls() {
    let mlir = compile_source("(lawa (upa 1 2))");
    assert!(mlir.contains("@mita_cons"));
    assert!(mlir.contains("@mita_car"));
}

#[test]
fn emits_string_global_and_constructor() {
    let mlir = compile_source("(print \"hi\")");
    assert!(mlir.contains("llvm.mlir.global"));
    assert!(mlir.contains("@mita_string"));
}
