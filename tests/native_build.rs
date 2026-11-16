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
    std::fs::write(&src, "(beru (celi 1 2))").unwrap();
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
        ("(zido ((x 1)) (movo x (celi x 1)) x)", "2\n"),
        ("(in 1 2 3)", "3\n"),
        ("(dala ((shato 1 2) 10) 20)", "20\n"),
        ("(dala ((shato 1 2) 10) (celi 1 2))", "3\n"),
        ("(dala ((shato 1 1) 10) 20)", "10\n"),
        ("(lawa (upa 1 2))", "1\n"),
        ("'(1 2)", "(1 2)\n"),
        ("(beru \"hi\")", "\"hi\"\nnil\n"),
    ];
    for (source, expected) in cases {
        let out = run_program(source);
        assert_eq!(out, expected, "source: {}", source);
    }
}

/// Builds and runs a program read from an example file, returning its stdout.
fn run_example(name: &str) -> String {
    if !tools_available() {
        return String::new();
    }
    let dir = tempdir();
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name);
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

/// Builds and runs a program assembled from multiple source files.
fn run_files(files: &[(&str, &str)]) -> String {
    let dir = tempdir();
    let mut inputs = Vec::new();
    for (name, source) in files {
        let path = dir.join(name);
        std::fs::write(&path, source).unwrap();
        inputs.push(path.display().to_string());
    }
    let exe = dir.join("prog");
    build_files(
        &inputs,
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
fn builds_and_runs_factorial() {
    if !tools_available() {
        return;
    }
    let out = run_example("factorial.mita");
    assert_eq!(out, "120\n");
}

#[test]
fn builds_and_runs_fibonacci() {
    if !tools_available() {
        return;
    }
    let out = run_example("fibonacci.mita");
    assert_eq!(out, "5\n");
}

#[test]
fn builds_and_runs_cond() {
    if !tools_available() {
        return;
    }
    let out = run_example("cond.mita");
    assert_eq!(out, "no\n");
}

#[test]
fn builds_and_runs_setq() {
    if !tools_available() {
        return;
    }
    let out = run_example("setq.mita");
    assert_eq!(out, "(2 . 1)\n");
}

/// A `dala` whose last clause has a falsy test and an uncallable head reaches
/// the `mita_panic` fallback at runtime. Regression for the NUL-termination
/// bug: the panic message must be a proper C string.
#[test]
fn cond_implicit_else_panics_on_uncallable_head() {
    if !tools_available() {
        return;
    }
    let dir = tempdir();
    let src = dir.join("prog.mita");
    std::fs::write(&src, "(dala (nye 1))").unwrap();
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
        !out.status.success(),
        "expected panic (non-zero exit), but the program succeeded"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("cond: implicit else clause is not callable"),
        "stderr did not contain the panic message: {}",
        stderr
    );
}

#[test]
fn runs_higher_order_lambda() {
    if !tools_available() {
        return;
    }
    let out = run_program("((muhe (f) (f 5)) (muhe (x) (celi x x)))");
    assert_eq!(out, "10\n");
}

#[test]
fn runs_mutual_recursion() {
    if !tools_available() {
        return;
    }
    let out = run_program(
        "(sada (\
         (myeven (muhe (n) (ka (shato n 0) da (myodd (lata n 1))))) \
         (myodd (muhe (n) (ka (shato n 0) nye (myeven (lata n 1))))))) \
         (myeven 10)",
    );
    assert_eq!(out, "da\n");
}

#[test]
fn builds_from_multiple_files() {
    if !tools_available() {
        return;
    }
    let out = run_files(&[
        ("lib.mita", "(sada ((double (muhe (x) (kuzi x du)))))"),
        ("app.mita", "(double 21)"),
    ]);
    assert_eq!(out, "42\n");
}

#[test]
fn local_lambda_shadows_muhe() {
    if !tools_available() {
        return;
    }
    let out = run_program(
        "(sada ((f (muhe (x) (celi x 1))))) \
         (zido ((f (muhe (x) (celi x 2)))) (f 5))",
    );
    assert_eq!(out, "7\n");
}

#[test]
fn local_lambda_shadows_builtin() {
    if !tools_available() {
        return;
    }
    let out = run_program("(zido ((celi (muhe (x) (kuzi x x)))) (celi 5))");
    assert_eq!(out, "25\n");
}

/// Cross builds are opt-in: only run when `MITA_TEST_CROSS=1` (requires the
/// cross gcc toolchains and qemu-user on the host).
fn cross_enabled() -> bool {
    std::env::var("MITA_TEST_CROSS").as_deref() == Ok("1")
}

/// Builds `source` for `target` and runs the result under qemu with `-L sysroot`.
fn run_cross(source: &str, target: &str, qemu: &str, sysroot: &str) -> String {
    let dir = tempdir();
    let src = dir.join("prog.mita");
    std::fs::write(&src, source).unwrap();
    let exe = dir.join("prog");
    build_files(
        &[src.display().to_string()],
        &BuildOptions {
            target: target.into(),
            output: exe.clone(),
            emit_mlir: None,
            keep_temps: false,
        },
    )
    .unwrap();
    let out = Command::new(qemu)
        .arg("-L")
        .arg(sysroot)
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "cross program failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn builds_aarch64_and_runs_under_qemu() {
    if !cross_enabled() {
        return;
    }
    let out = run_cross(
        "(celi 20 22)",
        "aarch64-unknown-linux-gnu",
        "qemu-aarch64",
        "/usr/aarch64-linux-gnu",
    );
    assert_eq!(out, "42\n");
}

#[test]
fn builds_riscv64_and_runs_under_qemu() {
    if !cross_enabled() {
        return;
    }
    let out = run_cross(
        "(celi 20 22)",
        "riscv64-unknown-linux-gnu",
        "qemu-riscv64",
        "/usr/riscv64-linux-gnu",
    );
    assert_eq!(out, "42\n");
}
