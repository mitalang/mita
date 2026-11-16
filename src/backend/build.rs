//! Native build pipeline: MITA source -> MLIR -> LLVM IR -> object -> ELF.
//!
//! `build_files` parses the inputs, generates LLVM-dialect MLIR, then drives
//! `mlir-translate`, `llc`, and the target C compiler (which also compiles the
//! embedded runtime and links the final executable). The runtime sources are
//! embedded with `include_str!` so the `mita` binary has no install-time
//! dependency on the repository layout.

use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::expr::Expr;

const RUNTIME_C: &str = include_str!("../../runtime/mita_runtime.c");
const RUNTIME_H: &str = include_str!("../../runtime/mita_runtime.h");

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub target: String,
    pub output: PathBuf,
    pub emit_mlir: Option<PathBuf>,
    pub keep_temps: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        BuildOptions {
            target: "x86_64-unknown-linux-gnu".to_string(),
            output: PathBuf::from("a.out"),
            emit_mlir: None,
            keep_temps: false,
        }
    }
}

/// Parse and build a set of `.mita` source files into `opts.output`.
pub fn build_files(inputs: &[String], opts: &BuildOptions) -> Result<(), String> {
    let mut exprs: Vec<Rc<Expr>> = Vec::new();
    for input in inputs {
        let content = std::fs::read_to_string(input)
            .map_err(|e| format!("failed to read '{}': {}", input, e))?;
        let parsed = parse_source(&content)
            .map_err(|e| format!("{}: {}", input, e))?;
        exprs.extend(parsed);
    }

    let mlir = crate::backend::mlir::compile(&exprs)?;

    if let Some(path) = &opts.emit_mlir {
        std::fs::write(path, &mlir)
            .map_err(|e| format!("failed to write MLIR to '{}': {}", path.display(), e))?;
    }

    let mlir_translate = find_tool("MLIR_TRANSLATE", "mlir-translate", "apt install mlir-19-tools")?;
    let llc = find_tool("LLC", "llc", "install the llvm toolchain")?;
    let cc_name = cc_for_target(&opts.target)?;
    let cc = find_tool("CC", cc_name, "install a C compiler")?;

    let tmp = temp_dir();
    std::fs::create_dir_all(&tmp)
        .map_err(|e| format!("failed to create temp dir '{}': {}", tmp.display(), e))?;

    let runtime_c = tmp.join("mita_runtime.c");
    let runtime_h = tmp.join("mita_runtime.h");
    std::fs::write(&runtime_c, RUNTIME_C)
        .map_err(|e| format!("failed to write runtime C: {}", e))?;
    std::fs::write(&runtime_h, RUNTIME_H)
        .map_err(|e| format!("failed to write runtime header: {}", e))?;

    let mlir_path = tmp.join("prog.mlir");
    let ll_path = tmp.join("prog.ll");
    let obj_path = tmp.join("prog.o");
    let runtime_obj = tmp.join("runtime.o");
    std::fs::write(&mlir_path, &mlir)
        .map_err(|e| format!("failed to write MLIR: {}", e))?;

    let result = (|| -> Result<(), String> {
        run(
            Command::new(&mlir_translate).args([
                "--mlir-to-llvmir",
                mlir_path.to_str().unwrap(),
                "-o",
                ll_path.to_str().unwrap(),
            ]),
            "mlir-translate",
        )?;

        let mut llc_args = vec![
            "-O2".to_string(),
            "-filetype=obj".to_string(),
            format!("-mtriple={}", opts.target),
            "-relocation-model=pic".to_string(),
        ];
        llc_args.extend(llc_target_args(&opts.target));
        llc_args.push(ll_path.to_str().unwrap().to_string());
        llc_args.push("-o".to_string());
        llc_args.push(obj_path.to_str().unwrap().to_string());

        run(Command::new(&llc).args(&llc_args), "llc")?;

        run(
            Command::new(&cc).args([
                "-c",
                runtime_c.to_str().unwrap(),
                "-o",
                runtime_obj.to_str().unwrap(),
            ]),
            "cc (runtime)",
        )?;

        run(
            Command::new(&cc).args([
                obj_path.to_str().unwrap(),
                runtime_obj.to_str().unwrap(),
                "-o",
                opts.output.to_str().unwrap(),
            ]),
            "cc (link)",
        )?;

        Ok(())
    })();

    if !opts.keep_temps {
        let _ = std::fs::remove_dir_all(&tmp);
    }

    result
}

