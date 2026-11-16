//! Contract test for the native C runtime value ABI.
//!
//! The MLIR backend lowers every MITA value to `!llvm.struct<(i64, i64)>` and
//! calls the C symbols declared in `runtime/mita_runtime.h`. These tests
//! compile a small C harness against `runtime/mita_runtime.c` and assert the
//! exact stdout, so the runtime's value representation, printing, and core
//! operations are pinned to the same behavior as the bytecode VM.
//!
//! Skips cleanly when a C compiler is not on PATH.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn tool_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Compile `source` (a C file) together with the runtime and run the binary,
/// returning its stdout.
fn compile_and_run(source: &str, test_id: &str) -> String {
    let root = repo_root();
    let runtime_dir = root.join("runtime");
    let runtime_c = runtime_dir.join("mita_runtime.c");

    let dir = std::env::temp_dir().join(format!("mita-runtime-{}-{}", test_id, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let harness_c = dir.join("harness.c");
    let exe = dir.join("harness");
    std::fs::write(&harness_c, source).unwrap();

    let out = Command::new("cc")
        .args([
            "-I",
            runtime_dir.to_str().unwrap(),
            runtime_c.to_str().unwrap(),
            harness_c.to_str().unwrap(),
            "-o",
            exe.to_str().unwrap(),
        ])
        .output()
        .expect("failed to spawn cc");
    assert!(
        out.status.success(),
        "cc failed to compile harness:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = Command::new(&exe).output().expect("failed to run harness");
    assert!(
        out.status.success(),
        "harness exited with {:?}:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

// The exact harness from the task brief.
const MINIMAL_HARNESS: &str = r#"
#include <stdio.h>
#include "mita_runtime.h"

int main(void) {
    mita_value a = mita_int(1);
    mita_value b = mita_int(2);
    mita_value sum = mita_add(a, b);
    mita_print_value(sum);
    mita_value pair = mita_cons(a, b);
    mita_print_value(pair);
    mita_print_value(mita_car(pair));
    mita_print_value(mita_cdr(pair));
    return 0;
}
"#;

#[test]
fn minimal_value_abi_roundtrip() {
    if !tool_exists("cc") {
        eprintln!("skip: cc not found");
        return;
    }
    let stdout = compile_and_run(MINIMAL_HARNESS, "minimal");
    assert_eq!(stdout, "3\n(1 . 2)\n1\n2\n");
}

const FULL_HARNESS: &str = r#"
#include <stdio.h>
#include "mita_runtime.h"

int main(void) {
    /* truthiness: only bool true is truthy */
    printf("%lld\n", (long long)mita_truthy(mita_bool(1)));
    printf("%lld\n", (long long)mita_truthy(mita_bool(0)));
    printf("%lld\n", (long long)mita_truthy(mita_nil()));
    printf("%lld\n", (long long)mita_truthy(mita_int(42)));

    /* structural equality; strings/symbols by content */
    mita_value s1 = mita_string("hi", 2);
    mita_value s2 = mita_string("hi", 2);
    mita_value s3 = mita_string("ho", 2);
    mita_print_value(mita_eq(s1, s2));
    mita_print_value(mita_eq(s1, s3));
    mita_print_value(mita_eq(mita_symbol("x", 1), mita_symbol("x", 1)));
    mita_print_value(mita_eq(mita_symbol("x", 1), mita_string("x", 1)));
    mita_print_value(mita_eq(mita_nil(), mita_bool(0)));
    mita_print_value(mita_eq(mita_nil(), mita_nil()));
    mita_print_value(mita_ne(mita_int(1), mita_int(2)));

    /* lists and improper lists */
    mita_value list = mita_cons(mita_int(1), mita_cons(mita_int(2), mita_cons(mita_int(3), mita_nil())));
    mita_print_value(list);
    mita_value improper = mita_cons(mita_int(1), mita_cons(mita_int(2), mita_int(3)));
    mita_print_value(improper);
    mita_value nested = mita_cons(list, mita_nil());
    mita_print_value(nested);

    /* car/cdr on non-cons return nil */
    mita_print_value(mita_car(mita_int(7)));
    mita_print_value(mita_cdr(mita_int(7)));

    /* printing of each scalar type */
    mita_print_value(mita_string("hello", 5));
    mita_print_value(mita_symbol("abc", 3));
    mita_print_value(mita_bool(1));
    mita_print_value(mita_bool(0));
    mita_print_value(mita_nil());

    /* mita_print: space-separated, trailing newline */
    mita_value args[3] = { mita_int(1), mita_string("two", 3), mita_nil() };
    mita_print(args, 3);

    /* arithmetic and numeric comparisons */
    mita_print_value(mita_sub(mita_int(10), mita_int(4)));
    mita_print_value(mita_mul(mita_int(3), mita_int(4)));
    mita_print_value(mita_div(mita_int(7), mita_int(2)));
    mita_print_value(mita_lt(mita_int(1), mita_int(2)));
    mita_print_value(mita_gt(mita_int(1), mita_int(2)));
    mita_print_value(mita_le(mita_int(2), mita_int(2)));
    mita_print_value(mita_ge(mita_int(1), mita_int(2)));

    return 0;
}
"#;

#[test]
fn full_runtime_api() {
    if !tool_exists("cc") {
        eprintln!("skip: cc not found");
        return;
    }
    let stdout = compile_and_run(FULL_HARNESS, "full");
    let expected = r#"1
0
0
0
da
nye
da
nye
da
da
da
(1 2 3)
(1 2 . 3)
((1 2 3))
nil
nil
"hello"
abc
da
nye
nil
1 "two" nil
6
12
3
da
nye
da
nye
"#;
    assert_eq!(stdout, expected);
}
