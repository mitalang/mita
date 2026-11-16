# MITA MLIR Native MVP 设计

- 日期：2026-11-16
- 状态：已批准
- 范围：项目（新增 native 后端）

## 意图

让 MITA 程序能编译成真正的目标平台可执行文件，而不是只由字节码 VM 解释：

```bash
mita build --target x86_64-unknown-linux-gnu app.mita -o app
mita build --target aarch64-unknown-linux-gnu app.mita -o app-arm64
mita build --target riscv64-unknown-linux-gnu app.mita -o app-riscv64
```

`mita build` 直接生成 MLIR，使用 MLIR/LLVM 工具链 lower 到 LLVM IR，再生成目标文件和 ELF。物理寄存器分配与 spill 由 LLVM 负责，不手写 native spill。

## 目标

- 打通 `MITA source → MLIR → LLVM IR → object → ELF`。
- x86-64 本机可运行；aarch64 / riscv64 可通过交叉工具链构建并用 qemu 验证。
- MVP 支持足够的语言子集，能编译并运行 `examples/add.mita`、`examples/factorial.mita`、`examples/fibonacci.mita` 这类自包含程序。
- 新增自动化测试，覆盖 MLIR 生成、本机构建运行、交叉构建和 qemu 运行。
- 运行时用 C 实现，随 `mita` 二进制内嵌，构建时释放到临时目录，避免安装路径依赖。

## 非目标

- 不实现 VM 解释器的 32 寄存器 spill。
- 不实现闭包上值捕获、变参 lambda、`mite` FFI、socket/HTTP、完整 `odomu.mita` 标准库。
- 不实现组合 car/cdr 选择器（`lawakucha`、`lalalakukucha` 等）；MVP 只用 `lawa`/`kucha`。
- 不实现 GC；MVP 使用 `malloc`，允许泄漏。
- 不实现尾调用优化；深递归可能栈溢出。
- 不直接生成三套汇编；MLIR/LLVM 负责指令选择和寄存器分配。
- 不支持 macOS/Windows 目标，只做 Linux ELF。

## 约束

- 使用 LLVM/MLIR 19 工具链：`mlir-translate`、`llc`、目标 `cc`。
- 当前环境有 `clang`/`llc`/`ld.lld`、`aarch64-linux-gnu-gcc`、`riscv64-linux-gnu-gcc`、`qemu-aarch64`、`qemu-riscv64`；`mlir-19-tools` 需要安装。
- 构建时允许调用外部工具，不新增 Rust 依赖。
- 目标 triple 只接受：
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `riscv64-unknown-linux-gnu`

## 方案

考虑过三种路线：

- **A. 打包字节码 + VM runtime**：最终 ELF 内嵌 VM，构建简单，但仍是解释执行，不满足“原生机器码”。
- **B. 生成 MLIR，LLVM lower 成机器码（已选）**：真正的 native 后端；LLVM 自动做寄存器分配和 spill；三平台共用一套 MLIR 生成逻辑。
- **C. 手写三套汇编后端**：控制和调试成本最高，不适合当前阶段。

选 B。MVP 直接生成 **MLIR LLVM dialect**，因为它是 MLIR 中离最终 LLVM IR 最近、控制最直接的形态；后续可以再引入自定义 `mita` dialect 和优化 pass。

## 设计

### 值表示与 C ABI

运行时值用两个 `i64`：

```c
typedef struct {
    int64_t tag;
    int64_t payload;
} mita_value;
```

Tag：

| tag | 类型 | payload |
|---:|---|---|
| 0 | nil | 0 |
| 1 | bool | 0/1 |
| 2 | int | i64 |
| 3 | string | 指向 `{ int64_t len; char data[]; }` |
| 4 | symbol | 同 string |
| 5 | cons | 指向 `{ mita_value car; mita_value cdr; }` |
| 6 | closure | 指向 `{ mita_fn fn; int64_t arity; }` |

MLIR 中所有 MITA 值类型为：

```mlir
!llvm.struct<(i64, i64)>
```

C 结构体 `{i64, i64}` 在三种目标 ABI 下与 MLIR/LLVM 的 struct 传参/返回兼容（x86-64 用寄存器对，AArch64/RISC-V 同样按 16 字节 struct 处理）。这是 MVP 的核心 ABI 约定，必须在三平台测试中验证。

### Runtime API

新增 `runtime/mita_runtime.c`，通过 `include_str!` 内嵌到 `mita` 二进制。核心函数：

