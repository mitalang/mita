# MITA 丘丘语语义重命名 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 MITA 的语言层名字全量切换为语义对齐的丘丘语命名，删除旧名，不保留兼容层。

**Architecture:** 使用设计文档中的“同时替换”token 映射，一次改完 `.mita` 符号和 Rust 字符串字面量；先加命名审计测试（RED），再执行映射（GREEN），最后同步 README/AGENTS 和命名说明。

**Tech Stack:** Rust、MITA 解析器/VM/MLIR 后端、Python 一次性映射脚本（不提交）。

**Spec:** `docs/superpowers/specs/2026-11-16-hilichurlian-semantics-design.md`

## Global Constraints

- 不保留旧名字别名；不提供迁移工具。
- 不改变运行时语义，只改名字。
- 语言层全量改；`sys_*`、`net_*`、`http_*` 公开函数名、底层数学/字符串内建、`mite` 保持不变。
- `net.mita` / `http.mita` 的函数体仍要跟随 token 重命名。
- 使用同时替换；禁止顺序 `sed`，因为存在 `mita→muhe`、`muhe→sada`、`sada→tiga`、`tiga→tigamani`、`tido→zido`、`zido→untasi`、`plama→movo`、`movo→lata`、`celida→kuzi`、`kuzi→kuzitomo` 等链。
- 命名审计按完整 token，不做子串匹配；`tigakundala` / `kundalasi` 合法。
- 不新增 Rust crate 依赖。
- 分支：`hilichurlian-semantics`，从 `mlir-native-mvp` 的 `2bce382` 叠加。

## Review Focus

- 链式替换是否真的同时完成；不能出现 `muhe` 被连续替换成 `sada` 或 `mita` 残留。
- `net.mita` / `http.mita` 公开名是否保持不变，但函数体仍能编译。
- `null` / `consp` / `atom` 是否在 odomu、VM builtins、compiler `builtin_index`、MLIR 后端四处同步改名。
- tree-only `list` / `apply` 是否删除，`eq` / `eql` 是否删除而不是留下别名。
- native MLIR 后端是否仍能编译并运行；三种目标测试不能只跑 host。
- `bench/`、`examples/`、`tests/`、`README.md`、`AGENTS.md` 是否全部使用新名，且命名审计无残留。

---

### Task 1: 执行全量 token 重命名

**Files:**
- Create: `tests/naming_test.rs`
- Modify: `odomu.mita`
- Modify: `net.mita`
- Modify: `http.mita`
- Modify: `src/eval.rs`
- Modify: `src/elementary.rs`
- Modify: `src/expr.rs`
- Modify: `src/compiler/mod.rs`
- Modify: `src/vm/builtins.rs`
- Modify: `src/backend/mlir.rs`
- Modify: `examples/*.mita`
- Modify: `bench/*.mita`
- Modify: `tests/*.rs`

**Interfaces:**
- Consumes: 设计文档中的完整旧名→新名表。
- Produces: 所有用户可见 MITA 名字切换完成；`tests/naming_test.rs` 通过。

- [x] **Step 1: 写命名审计测试（RED）**

创建 `tests/naming_test.rs`。它应：

1. 读取 `odomu.mita`、`net.mita`、`http.mita`、`examples/*.mita`、`bench/*.mita`。
2. 用 `mita::Parser` 解析，遍历 `Expr` 收集所有 atom 符号（不执行）。
3. 断言这些已删除的旧名不再作为完整符号出现：
   `tido`, `plama`, `pupu`, `celida`, `movoda`, `eq`, `eql`, `consp`, `atom`, `null`, `list`, `apply`。
4. 断言 `odomu.mita` 中出现了新名：
   `tiga`, `tigamani`, `kuzitomo`, `tigakundala`, `untasi`, `tigasi`, `upasi`, `unusi`, `nyasi`。
