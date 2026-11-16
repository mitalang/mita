//! Environment contract test for the MLIR -> LLVM IR -> object -> ELF pipeline.
//!
//! Later tasks generate MLIR from MITA and feed it through the same tools, so this
//! test proves the local toolchain can lower a minimal module end to end.
//! It skips cleanly when the tools are not on PATH.

fn tool_exists(name: &str) -> bool {
    std::process::Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn minimal_mlir_builds_and_runs() {
    if !tool_exists("mlir-translate") || !tool_exists("llc") {
        eprintln!("skip: mlir-translate/llc not found");
        return;
    }

    let dir = std::env::temp_dir().join(format!("mita-mlir-contract-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mlir = dir.join("minimal.mlir");
    let ll = dir.join("minimal.ll");
    let obj = dir.join("minimal.o");
    let exe = dir.join("minimal");
    std::fs::write(&mlir, include_str!("fixtures/minimal.mlir")).unwrap();

    assert!(std::process::Command::new("mlir-translate")
        .args([
            "--mlir-to-llvmir",
            mlir.to_str().unwrap(),
            "-o",
            ll.to_str().unwrap()
        ])
        .status()
        .unwrap()
        .success());
    assert!(std::process::Command::new("llc")
        .args([
            "-filetype=obj",
            "-mtriple=x86_64-unknown-linux-gnu",
            ll.to_str().unwrap(),
            "-o",
            obj.to_str().unwrap()
        ])
        .status()
        .unwrap()
        .success());
    assert!(std::process::Command::new("cc")
        .args([obj.to_str().unwrap(), "-o", exe.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let status = std::process::Command::new(&exe).status().unwrap();
    assert_eq!(status.code(), Some(42));
}
