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