5. 对 `src/eval.rs`、`src/elementary.rs`、`src/compiler/mod.rs`、`src/vm/builtins.rs`、`src/backend/mlir.rs` 做简单的引号字符串扫描，断言 `"tido"`、`"plama"`、`"pupu"`、`"celida"`、`"movoda"` 不再出现。

Run: `cargo test --test naming_test`
Expected: FAIL，旧名仍在。

- [x] **Step 2: 生成同时替换映射**

在 `/tmp/opencode` 写一次性 Python 脚本（不提交）。映射表直接取设计文档的“核心重命名表”和“`odomu.mita` 重命名表”。

要求：

- 对 `.mita` 文件：用 `;` 和 `"` 识别注释/字符串，只替换字符串和注释之外的**完整符号 token**。
- 对 Rust 文件：只替换双引号字符串字面量内部的完整单词 token；不改 Rust 标识符。
- **Rust 字符串的安全规则**：跳过包含 `.mita`、`/`、`\\`、目标 triple 的路径字符串；只替换“恰好等于旧名”的字符串，或包含 `(` / `)` 的 MITA 源码字符串。错误信息（如 `malformed muhe`）在 Step 3 手工改。
- 一次正则 alternation + 回调映射，保证同时替换，不按顺序 `sed`。
- 不要替换 `sys_*`、`net_*`、`http_*`、`mite`、数学/字符串内建。
- 不要替换语言名 `MITA` / crate 名 `mita` / 文件名（`*.mita` 路径）。

- [x] **Step 3: 应用映射到 `.mita` 和 Rust**

覆盖：

- `odomu.mita`、`net.mita`、`http.mita`
- `examples/*.mita`、`bench/*.mita`
- `tests/*.rs` 中的 MITA 源码字符串和期望输出
- `src/eval.rs`、`src/elementary.rs`、`src/compiler/mod.rs`、`src/vm/builtins.rs`、`src/backend/mlir.rs` 中的关键字/内建字符串

额外手工修改：

- `src/expr.rs`：`get_sada` → `get_atom`，同步所有调用点。
- `src/elementary.rs`：删除 tree-only 的 `list` / `apply` 注册和实现（如果无其他引用）。
- `tests/eval_tests.rs`：把 `(list 'a 'b 'c)` 改成 `(upa 'a (upa 'b (upa 'c '())))`，不要保留已删除的 `list` 内建。
- `odomu.mita`：删除 `eq` / `eql` 定义。
- 更新 Rust 错误信息里的旧名，例如 `malformed sada`、`movo: odd number of args`、`zido: no tiga param`、`sada: expected function name`。

- [x] **Step 4: 运行命名审计（GREEN）**

Run: `cargo test --test naming_test`
Expected: PASS。

- [x] **Step 5: 修编译/测试失败**

Run: `PATH=/usr/lib/llvm-19/bin:$PATH cargo test`
Expected: 全部通过。重点检查：

- 链式替换是否把 `muhe` 误改成 `sada` 或把新 `muhe` 改回 `mita`。
- `net.mita` / `http.mita` 是否仍能加载。
- `tests/eval_tests.rs` / `vm_tests.rs` / `library_test.rs` / `native_build.rs` / `mlir_codegen.rs` 的字符串是否同步。
- native 后端 `builtin_for` / `DECLS` / 特殊形式是否使用新名。

- [x] **Step 6: 运行集成与 native 测试**

Run:

```bash
PATH=/usr/lib/llvm-19/bin:$PATH cargo build --release
./target/release/mita odomu.mita examples/odomu_test.mita
PATH=/usr/lib/llvm-19/bin:$PATH MITA_TEST_CROSS=1 cargo test --test native_build -- --nocapture
```

Expected: 全部退出码 0；aarch64/riscv64 qemu 输出正确。

- [x] **Step 7: 提交**

```bash
git add tests/naming_test.rs odomu.mita net.mita http.mita src examples bench tests
git commit -m "refactor: rename MITA language surface to Hilichurlian semantics"
```

