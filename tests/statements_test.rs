use std::rc::Rc;

use mita::compiler::Compiler;
use mita::vm::VM;
use mita::{Context, Expr, Parser};

fn parse_all(source: &str) -> Vec<Rc<Expr>> {
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

fn eval_tree(source: &str) -> String {
    let mut context = Context::new(0);
    let mut result = Rc::new(Expr::Nil);
    for expr in parse_all(source) {
        result = context.eval_toplevel(expr);
    }
    result.to_string()
}

fn eval_vm(source: &str) -> String {
    let exprs = parse_all(source);
    let (func, globals) = Compiler::compile_toplevel(&exprs);
    let mut vm = VM::new();
    mita::vm::builtins::register_all(&mut vm);
    for (name, value) in globals {
        vm.define_global(&name, value);
    }
    let closure = Rc::new(mita::vm::value::Closure {
        func: Rc::new(func),
        upvalues: Vec::new(),
    });
    format!("{}", vm.run(closure))
}

#[test]
fn tree_and_vm_agree_on_special_forms() {
    let cases = [
        (
            "muhe",
            "(zido ((f (muhe (x) (celi x 1)))) (f 5))",
            "6",
        ),
        (
            "muhe zero args",
            "(zido ((f (muhe () 42))) (f))",
            "42",
        ),
        (
            "sada",
            "(sada ((inc (muhe (x) (celi x 1))))) (inc 5)",
            "6",
        ),
        (
            "sada mutual recursion",
            "(sada (\
               (myeven (muhe (n) (ka (shato n 0) da (myodd (lata n 1))))) \
               (myodd (muhe (n) (ka (shato n 0) nye (myeven (lata n 1))))))) \
             (myeven 10)",
            "da",
        ),
        ("zido", "(zido ((x 1) (y 2)) (celi x y))", "3"),
        ("zido empty", "(zido () 1)", "1"),
        ("zido shadowing", "(zido ((x 1)) (zido ((x 2)) x))", "2"),
        ("ka true", "(ka da 'yes 'no)", "yes"),
        ("ka false", "(ka nye 'yes 'no)", "no"),
        ("ka only da truthy", "(ka 42 'yes 'no)", "no"),
        (
            "dala matching clause",
            "(dala ((shato 1 2) 'a) (da 'b))",
            "b",
        ),
        (
            "dala bare atom fallback",
            "(dala ((shato 1 2) 'a) 'fallback)",
            "fallback",
        ),
        (
            "dala last clause re-eval",
            "(dala ((shato 1 2) 'a) (celi 1 2))",
            "3",
        ),
        ("in", "(in 1 2 3)", "3"),
        ("in empty", "(in)", "nil"),
        ("movo", "(zido ((x 1) (y 2)) (in (movo x 3 y 4) (celi x y)))", "7"),
        ("plata atom", "'a", "a"),
        ("plata list", "'(1 2)", "(1 2)"),
    ];

    for (name, source, expected) in cases {
        assert_eq!(eval_tree(source), expected, "tree: {}", name);
        assert_eq!(eval_vm(source), expected, "vm: {}", name);
    }
}