```c
mita_value mita_nil(void);
mita_value mita_bool(int64_t v);
mita_value mita_int(int64_t v);
mita_value mita_string(const char* data, int64_t len);
mita_value mita_symbol(const char* data, int64_t len);
mita_value mita_cons(mita_value car, mita_value cdr);
mita_value mita_car(mita_value v);
mita_value mita_cdr(mita_value v);

mita_value mita_add(mita_value a, mita_value b);
mita_value mita_sub(mita_value a, mita_value b);
mita_value mita_mul(mita_value a, mita_value b);
mita_value mita_div(mita_value a, mita_value b);
mita_value mita_eq(mita_value a, mita_value b);
mita_value mita_ne(mita_value a, mita_value b);
mita_value mita_lt(mita_value a, mita_value b);
mita_value mita_gt(mita_value a, mita_value b);
mita_value mita_le(mita_value a, mita_value b);
mita_value mita_ge(mita_value a, mita_value b);

int64_t mita_truthy(mita_value v);
mita_value mita_make_closure(void* fn, int64_t arity);
mita_value mita_call(mita_value fn, mita_value* args, int64_t argc);
mita_value mita_print(mita_value* args, int64_t argc);
mita_value mita_print_value(mita_value v);
void mita_panic(const char* msg);
```

行为对齐现有 VM：

- 只有 `tag=bool, payload=1` 为真；其他都假。
- `lawa`/`kucha` 对非 cons 返回 `nil`。
- `shato` 做结构相等；字符串/符号按内容比较。
- `print` 按 VM 的格式输出，参数间空格，结尾换行。
- 除零、类型错误、闭包 arity 不匹配调用 `mita_panic` 并 `abort()`。
- `malloc` 失败直接 abort；不实现 GC。

### MLIR 代码生成

新增 `src/backend/mlir.rs`。

- 每个 MITA 函数编译为：
  ```mlir
  llvm.func @fn(!llvm.ptr, i64) -> !llvm.struct<(i64, i64)>
  ```
  参数分别是 `mita_value* args` 和 `argc`。
- `main`：
  ```mlir
  llvm.func @main() -> i32
  ```
  顶层表达式直接在 `main` 中执行；**最后一个顶层表达式的结果会像 CLI `load()` 一样打印**，然后返回 0。这样 `factorial.mita`、`add.mita` 这类没有显式 `print` 的程序仍可被测试断言。
- 调用约定：
  - 名字解析顺序：`tido`/参数局部 → 内建 → 已知 `muhe` 函数 → 编译错误。
  - `muhe` 采用**先声明后编译**：先收集所有顶层 `muhe` 名字并发出 `llvm.func` 声明，再编译函数体。因此支持自递归、互递归和前向引用。
  - 编译期已知的函数（`muhe` 名字）：直接 `llvm.call @fn`。
  - 运行时闭包：`mita_call(fn, args_ptr, argc)`。
  - 参数数组在函数入口块的 `llvm.alloca` 分配，避免循环内反复增长栈。
- `mita` lambda：编译成独立 `llvm.func`，用 `mita_make_closure(@lambda, arity)` 创建闭包值。
- `muhe`：顶层定义只生成函数；作为值使用时生成闭包。
- 控制流：
  - `ka` → `llvm.cond_br` + merge block；then/else 结果写入一个 `alloca` 结果槽，merge block 再 load，避免依赖 block argument。
  - `dala` → 展开成嵌套 `ka`，语义严格对齐现有 VM：
    - 最后一个 clause 是隐式 else；
    - 最后 clause 的 test 为假时，把整个 clause 当作表达式重新求值；
    - 裸 atom clause 直接求值。
  - `in` → 顺序执行，最后一个值作为结果。
- 局部变量：
  - 所有 `tido` 绑定和函数参数都分配 `llvm.alloca` 槽位，读写用 `llvm.store/load`；这样 `plama` 可以直接修改局部变量，不需要额外的可变性分析。
  - `plama` 对标 VM 行为：只修改当前函数作用域内的参数或 `tido` 绑定；找不到局部变量时编译报错。
  - 不支持上值捕获；lambda 引用外层局部变量时编译报错。
- `plata`：
  - 数字/字符串/符号 → 常量。
  - 列表 → 运行时递归 `mita_cons` 构造。
- 内建函数映射：
  - `celi/movo/celida/movoda` → `mita_add/sub/mul/div`
  - `shato/nyeshato/aba/unta/abashato/untashato` → `mita_eq/ne/lt/gt/le/ge`
  - `upa/lawa/kucha` → `mita_cons/car/cdr`
  - `print` → `mita_print`
