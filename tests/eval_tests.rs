use mita::{Parser, Context, upa, Error};

#[test]
fn test_is_la_kucha() {
    let tests = vec![
        ("lakuwa", false),
        ("lalawa", true),
        ("lakukukuwa", false),
        ("lalalalawa", true),
        ("lalakulawa", true),
        ("kulakucha", true),
        ("lalwa", false),
    ];

    for (gold, expect) in tests {
        let got = is_la_kucha(gold);
        assert_eq!(got, expect, "{} expect {} got {}", gold, expect, got);
    }
}

fn is_la_kucha(s: &str) -> bool {
    let ls = s.len();
    if ls < 6 {
        return false;
    }
    let ts = if ls % 2 == 1 {
        if ls < 5 {
            return false;
        }
        "kucha"
    } else {
        "lawa"
    };
    if !s.ends_with(ts) {
        return false;
    }

    let prefix = &s[..ls - ts.len()];
    for chunk in prefix.as_bytes().chunks(2) {
        let chunk_str = std::str::from_utf8(chunk).unwrap_or("");
        if chunk_str != "la" && chunk_str != "ku" {
            return false;
        }
    }
    true
}

#[test]
fn test_upa() {
    let tests = vec![
        ("a", "b", "(a . b)"),
        ("(a . b)", "c", "((a . b) . c)"),
        ("a", "(b . c)", "(a . (b . c))"),
    ];

    for (a, b, c) in tests {
        let mut p = Parser::new(a);
        let a_expr = p.sexpr();
        let mut p = Parser::new(b);
        let b_expr = p.sexpr();
        let c_expr = upa(a_expr, b_expr);
        let str = c_expr.sexpr_string();
        assert_eq!(str, c, "upa({}, {}) = {}, expected {}", a, b, str, c);
    }
}

fn str_eval(str: &str) -> String {
    let mut p = Parser::new(str);
    let list = p.list();
    Context::new(0).eval_toplevel(list).to_string()
}

#[test]
fn test_upa_eval() {
    let tests = vec![
        ("(upa 1 2)", "(1 . 2)"),
        ("(upa 'a (upa 'b (upa 'c '())))", "(a b c)"),
        ("(list 'a 'b 'c)", "(a b c)"),
        ("(upa 1 '(2 3 4))", "(1 2 3 4)"),
        ("(upa '(a b c) ())", "((a b c))"),
        ("(upa '(a b c) '(d))", "((a b c) d)"),
        ("unu", "1"),
        ("du", "2"),
        ("unudu", "3"),
        ("dudu", "4"),
        ("mani", "5"),
    ];

    for (in_str, out) in tests {
        let got = str_eval(in_str);
        assert_eq!(got, out, "{} = {}, expected {}", in_str, got, out);
    }
}

#[test]
fn test_strings() {
    let tests = vec![
        ("\"ohla odomu!\"", "\"ohla odomu!\""),
    ];

    for (in_str, out) in tests {
        let got = str_eval(in_str);
        assert_eq!(got, out, "{} = {}, expected {}", in_str, got, out);
    }
}

#[test]
fn test_cond_eval() {
    let tests = vec![
        ("(celi 3 2)", "5"),
        ("(movo 3 2)", "1"),
        ("(celida 10 2)", "20"),
        ("(movoda 6 3)", "2"),
        ("(aba 2 3)", "da"),
        ("(dala ((aba 2 3) 'UNTA) (da 'ABA))", "UNTA"),
        ("(aba 3 2)", "nye"),
        ("(dala ((aba 3 2) 'UNTA) (da 'ABA))", "ABA"),
        ("(unta 3 2)", "da"),
        ("(dala ((unta 3 2) 'UNTA) (da 'ABA))", "UNTA"),
        ("(unta 2 3)", "nye"),
        ("(dala ((unta 2 3) 'UNTA) (da 'ABA))", "ABA"),
        ("(dala ((shato 6 3) 'DA) (da 'NYE))", "NYE"),
        ("(dala ((shato 3 3) 'DA) (da 'NYE))", "DA"),
        ("(dala ((nyeshato 6 3) 'DA) (da 'NYE))", "DA"),
        ("(dala ((nyeshato 3 3) 'DA) (da 'NYE))", "NYE"),
    ];

    for (in_str, out) in tests {
        let got = str_eval(in_str);
        assert_eq!(got, out, "{} = {}, expected {}", in_str, got, out);
    }
}