---

### Task 2: 文档、命名说明与最终验证

**Files:**
- Create: `docs/hilichurlian-naming.md`
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/superpowers/specs/2026-11-16-hilichurlian-semantics-design.md`（状态改为“已实施”）
- Modify: `docs/superpowers/plans/2026-11-16-hilichurlian-semantics.md`（勾选完成项）

**Interfaces:**
- Consumes: Task 1 完成的新名字。
- Produces: 文档、命名说明与实际代码一致。

- [x] **Step 1: 写命名说明**

创建 `docs/hilichurlian-naming.md`，包含：

- 本义 / MITA 义 / 备注的完整表（核心 + odomu）。
- 链式改名说明（`mita→muhe` 等）。
- 为什么 `net/http/sys_*` 保留英文。
- 复合词构造规则（`tiga` + `kundala` = `tigakundala`）。
- 已删除名字列表。

- [x] **Step 2: 更新 README**

- “sada symbol” 改为 `unusi` / atom 的表述。
- 内建函数表、`odomu` 函数表全部使用新名。
- 示例代码块使用新关键字（`sada` defun、`muhe` lambda、`zido` let、`movo` setq、`beru` print）。
- Native build 示例不变（命令本身不含语言名）。

- [x] **Step 3: 更新 AGENTS.md**

- 特殊形式列表：`muhe` lambda、`sada` defun、`zido` let、`movo` setq、`ka` if、`dala` cond、`in` progn、`plata` quote。
- 内建/库函数表全部换新名。
- `NOTES` 增加：`sada` 现在是 defun，不再表示 symbol；`unusi` 表示 atom/symbol。
- 更新任何旧名示例/命令。

- [x] **Step 4: 更新设计状态**

把 `docs/superpowers/specs/2026-11-16-hilichurlian-semantics-design.md` 顶部状态改为 `已实施`。

- [x] **Step 5: 最终验证**

Run:

```bash
PATH=/usr/lib/llvm-19/bin:$PATH cargo test
./target/release/mita odomu.mita examples/odomu_test.mita
PATH=/usr/lib/llvm-19/bin:$PATH MITA_TEST_CROSS=1 cargo test --test native_build -- --nocapture
```

再跑全部非服务端示例和 `bench/*.mita`：

```bash
for f in examples/add.mita examples/car.mita examples/cdr.mita examples/compare.mita examples/cond.mita examples/defn.mita examples/factorial.mita examples/fibonacci.mita examples/hello.mita examples/higher-order.mita examples/if.mita examples/lambda.mita examples/let.mita examples/list.mita examples/progn.mita examples/quicksort.mita examples/setq.mita examples/odomu_test.mita bench/fak_tco.mita bench/countdown.mita; do
  ./target/release/mita odomu.mita "$f" >/dev/null || exit 1
done
```

Expected: 全部退出码 0。

再验证两个 web server 示例（使用新的语言 token，公开 net/http 名不变）：

```bash
timeout 10 ./target/release/mita odomu.mita net.mita http.mita examples/web_server_test.mita >/tmp/opencode/web_test.log 2>&1 &
pid=$!
for i in $(seq 1 20); do body=$(curl -sS http://127.0.0.1:18080/ 2>/dev/null) && break; sleep 0.1; done
printf '%s\n' "$body"
kill $pid 2>/dev/null || true
wait $pid 2>/dev/null || true
```

Expected: 输出 `Hello from MITA`。`web_server_test2.mita` 同理。

- [x] **Step 6: 提交**

```bash
git add README.md AGENTS.md docs/hilichurlian-naming.md docs/superpowers/specs/2026-11-16-hilichurlian-semantics-design.md docs/superpowers/plans/2026-11-16-hilichurlian-semantics.md
git commit -m "docs: document Hilichurlian naming and update guides"
```
