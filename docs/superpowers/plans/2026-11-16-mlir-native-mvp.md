# MITA MLIR Native MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `mita build` 把 MITA 源码编译成 x86-64 / aarch64 / riscv64 的 Linux ELF。

**Architecture:** 新增 `src/backend/`：`mlir.rs` 生成 MLIR LLVM dialect，`build.rs` 调用 `mlir-translate`、`llc` 和目标 C 编译器；`runtime/mita_runtime.c` 提供 `{i64,i64}` 值、列表、字符串、闭包和打印。LLVM 负责 native 寄存器分配与 spill。

**Tech Stack:** Rust 2021、LLVM/MLIR 19、C11 runtime、GNU/LLVM 交叉工具链、qemu-user。

**Spec:** `docs/superpowers/specs/2026-11-16-mlir-native-mvp-design.md`

## Global Constraints

- 只支持 `x86_64-unknown-linux-gnu`、`aarch64-unknown-linux-gnu`、`riscv64-unknown-linux-gnu` 三个 Linux ELF 目标。
- 使用 MLIR/LLVM 19；`mlir-translate` 缺失时给出明确安装提示。
- 不新增 Rust crate 依赖；外部工具通过 `std::process::Command` 调用。
- C runtime 通过 `include_str!` 内嵌，构建时写入临时目录。
- 值 ABI 固定为 `struct { int64_t tag; int64_t payload; }`，MLIR 类型为 `!llvm.struct<(i64, i64)>`。
- 只有 `da`（bool tag=1, payload=1）为真；`lawa`/`kucha` 对非 cons 返回 nil。
- `tido`/参数统一用 `alloca` 槽位；`plama` 只修改当前函数局部变量，找不到局部变量时编译报错。
- `muhe` 两阶段：先收集所有顶层函数名并声明，再编译函数体；支持自递归、互递归、前向引用。
- `dala` 严格对齐现有 VM：最后 clause 隐式 else；最后 test 为假时重新求值整个 clause；裸 atom 直接求值。
- 最后一个顶层表达式的结果自动打印，然后 `main` 返回 0。
- 非目标：上值捕获、变参 lambda、组合 car/cdr 选择器、FFI、socket/HTTP、GC、TCO、macOS/Windows。

## Review Focus

- `main` 是否真的打印最后一个顶层表达式结果；用 `examples/factorial.mita` 和 `examples/add.mita` 验证，而不是只看显式 `print`。
- `plama` 是否修改当前函数局部变量；用 `examples/setq.mita` 的 `tido` + `plama` 场景验证。
- `muhe` 的前向/自递归/互递归是否都能解析；用 factorial、fibonacci 和一对互递归函数验证。
- `dala` 的隐式 else 特殊语义是否与 VM 一致；用 `cond.mita` 的裸 atom 和最后 clause 重新求值场景验证。
- 三种目标的 `{i64,i64}` struct ABI 是否一致；每个目标都必须实际链接、运行并断言输出，不能只生成 MLIR。

---

### Task 1: 验证 MLIR 工具链契约

**Files:**
- Create: `tests/fixtures/minimal.mlir`
- Create: `tests/mlir_toolchain.rs`

**Interfaces:**
- Consumes: `mlir-translate`、`llc`、`cc`。
- Produces: 一个可重复运行的 toolchain 契约测试；后续任务据此确认 MLIR 语法和目标文件流程。

- [ ] **Step 1: 创建最小 MLIR fixture**

`tests/fixtures/minimal.mlir`：

```mlir
module {
  llvm.func @main() -> i32 {
    %c = llvm.mlir.constant(42 : i32) : i32
    llvm.return %c : i32
  }
}
```

- [ ] **Step 2: 写契约测试**

`tests/mlir_toolchain.rs`：如果 `mlir-translate` 或 `llc` 不在 PATH，打印 skip 并返回；否则在临时目录执行：

