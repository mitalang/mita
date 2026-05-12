use mita::Parser;

#[test]
fn test_sexpr_parse() {
    let tests = vec![
        ("nil", "nil"),
        ("a", "a"),
        ("(a . nil)", "(a)"),
        ("(a . b)", "(a . b)"),
        ("(a . (b . nil))", "(a b)"),
        ("((a . nil) . nil)", "((a))"),
        ("(a . (b . (c . nil)))", "(a b c)"),
        ("(a . (b . (c . (d . nil))))", "(a b c d)"),
        ("(a . (b . (c . (d . (e . nil)))))", "(a b c d e)"),
        ("((a . (b . nil)) . (c . nil))", "((a b) c)"),
        ("(a . (b . ((c . (d . nil)) . nil)))", "(a b (c d))"),
        ("(a . ((b . c) . nil))", "(a (b . c))"),
    ];

    for (s, l) in tests {
        let mut p = Parser::new(s);
        let expr = p.sexpr();
        let str = expr.sexpr_string();
        assert_eq!(str, s, "{} != {}", str, s);
        let str = expr.to_string();
        assert_eq!(str, l, "{} != {}", str, l);
    }
}

#[test]
fn test_list_parse() {
    let tests = vec![
        ("nil", "nil"),
        ("a", "a"),
        ("(a . nil)", "(a)"),
        ("(a . b)", "(a . b)"),
        ("(a . (b . nil))", "(a b)"),
        ("((a . nil) . nil)", "((a))"),
        ("(a . (b . (c . nil)))", "(a b c)"),
        ("(a . (b . (c . (d . nil))))", "(a b c d)"),
        ("(a . (b . (c . (d . (e . nil)))))", "(a b c d e)"),
        ("((a . (b . nil)) . (c . nil))", "((a b) c)"),
        ("(a . (b . ((c . (d . nil)) . nil)))", "(a b (c d))"),
        ("(a . ((b . c) . nil))", "(a (b . c))"),
    ];

    for (s, l) in tests {
        let mut p = Parser::new(l);
        let expr = p.list();
        let str = expr.sexpr_string();
        assert_eq!(str, s, "{} != {}", str, s);
        let str = expr.to_string();
        assert_eq!(str, l, "{} != {}", str, l);
    }
}

#[test]
fn test_parse_quote() {
    let tests = vec![
        ("()", "nil", "nil", "nil"),
        ("a", "a", "a", "a"),
        ("'a", "(plata . (a . nil))", "'a", "(plata a)"),
        ("'(a)", "(plata . ((a . nil) . nil))", "'(a)", "(plata (a))"),
        ("''a", "(plata . ((plata . (a . nil)) . nil))", "''a", "(plata (plata a))"),
        ("''(a)", "(plata . ((plata . ((a . nil) . nil)) . nil))", "''(a)", "(plata (plata (a)))"),
        ("('a 'b 'c)", "((plata . (a . nil)) . ((plata . (b . nil)) . ((plata . (c . nil)) . nil)))", "('a 'b 'c)", "((plata a) (plata b) (plata c))"),
    ];

    for (l, s, quoted, nonquoted) in tests {
        let mut p = Parser::new(l);
        let expr = p.list();
        let str = expr.sexpr_string();
        assert_eq!(str, s, "SExpr mismatch for {}: {} != {}", l, str, s);
        let str = expr.to_string();
        assert_eq!(str, quoted, "Quoted mismatch for {}: {} != {}", l, str, quoted);
        let str = expr.string_no_quote();
        assert_eq!(str, nonquoted, "Nonquoted mismatch for {}: {} != {}", l, str, nonquoted);
    }
}