/// CLI entry point for `mita build ...`. `args` excludes the `build` token.
pub fn run_cli(args: &[String]) -> Result<(), String> {
    let mut opts = BuildOptions::default();
    let mut inputs: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                i += 1;
                if i >= args.len() {
                    return Err("--target requires a value".to_string());
                }
                opts.target = args[i].clone();
            }
            "--emit-mlir" => {
                i += 1;
                if i >= args.len() {
                    return Err("--emit-mlir requires a path".to_string());
                }
                opts.emit_mlir = Some(PathBuf::from(&args[i]));
            }
            "--keep-temps" => opts.keep_temps = true,
            "-o" => {
                i += 1;
                if i >= args.len() {
                    return Err("-o requires a path".to_string());
                }
                opts.output = PathBuf::from(&args[i]);
            }
            flag if flag.starts_with('-') && flag != "-" => {
                return Err(format!("unknown option: {}", flag));
            }
            input => inputs.push(input.to_string()),
        }
        i += 1;
    }

    if inputs.is_empty() {
        return Err("no input files".to_string());
    }

    build_files(&inputs, &opts)
}

/// Maps a target triple to the C compiler used to build/link the runtime.
///
/// The runtime C and the generated object are both consumed by this compiler,
/// so the whole pipeline is target-consistent (no host-cc leakage).
fn cc_for_target(target: &str) -> Result<&'static str, String> {
    match target {
        "x86_64-unknown-linux-gnu" => Ok("cc"),
        "aarch64-unknown-linux-gnu" => Ok("aarch64-linux-gnu-gcc"),
        "riscv64-unknown-linux-gnu" => Ok("riscv64-linux-gnu-gcc"),
        other => Err(format!(
            "target '{}' is not yet supported (supported: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, riscv64-unknown-linux-gnu)",
            other
        )),
    }
}

/// Extra `llc` feature flags needed to match the target C compiler's default
/// float ABI, so the generated object and the compiled runtime link cleanly.
fn llc_target_args(target: &str) -> Vec<String> {
    match target {
        // Debian's `riscv64-linux-gnu-gcc` defaults to hard double-float
        // (`-mabi=lp64d -march=rv64imafdc`), but LLVM's bare `riscv64` triple
        // defaults to soft-float `rv64i`. The linker rejects the mismatch
        // ("can't link soft-float modules with double-float modules"), so force
        // the same feature set as the cross gcc.
        "riscv64-unknown-linux-gnu" => vec!["-mattr=+m,+a,+f,+d,+c".to_string()],
        _ => vec![],
    }
}

/// Discovers a tool: `$env_var` -> PATH -> `/usr/lib/llvm-*/bin/`.
fn find_tool(env_var: &str, name: &str, hint: &str) -> Result<String, String> {
    if let Ok(p) = std::env::var(env_var) {
        if !p.trim().is_empty() {
            return Ok(p);
        }
    }
    if let Some(p) = which_in_path(name) {
        return Ok(p.to_string_lossy().into_owned());
    }
    if let Ok(entries) = std::fs::read_dir("/usr/lib") {
        for entry in entries.flatten() {
            let dir = entry.path();
            let dirname = dir
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            if dirname.starts_with("llvm-") {
                let candidate = dir.join("bin").join(name);
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().into_owned());
                }
            }
        }
    }
    Err(format!(
        "missing tool '{}' (set {} or install it; hint: {})",
        name, env_var, hint
    ))
}

fn which_in_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if candidate
                    .metadata()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
                {
                    return Some(candidate);
                }
            }
            #[cfg(not(unix))]
            {
                return Some(candidate);
            }
        }
    }
    None
}

fn temp_dir() -> PathBuf {
    let n = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("mita-build-{}-{}", std::process::id(), n))
}

/// Parses a source buffer into top-level expressions, converting parser panics
/// (the project's error convention) into `Result::Err`.
fn parse_source(content: &str) -> Result<Vec<Rc<Expr>>, String> {
    let result = std::panic::catch_unwind(|| {
        let mut parser = crate::Parser::new(content);
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
    });
    match result {
        Ok(exprs) => Ok(exprs),
        Err(payload) => {
            if let Some(err) = payload.downcast_ref::<crate::elementary::Error>() {
                Err(err.0.clone())
            } else if payload.downcast_ref::<crate::elementary::EOF>().is_some() {
                Err("unexpected end of input".to_string())
            } else if let Some(s) = payload.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                Err((*s).to_string())
            } else {
                Err("unknown parse error".to_string())
            }
        }
    }
}

fn run(cmd: &mut Command, what: &str) -> Result<(), String> {
    let output = cmd
        .output()
        .map_err(|e| format!("failed to run {}: {}", what, e))?;
    if !output.status.success() {
        return Err(format!(
            "{} failed:\n{}",
            what,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