```rust
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
        .args(["--mlir-to-llvmir", mlir.to_str().unwrap(), "-o", ll.to_str().unwrap()])
        .status().unwrap().success());
    assert!(std::process::Command::new("llc")
        .args(["-filetype=obj", "-mtriple=x86_64-unknown-linux-gnu", ll.to_str().unwrap(), "-o", obj.to_str().unwrap()])
        .status().unwrap().success());
    assert!(std::process::Command::new("cc")
        .args([obj.to_str().unwrap(), "-o", exe.to_str().unwrap()])
        .status().unwrap().success());

    let status = std::process::Command::new(&exe).status().unwrap();
    assert_eq!(status.code(), Some(42));
}
```

- [ ] **Step 3: 安装/确认工具链**

Run:

```bash
command -v mlir-translate || sudo apt-get update && sudo apt-get install -y mlir-19-tools
export PATH="/usr/lib/llvm-19/bin:$PATH"
mlir-translate --version
llc --version | grep x86-64
```

Expected: `mlir-translate` 和 `llc` 可用。

- [ ] **Step 4: 运行契约测试**

Run: `cargo test --test mlir_toolchain -- --nocapture`
Expected: PASS，退出码 42。

- [ ] **Step 5: 提交**

```bash
git add tests/fixtures/minimal.mlir tests/mlir_toolchain.rs
git commit -m "test: add minimal MLIR toolchain contract"
```

---

### Task 2: C runtime 与值 ABI

**Files:**
- Create: `runtime/mita_runtime.h`
- Create: `runtime/mita_runtime.c`
- Create: `tests/runtime_test.rs`

**Interfaces:**
- Consumes: 无。
- Produces: `mita_value` 结构、构造器、算术/比较、`mita_cons/car/cdr`、`mita_print_value`、`mita_print`、`mita_make_closure`、`mita_call`、`mita_panic`。后续 codegen 只依赖这些 C 符号。

- [ ] **Step 1: 写失败测试**

`tests/runtime_test.rs`：生成一个临时 C harness，包含 `runtime/mita_runtime.h`，执行：

```c
mita_value a = mita_int(1);
mita_value b = mita_int(2);
mita_value sum = mita_add(a, b);
mita_print_value(sum);
mita_value pair = mita_cons(a, b);
mita_print_value(pair);
mita_print_value(mita_car(pair));
mita_print_value(mita_cdr(pair));
```

用 `cc -I runtime runtime/mita_runtime.c harness.c -o harness` 编译并运行，断言 stdout：

```text
3
(1 . 2)
1
2
```

测试在 `runtime/mita_runtime.c` 不存在时先失败。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --test runtime_test -- --nocapture`
Expected: FAIL（runtime 文件不存在）。

- [ ] **Step 3: 实现 header**

`runtime/mita_runtime.h`：定义 `mita_value`、tag 常量、所有 runtime 函数原型。

- [ ] **Step 4: 实现 runtime**

`runtime/mita_runtime.c`：

- `malloc` 分配 cons/string/symbol/closure；不释放。
- `mita_print_value` 对齐 VM：`da/nye/nil`、数字、字符串带引号、符号裸名、cons 递归（非列表尾部用 ` . `）。
- `mita_print` 参数间空格、结尾换行。
- `mita_eq` 结构相等；字符串/符号按内容。
- 除零、类型错误、arity 不匹配调用 `mita_panic` 并 abort。

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test --test runtime_test`
Expected: PASS。

- [ ] **Step 6: 提交**

```bash
git add runtime/mita_runtime.h runtime/mita_runtime.c tests/runtime_test.rs
git commit -m "feat: add native C runtime for MLIR backend"
```

---

### Task 3: 最小端到端 `mita build`

**Files:**
- Create: `src/backend/mod.rs`
- Create: `src/backend/mlir.rs`
- Create: `src/backend/build.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Create: `tests/mlir_codegen.rs`
- Create: `tests/native_build.rs`

**Interfaces:**
- Consumes: Task 2 的 runtime 符号。
- Produces:
  - `src/backend/mlir.rs`: `pub fn compile(exprs: &[Rc<Expr>]) -> Result<String, String>`
  - `src/backend/build.rs`:
    - `pub struct BuildOptions { pub target: String, pub output: PathBuf, pub emit_mlir: Option<PathBuf>, pub keep_temps: bool }`
    - `pub fn build_files(inputs: &[String], opts: &BuildOptions) -> Result<(), String>`
  - `mita build [--target <triple>] [--emit-mlir <path>] [--keep-temps] [-o <path>] <inputs...>`

- [ ] **Step 1: 写失败测试**

`tests/mlir_codegen.rs`：

```rust
use mita::Parser;

