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
