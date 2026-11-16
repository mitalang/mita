use std::collections::HashSet;
use std::path::{Path, PathBuf};

use mita::{Expr, Parser};

fn collect_symbols(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Nil => {}
        Expr::Atom(token) => out.push(token.text.clone()),
        Expr::Cons { lawa, kucha } => {
            collect_symbols(lawa, out);
            collect_symbols(kucha, out);
        }
    }
}

fn symbols_in(path: &Path) -> Vec<String> {
    let source = std::fs::read_to_string(path).unwrap();
    let mut parser = Parser::new(&source);
    let mut exprs = Vec::new();
    loop {
        match parser.skip_space() {
            '\n' => continue,
            '\0' => break,
            _ => {}
        }
        exprs.push(parser.list());
    }
    let mut symbols = Vec::new();
    for expr in &exprs {
        collect_symbols(expr, &mut symbols);
    }
    symbols
}

fn mita_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![
        root.join("odomu.mita"),
        root.join("net.mita"),
        root.join("http.mita"),
    ];
    for dir in ["examples", "bench"] {
        let mut entries: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("mita"))
            .collect();
        entries.sort();
        files.extend(entries);
    }
    files
}

#[test]
fn audit_covers_examples_and_bench() {
    let files = mita_files();
    assert!(
        files.iter().any(|path| path.to_string_lossy().contains("/examples/")),
        "naming audit must include examples/*.mita"
    );
    assert!(
        files.iter().any(|path| path.to_string_lossy().contains("/bench/")),
        "naming audit must include bench/*.mita"
    );
    assert!(
        files.len() >= 20,
        "naming audit only found {} .mita files",
        files.len()
    );
}

#[test]
fn removed_names_are_gone() {
    let removed = [
        "tido", "plama", "pupu", "celida", "movoda", "eq", "eql", "consp", "atom",
        "null", "list", "apply",
    ];
    let removed: HashSet<&str> = removed.into_iter().collect();

    for path in mita_files() {
        let symbols = symbols_in(&path);
        for symbol in symbols {
            assert!(
                !removed.contains(symbol.as_str()),
                "{} still contains removed name `{}`",
                path.display(),
                symbol
            );
        }
    }
}

#[test]
fn new_names_are_present() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let symbols: HashSet<String> = symbols_in(&root.join("odomu.mita")).into_iter().collect();
    for expected in [
        "tiga",
        "tigamani",
        "kuzitomo",
        "tigakundala",
        "untasi",
        "tigasi",
        "upasi",
        "unusi",
        "nyasi",
    ] {
        assert!(
            symbols.contains(expected),
            "odomu.mita is missing new name `{}`",
            expected
        );
    }
}

#[test]
fn removed_names_are_gone_from_rust_strings() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = [
        "src/eval.rs",
        "src/elementary.rs",
        "src/compiler/mod.rs",
        "src/vm/builtins.rs",
        "src/backend/mlir.rs",
    ];
    for file in files {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        for name in ["tido", "plama", "pupu", "celida", "movoda"] {
            let quoted = format!("\"{}\"", name);
            assert!(
                !source.contains(&quoted),
                "{} still contains Rust string literal {}",
                file,
                quoted
            );
        }
    }
}