fn compile_source(source: &str) -> String {
    let mut parser = Parser::new(source);
    let mut exprs = Vec::new();
    loop {
        match parser.skip_space() {
            '\n' => continue,
            '\0' => break,
            _ => {}
        }
        exprs.push(parser.list());
    }
    mita::backend::mlir::compile(&exprs).unwrap()
}

#[test]
fn emits_main_and_add() {
    let mlir = compile_source("(print (celi 1 2))");
    assert!(mlir.contains("llvm.func @main"));
    assert!(mlir.contains("@mita_add"));
    assert!(mlir.contains("@mita_print"));
}
```

`tests/native_build.rs`：

```rust
#[test]
fn builds_and_runs_add_program() {
    if !tool_exists("mlir-translate") || !tool_exists("llc") { return; }
    let dir = tempdir();
    let src = dir.join("add.mita");
    std::fs::write(&src, "(print (celi 1 2))").unwrap();
    let exe = dir.join("add");
    build_files(&[src.display().to_string()], &BuildOptions {
        target: "x86_64-unknown-linux-gnu".into(),
        output: exe.clone(),
        emit_mlir: None,
        keep_temps: false,
    }).unwrap();
    let out = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\nnil\n");
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --test mlir_codegen --test native_build`
Expected: FAIL（`compile_source` / `build_files` 不存在）。

- [ ] **Step 3: 实现 MLIR 生成骨架**

`src/backend/mlir.rs`：

- 值类型 `!llvm.struct<(i64, i64)>`。
- 常量 helper：`nil`、`bool`、`int`、`string`、`symbol`。
- 调用 helper：`llvm.call @mita_xxx(...)`。
- 支持：数字、`celi`、`print`、顶层顺序执行、`main` 自动打印最后结果。
- `compile` 返回完整 `module { ... }` 文本。

- [ ] **Step 4: 实现构建管线**

`src/backend/build.rs`：

- `include_str!("../../runtime/mita_runtime.c")` / `.h`。
- 工具发现：`MLIR_TRANSLATE`、`LLC`、`CC` 环境变量 → PATH → `/usr/lib/llvm-*/bin/`。
- 临时目录、`mlir-translate --mlir-to-llvmir`、`llc -filetype=obj -mtriple=...`、目标 `cc` 编译 runtime、链接。
- `x86_64` 映射到 `cc`；其余目标在 Task 6 启用。
- 失败时保留 stderr 并返回错误。

- [ ] **Step 5: 接 CLI**

`src/main.rs` 在参数解析前检测 `build` 子命令，调用 `backend::build::run_cli(&args[2..])`；失败 `std::process::exit(1)`。

- [ ] **Step 6: 运行测试确认通过**

Run: `cargo test --test mlir_codegen --test native_build`
Expected: PASS。

- [ ] **Step 7: 全量测试**

Run: `cargo test`
Expected: 全部通过。

- [ ] **Step 8: 提交**

```bash
git add src/backend src/lib.rs src/main.rs tests/mlir_codegen.rs tests/native_build.rs
git commit -m "feat: add minimal MLIR native build pipeline"
```

---

### Task 4: 表达式、控制流与局部变量

**Files:**
- Modify: `src/backend/mlir.rs`
- Modify: `tests/mlir_codegen.rs`
- Modify: `tests/native_build.rs`

**Interfaces:**
- Consumes: Task 3 的 `compile` / `build_files`。
- Produces: MVP 的 `ka`、`dala`、`tido`、`in`、`plama`、quote、比较、列表操作、字符串。

- [ ] **Step 1: 写失败测试**

`tests/mlir_codegen.rs` 增加：

```rust
#[test]
fn emits_cond_branch_and_runtime_calls() {
    let mlir = compile_source("(ka (shato 1 1) 2 3)");
    assert!(mlir.contains("@mita_eq"));
    assert!(mlir.contains("llvm.cond_br"));
}

#[test]
fn emits_cons_and_list_calls() {
    let mlir = compile_source("(lawa (upa 1 2))");
    assert!(mlir.contains("@mita_cons"));
    assert!(mlir.contains("@mita_car"));
}
```

`tests/native_build.rs` 增加一个 `run_program` helper，并断言：

| 程序 | 期望 stdout |
|---|---|
| `(ka (shato 1 1) 10 20)` | `10` |
| `(tido ((x 1)) (plama x (celi x 1)) x)` | `2` |
| `(in 1 2 3)` | `3` |
| `(dala ((shato 1 2) 10) 20)` | `20`（最后 clause 是裸 atom / 隐式 else） |
| `(dala ((shato 1 2) 10) (celi 1 2))` | `3`（最后 test 为假时重新求值整个 clause） |
| `(dala ((shato 1 1) 10) 20)` | `10` |
| `(lawa (upa 1 2))` | `1` |
| `'(1 2)` | `(1 2)` |
| `(print "hi")` | `"hi"` 后跟 `nil` |

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --test mlir_codegen --test native_build`
Expected: FAIL（未实现的 form / runtime 调用缺失）。

- [ ] **Step 3: 实现控制流与变量**

- `ka`：`llvm.cond_br` + then/else/end block；结果写入 `alloca` 槽。
- `dala`：嵌套 `ka`，严格实现隐式 else、最后 clause 重新求值、裸 atom。
- `in`：顺序执行。
- `tido`：每个绑定分配 `alloca` 槽；参数也 alloca。
- `plama`：`llvm.store` 到已有局部槽；找不到变量返回编译错误。
- quote：数字/字符串/符号常量；列表递归 `mita_cons`。

- [ ] **Step 4: 实现比较、列表和字符串**

- `shato/nyeshato/aba/unta/abashato/untashato` → runtime。
- `upa/lawa/kucha` → runtime。
- 字符串全局常量和 `mita_string`；`print` → `mita_print`。

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test --test mlir_codegen --test native_build`
Expected: PASS。

- [ ] **Step 6: 全量测试并提交**

Run: `cargo test`
Expected: 全部通过。

```bash
git add src/backend/mlir.rs tests/mlir_codegen.rs tests/native_build.rs
git commit -m "feat: support expressions, control flow, and locals in MLIR backend"
```

---

### Task 5: 函数、递归、lambda 与 muhe

**Files:**
- Modify: `src/backend/mlir.rs`
- Modify: `tests/mlir_codegen.rs`
- Modify: `tests/native_build.rs`

**Interfaces:**
- Consumes: Task 4 的表达式/控制流。
- Produces: 固定参数函数、`muhe` 两阶段解析、直接调用、闭包创建与 `mita_call`、非捕获 lambda、递归/互递归。

- [ ] **Step 1: 写失败测试**

`tests/mlir_codegen.rs`：

```rust
#[test]
fn emits_function_declarations_before_bodies() {
    let mlir = compile_source("(muhe ((f (mita (x) x)))) (f 1)");
    assert!(mlir.contains("@f"));
    assert!(mlir.contains("llvm.func @f"));
}

#[test]
fn emits_closure_creation() {
    let mlir = compile_source("(tido ((f (mita (x) x))) (f 1))");
    assert!(mlir.contains("@mita_make_closure"));
    assert!(mlir.contains("@mita_call"));
}
```

`tests/native_build.rs`：

| 程序 | 期望 stdout |
|---|---|
| `examples/factorial.mita` | `120` |
| `examples/fibonacci.mita` | `5` |
| `((mita (f) (f 5)) (mita (x) (celi x x)))` | `10` |
| 互递归（见下） | `da` |
| 两个输入文件：`lib.mita` 定义 `(double x)`，`app.mita` 调用 `(double 21)` | `42` |

互递归测试程序：

```lisp
(muhe (
  (myeven (mita (n) (ka (shato n 0) da (myodd (movo n 1)))))
  (myodd (mita (n) (ka (shato n 0) nye (myeven (movo n 1)))))))
(myeven 10)
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --test mlir_codegen --test native_build`
Expected: FAIL（函数/lambda 未实现）。

- [ ] **Step 3: 两阶段 muhe**

扫描所有顶层 `muhe`，记录函数名/参数/body，先发出 `llvm.func` 声明，再逐个编译函数体。名字解析顺序：局部 → 内建 → 已知 `muhe` → 编译错误。

- [ ] **Step 4: 函数调用与参数槽**

- 编译期已知函数：`llvm.call @fn(args_ptr, argc)`。
- 参数在入口块 `alloca` 到 `mita_value` 槽；函数体从槽 load。
- 返回值：函数最后表达式结果。

- [ ] **Step 5: lambda 与闭包**

- `mita` 编译成独立函数，签名 `(ptr args, i64 argc) -> !llvm.struct<(i64,i64)>`。
- 创建闭包：`mita_make_closure(@lambda, arity)`。
- 调用闭包值：`mita_call(fn, args_ptr, argc)`。
- 非捕获；引用外层局部变量时报编译错误。

- [ ] **Step 6: 运行测试确认通过**

Run: `cargo test --test mlir_codegen --test native_build`
Expected: PASS。

- [ ] **Step 7: 全量测试并提交**

Run: `cargo test`
Expected: 全部通过。

```bash
git add src/backend/mlir.rs tests/mlir_codegen.rs tests/native_build.rs
git commit -m "feat: add functions, recursion, and lambdas to MLIR backend"
```

---

### Task 6: 三平台构建、CI 与文档

**Files:**
- Modify: `src/backend/build.rs`
- Modify: `tests/native_build.rs`
- Modify: `.github/workflows/mita-test.yml`
- Modify: `README.md`
- Modify: `AGENTS.md`

**Interfaces:**
- Consumes: Task 3-5 的构建管线与 codegen。
- Produces: `--target aarch64-unknown-linux-gnu` / `riscv64-unknown-linux-gnu` 构建，CI 安装工具并跑 qemu 验证。

- [ ] **Step 1: 写跨平台测试**

`tests/native_build.rs` 增加，只有 `MITA_TEST_CROSS=1` 时运行：

```rust
#[test]
fn builds_aarch64_and_runs_under_qemu() {
    if std::env::var("MITA_TEST_CROSS").as_deref() != Ok("1") { return; }
    // build (celi 20 22) for aarch64, run qemu-aarch64 -L /usr/aarch64-linux-gnu
    // assert stdout "42\n"
}
```

riscv64 同理，用 `qemu-riscv64 -L /usr/riscv64-linux-gnu`。

- [ ] **Step 2: 扩展目标映射**

`src/backend/build.rs`：

| triple | CC | qemu |
|---|---|---|
| x86_64 | `cc` | 本机 |
| aarch64 | `aarch64-linux-gnu-gcc` | `qemu-aarch64 -L /usr/aarch64-linux-gnu` |
| riscv64 | `riscv64-linux-gnu-gcc` | `qemu-riscv64 -L /usr/riscv64-linux-gnu` |

未支持 triple 返回明确错误。

- [ ] **Step 3: 更新 CI**

`.github/workflows/mita-test.yml`：

- `apt-get install -y mlir-19-tools qemu-user gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu`
- 将 `/usr/lib/llvm-19/bin` 加到 `GITHUB_PATH`
- 运行 `cargo test`
- 运行 `MITA_TEST_CROSS=1 cargo test --test native_build -- --nocapture`

- [ ] **Step 4: 更新文档**

README 增加：

```markdown
### Native builds

mita build --target x86_64-unknown-linux-gnu app.mita -o app
mita build --target aarch64-unknown-linux-gnu app.mita -o app-arm64
mita build --target riscv64-unknown-linux-gnu app.mita -o app-riscv64
```

AGENTS.md 增加 `src/backend/`、`runtime/`、`mita build` 和工具链说明。

- [ ] **Step 5: 三平台验证**

Run:

```bash
cargo test
MITA_TEST_CROSS=1 cargo test --test native_build -- --nocapture
```

Expected: x86_64 本机、aarch64 qemu、riscv64 qemu 都输出 `42`。

- [ ] **Step 6: 提交**

```bash
git add src/backend/build.rs tests/native_build.rs .github/workflows/mita-test.yml README.md AGENTS.md
git commit -m "feat: support aarch64 and riscv64 native builds"
```
