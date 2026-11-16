//! Environment contract test for the MLIR -> LLVM IR -> object -> ELF pipeline.
//!
//! Later tasks generate MLIR from MITA and feed it through the same tools, so this
//! test proves the local toolchain can lower a minimal module end to end.
//! It skips cleanly when the tools are not on PATH or when the host architecture
//! has no known Linux triple to target.

fn tool_exists(name: &str) -> bool {
    std::process::Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Map the host architecture to the LLVM triple `llc` should target.
///
/// The emitted object is linked by the host `cc`, so the triple has to match the
/// host architecture. Returns `None` for architectures we have no mapping for, which
/// makes the test skip instead of failing for an environment reason.
fn host_triple(arch: &str) -> Option<&'static str> {
    match arch {
        "x86_64" => Some("x86_64-unknown-linux-gnu"),
        "aarch64" => Some("aarch64-unknown-linux-gnu"),
        "riscv64" => Some("riscv64-unknown-linux-gnu"),
        _ => None,
    }
}

#[test]
fn host_triple_maps_known_archs_and_rejects_others() {
    assert_eq!(host_triple("x86_64"), Some("x86_64-unknown-linux-gnu"));
    assert_eq!(host_triple("aarch64"), Some("aarch64-unknown-linux-gnu"));
    assert_eq!(host_triple("riscv64"), Some("riscv64-unknown-linux-gnu"));
    assert_eq!(host_triple("x86"), None);
    assert_eq!(host_triple("powerpc64"), None);
    assert_eq!(host_triple(""), None);
}

#[test]
fn minimal_mlir_builds_and_runs() {
    if !tool_exists("mlir-translate") || !tool_exists("llc") {
        eprintln!("skip: mlir-translate/llc not found");
        return;
    }

    let triple = match host_triple(std::env::consts::ARCH) {
        Some(triple) => triple,
        None => {
            eprintln!(
                "skip: no known Linux triple for host arch '{}'",
                std::env::consts::ARCH
            );
            return;
        }
    };

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
            &format!("-mtriple={}", triple),
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