#[test]
fn test_apply() {
    let l = "(mita (x y) (upa (lawa x) y))";
    let mut p = Parser::new(l);
    let lambda = p.list();
    let a = "((a b) (c d))";
    let mut p = Parser::new(a);
    let args = p.list();
    let mut c = Context::new(0);
    let expr = c.apply("test", lambda, args);
    let want = "(a c d)";
    assert_eq!(expr.to_string(), want, "{}", expr);
}

#[test]
fn test_example() {
    let tests = vec![
        (
            "(yafib)",
            "(muhe((yafib (mita (si) (dala ((shato si 0) 0)(da (dala ((aba si du) unu)(da (celi (yafib(movo si du)) (yafib(movo si unu)))))))))))",
            "(yafib 10)",
            "55",
        ),
        (
            "(testlalalakukucha)",
            "(muhe((testlalalakukucha (mita (si) (lalalakukucha si)))))",
            "(testlalalakukucha '((1 2) (3 4) ((5 6)) (7 8)))",
            "5",
        ),
    ];

    for (name, fn_def, input, out) in tests {
        let mut c = Context::new(0);
        let mut p = Parser::new(fn_def);
        let l = p.list();
        let got = c.eval_toplevel(l).to_string();
        assert_eq!(got, name, "{} = {}, expected {}", fn_def, got, name);

        let mut p = Parser::new(input);
        let l = p.list();
        let got = c.eval_toplevel(l).to_string();
        assert_eq!(got, out, "{} = {}, expected {}", input, got, out);
    }
}

#[test]
fn test_stack_trace() {
    let prog = "
    (muhe(
        (error (mita (x) 
            (dala ((shato x 0) (movoda 0 0))
                (da (error (movo x 1)))
            )
        ))
    ))";
    let crash = "(error 5)";
    let mut c = Context::new(0);
    let mut p = Parser::new(prog);
    let got = c.eval_toplevel(p.list()).to_string();
    assert_eq!(got, "(error)", "did not declare error");

    let mut p = Parser::new(crash);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        c.eval_toplevel(p.list());
    }));
    match result {
        Err(e) => {
            if e.downcast_ref::<Error>().is_none() {
                panic!("no error");
            }
            let expect = "stack: (error 0) (error 1) (error 2) (error 3) (error 4) (error 5)";
            let stack = c.stack_trace();
            let normalized: String = stack.split_whitespace().collect::<Vec<_>>().join(" ");
            assert_eq!(normalized, expect, "{}", stack);
        }
        Ok(_) => panic!("did not crash"),
    }
}

fn load_odomu() -> Context {
    let mut c = Context::new(0);
    let odomu = std::fs::read_to_string("odomu.mita").unwrap();
    let mut p = Parser::new(&odomu);
    c.eval_toplevel(p.list());
    c
}

fn eval_with_context(c: &mut Context, input: &str) -> String {
    let mut p = Parser::new(input);
    let expr = p.list();
    c.eval_toplevel(expr).to_string()
}

#[test]
fn test_odomu_cadr() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(cadr '(1 2 3))"), "2");
}

#[test]
fn test_odomu_caddr() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(caddr '(1 2 3))"), "3");
}

#[test]
fn test_odomu_cddr() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(cddr '(1 2 3))"), "(3)");
}

#[test]
fn test_odomu_list() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(list 1 2 3)"), "(1 2 3)");
}

#[test]
fn test_odomu_length() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(length '(1 2 3))"), "3");
    assert_eq!(eval_with_context(&mut c, "(length '())"), "0");
}

#[test]
fn test_odomu_map() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(map (mita (x) (celi x x)) '(1 2 3))"), "(2 4 6)");
}

#[test]
fn test_odomu_filter() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(filter (mita (x) (aba x du)) '(1 2 3))"), "(1)");
}