- 字符串常量用 `llvm.mlir.global internal constant` + `llvm.mlir.addressof`，再调用 `mita_string` / `mita_symbol`。

### 支持的 MVP 语法

- 字面量：i64 数字、字符串、quote 符号、`da/nye/nya/unu/du/unudu/dudu/mani`。
- 特殊形式：`ka`、`dala`、`tido`、`in`、`plama`、`mita`、`muhe`、`plata`。
- 内建：`celi`、`movo`、`celida`、`movoda`、`shato`、`nyeshato`、`aba`、`unta`、`abashato`、`untashato`、`upa`、`lawa`、`kucha`、`print`。
- 顶层表达式、固定参数函数、递归、非捕获 lambda、通过闭包值的高阶调用。
- 多个输入文件按顺序解析、合并顶层；不支持运行时 `(load ...)`。

### 构建流程

`mita build` 的流程：

1. 解析输入文件，合并为顶层表达式列表。
2. `src/backend/mlir.rs` 生成 MLIR 文本。
3. 若 `--emit-mlir <path>`，写出 MLIR。
4. `mlir-translate --mlir-to-llvmir` → LLVM IR。
5. `llc -O2 -filetype=obj -mtriple=<triple> -relocation-model=pic` → 目标对象。
6. 把内嵌的 `runtime/mita_runtime.c` 写入临时目录，用目标 `cc` 编译成 runtime 对象。
7. 用目标 `cc` 链接成 ELF。
8. 清理临时文件（`--keep-temps` 可保留）。

目标映射：

| triple | `llc` triple | CC |
|---|---|---|
| `x86_64-unknown-linux-gnu` | `x86_64-unknown-linux-gnu` | `cc` |
| `aarch64-unknown-linux-gnu` | `aarch64-unknown-linux-gnu` | `aarch64-linux-gnu-gcc` |
| `riscv64-unknown-linux-gnu` | `riscv64-unknown-linux-gnu` | `riscv64-linux-gnu-gcc` |

工具发现顺序：

- `MLIR_TRANSLATE` / `LLC` / `CC` 环境变量。
- `PATH` 中的 `mlir-translate`、`llc`、目标 cc。
- `/usr/lib/llvm-*/bin/` 下的 `mlir-translate`、`llc`。

找不到工具时输出安装提示（如 `apt install mlir-19-tools`），退出码非 0。

### 错误处理

- 解析/代码生成错误：报告文件名和表达式，退出码非 0。
- 外部工具失败：保留完整 stderr，退出码非 0；`--keep-temps` 时可检查中间文件。
- 运行时不匹配：`mita_panic` 打印消息并 abort，行为与 VM panic 一致。

## 风险

- **MLIR 文本语法与版本差异**：固定 LLVM/MLIR 19；实现第一步先做 toolchain spike，验证 `llvm.func`/`llvm.call`/`llvm.mlir.global`/函数指针的准确语法。
- **C struct ABI 与 MLIR struct 不一致**：三平台都必须用真实运行验证；如果某平台不匹配，退化为显式的两个 i64 参数/返回值函数。
- **函数指针地址**：如果 `llvm.mlir.addressof` 不能直接取函数地址，改为生成一张函数指针表或 dispatch 函数，不改语言语义。
- **无 GC**：长时间运行会泄漏；MVP 明确接受，后续替换为 arena/GC。
- **无 TCO**：深递归可能栈溢出；MVP 接受。
- **交叉链接 sysroot**：优先使用交叉 gcc；如果目标 libc 缺失，构建报错并提示安装交叉工具链。

## 验证

- `cargo test` 全部通过（现有 65 项 + 新增 MLIR/codegen 测试）。
- MLIR 文本测试：`(celi 1 2)` 生成包含 `mita_add`、`llvm.func`、`llvm.call` 的输出。
- x86-64 构建测试：编译 `examples/factorial.mita` 或临时程序，运行并断言输出。
- aarch64 / riscv64：交叉构建后用 `qemu-aarch64` / `qemu-riscv64` 运行，断言输出。
- 至少验证一个递归程序、一个字符串/print 程序、一个 cons/列表程序（用 `examples/hello.mita` 或显式 `print` 的程序；`list.mita` 的组合选择器不在 MVP 内）。
- CI 增加 MLIR 工具安装、交叉构建和 qemu 运行步骤；没有工具时本机测试给出清晰 skip 原因。
