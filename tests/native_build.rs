use std::path::PathBuf;
use std::process::Command;

use mita::backend::build::{build_files, BuildOptions};

fn tool_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn tempdir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mita-native-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn tools_available() -> bool {
    tool_exists("mlir-translate") && tool_exists("llc")
}

/// Builds and runs a single source program, returning its stdout.
fn run_program(source: &str) -> String {
    let dir = tempdir();
    let src = dir.join("prog.mita");
    std::fs::write(&src, source).unwrap();
    let exe = dir.join("prog");
    build_files(
        &[src.display().to_string()],
        &BuildOptions {
            target: "x86_64-unknown-linux-gnu".into(),
            output: exe.clone(),
            emit_mlir: None,
            keep_temps: false,
        },
    )
    .unwrap();
    let out = Command::new(&exe).output().unwrap();
    assert!(
        out.status.success(),
        "program failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn builds_and_runs_add_program() {
    if !tools_available() {
        return;
    }
    let dir = tempdir();
    let src = dir.join("add.mita");
    std::fs::write(&src, "(print (celi 1 2))").unwrap();
    let exe = dir.join("add");
    build_files(
        &[src.display().to_string()],
        &BuildOptions {
            target: "x86_64-unknown-linux-gnu".into(),
            output: exe.clone(),
            emit_mlir: None,
            keep_temps: false,
        },
    )
    .unwrap();
    let out = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\nnil\n");
}

#[test]
fn runs_expressions_control_flow_and_locals() {
    if !tools_available() {
        return;
    }
    let cases = [
        ("(ka (shato 1 1) 10 20)", "10\n"),
        ("(tido ((x 1)) (plama x (celi x 1)) x)", "2\n"),
        ("(in 1 2 3)", "3\n"),
        ("(dala ((shato 1 2) 10) 20)", "20\n"),
        ("(dala ((shato 1 2) 10) (celi 1 2))", "3\n"),
        ("(dala ((shato 1 1) 10) 20)", "10\n"),
        ("(lawa (upa 1 2))", "1\n"),
        ("'(1 2)", "(1 2)\n"),
        ("(print \"hi\")", "\"hi\"\nnil\n"),
    ];
    for (source, expected) in cases {
        let out = run_program(source);
        assert_eq!(out, expected, "source: {}", source);
    }
}
