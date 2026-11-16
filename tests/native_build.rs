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

#[test]
fn builds_and_runs_add_program() {
    if !tool_exists("mlir-translate") || !tool_exists("llc") {
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