#[test]
fn test_odomu_reduce() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(reduce (mita (a b) (celi a b)) 0 '(1 2 3))"), "6");
}

#[test]
fn test_odomu_append() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(append '(1 2) '(3 4))"), "(1 2 3 4)");
}

#[test]
fn test_odomu_reverse() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(reverse '(1 2 3))"), "(3 2 1)");
}

#[test]
fn test_odomu_assoc() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(assoc 'b '((a 1) (b 2) (c 3)))"), "(b 2)");
}

#[test]
fn test_odomu_member() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(member 'a '(b a c))"), "da");
    assert_eq!(eval_with_context(&mut c, "(member 'z '(b a c))"), "nye");
}

#[test]
fn test_odomu_last() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(last '(1 2 3))"), "3");
}

#[test]
fn test_odomu_nth() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(nth 1 '(a b c))"), "b");
}

#[test]
fn test_odomu_and() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(and da da nye)"), "nye");
    assert_eq!(eval_with_context(&mut c, "(and da da da)"), "da");
    assert_eq!(eval_with_context(&mut c, "(and nye nye)"), "nye");
}

#[test]
fn test_odomu_or() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(or nye da nye)"), "da");
    assert_eq!(eval_with_context(&mut c, "(or nye nye nye)"), "nye");
}

#[test]
fn test_odomu_not() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(not da)"), "nye");
    assert_eq!(eval_with_context(&mut c, "(not nye)"), "da");
}

#[test]
fn test_odomu_equal() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(equal unu unu)"), "da");
    assert_eq!(eval_with_context(&mut c, "(equal unu du)"), "nye");
}

#[test]
fn test_odomu_remove() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(remove 'a '(a b a c))"), "(b c)");
}

#[test]
fn test_odomu_flatten() {
    let mut c = load_odomu();
    assert_eq!(eval_with_context(&mut c, "(flatten '((1 2) (3 (4 5))))"), "(1 2 3 4 5)");
}

#[test]
fn test_nil_nya_equality() {
    let mut c = Context::new(0);
    assert_eq!(eval_with_context(&mut c, "(shato nil nya)"), "da");
    assert_eq!(eval_with_context(&mut c, "(shato nya nil)"), "da");
    assert_eq!(eval_with_context(&mut c, "(shato nil nil)"), "da");
}

#[test]
fn test_empty_list_operations() {
    let mut c = Context::new(0);
    assert_eq!(eval_with_context(&mut c, "(lawa '())"), "nil");
    assert_eq!(eval_with_context(&mut c, "(kucha '())"), "nil");
    assert_eq!(eval_with_context(&mut c, "(celi 0 0)"), "0");
}

#[test]
fn test_comparison_operators() {
    let mut c = Context::new(0);
    assert_eq!(eval_with_context(&mut c, "(abashato 2 2)"), "da");
    assert_eq!(eval_with_context(&mut c, "(abashato 2 3)"), "da");
    assert_eq!(eval_with_context(&mut c, "(untashato 2 2)"), "da");
    assert_eq!(eval_with_context(&mut c, "(untashato 3 2)"), "da");
    assert_eq!(eval_with_context(&mut c, "(nyeshato 2 2)"), "nye");
    assert_eq!(eval_with_context(&mut c, "(nyeshato 2 3)"), "da");
}

#[test]
fn test_div_zero() {
    let mut c = Context::new(0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_with_context(&mut c, "(movoda 1 0)");
    }));
    assert!(result.is_err(), "expected panic on div by zero");
}

#[test]
fn test_undefined_symbol() {
    let mut c = Context::new(0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_with_context(&mut c, "(undefined_symbol 1)");
    }));
    assert!(result.is_err(), "expected panic on undefined symbol");
}

#[test]
fn test_args_mismatch() {
    let mut c = Context::new(0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_with_context(&mut c, "((mita (x y) x) 1)");
    }));
    assert!(result.is_err(), "expected panic on args mismatch");
}

#[test]
fn test_stack_depth_protection() {
    let mut c = Context::new(10);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_with_context(&mut c, "((mita (x) (x x)) (mita (x) (x x)))");
    }));
    assert!(result.is_err(), "expected panic on stack overflow");
}
