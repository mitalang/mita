use mita::Parser;
use mita::compiler::Compiler;
use mita::vm::{VM, value::Closure};
use std::rc::Rc;

#[test]
fn test_debug_if_vm() {
    let mut parser = Parser::new("(ka da 1 2)");
    let expr = parser.list();
    let (func, _) = Compiler::compile_toplevel(&[expr]);
    println!("Bytecode:");
    for (i, inst) in func.bytecode.iter().enumerate() {
        println!("  {:4}: {}", i, mita::vm::isa::disasm(*inst, i, &func.const_pool));
    }
    
    let closure = Rc::new(Closure {
        func: Rc::new(func),
        upvalues: Vec::new(),
    });
    let mut vm = VM::new();
    mita::vm::builtins::register_all(&mut vm);
    let result = vm.run(closure);
    println!("Result: {:?}", result);
}
