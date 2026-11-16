# MITA 丘丘语语义重命名设计

- 日期：2026-11-16
- 状态：已实施
- 范围：项目（破坏性重命名）

## 意图

让 MITA 的语言层名字尽量对齐丘丘语本义，而不是“丘丘语风味”。不再保留旧名字兼容层，所有 `.mita` 源码、示例、测试、文档一次性切换。

## 范围

- **改**：特殊形式、核心内建、`odomu.mita` 全部函数、语言层 `print`、类型谓词中的 `null/consp/atom`，以及所有 `.mita` 文件（包括 `bench/`）中的语言 token。
- **不改**：`sys_*`、`net.mita`、`http.mita` 的**公开函数名**（`net_*`、`http_*`、`url_decode`、`handle_connection`、`serve_loop`、`run_server`）、数学/字符串底层内建、`mite` FFI。这些是 runtime/协议层，丘丘语没有对应概念。
- `net.mita` / `http.mita` 的函数体仍然要跟随全局 token 重命名（例如 `sada`→`tiga`、`print`→`beru`、`tido`→`zido`、`mita`→`muhe`、`muhe`→`sada`），否则它们无法编译。
- 删除 tree-only 的 `list` / `apply` 内建：`list` 由 `odomu` 的 `tiga` 取代，`apply` 当前无使用且不属于 MVP；不保留英文重复入口。

## 命名原则

1. **本义优先**：`da/nye`、`unu/du`、`lawa`（头/王）、`kucha`（小/剩余）、`shato`（相似）、`aba/unta`（前/后）、`beru`（说）、`mosi`（吃）、`valo`（告别）、`mito`（知道）。
2. **隐喻其次**：`plata`（盾）→ quote，`mosi`（吃）→ reduce，`valo`（告别）→ filter。
3. **复合词最后**：`untasi`（after-thing）→ last，`elekasi`（at-thing）→ position，`tigakundala`（拆开一捆）→ flatten。
4. 避免强贬义词做中性操作；`pupu`（草/粪便）不再用于 flatten。

## 核心重命名表

| 旧名 | 新名 | 说明 |
|---|---|---|
| `mita` | `muhe` | lambda：want/desire，函数作为“愿望” |
| `muhe` | `sada` | defun：solidify，把名字做实 |
| `tido` | `zido` | let：here，绑定在此 |
| `plama` | `movo` | setq：move，把值移进变量 |
| `ka` | `ka` | if：exist?，保持不变 |
| `dala` | `dala` | cond：which?，保持不变 |
| `in` | `in` | progn，保持不变 |
| `plata` | `plata` | quote：盾牌保护，保持不变 |
| `celi` | `celi` | `+`：heat/fire，保持不变 |
| `movo` | `lata` | `-`：ice/cold，降温 |
| `celida` | `kuzi` | `*`：strengthen/grow |
| `movoda` | `gusha` | `/`：share/divide vegetables |
| `shato` | `shato` | `==`，保持不变 |
| `nyeshato` | `nyeshato` | `!=`，保持不变 |
| `aba` | `aba` | `<`：before |
| `unta` | `unta` | `>`：after |
| `abashato` | `abashato` | `<=` |
| `untashato` | `untashato` | `>=` |
| `upa` | `upa` | cons：combine/gather，保持不变 |
| `lawa` | `lawa` | car：king/head |
| `kucha` | `kucha` | cdr：small/rest |
| `print` | `beru` | speak |

## `odomu.mita` 重命名表

| 旧名 | 新名 | 说明 |
|---|---|---|
| `lawakucha` | `lawakucha` | car of cdr，复合词保留 |
| `lawakuchakucha` | `lawakuchakucha` | caddr，保留 |
| `kuchakucha` | `kuchakucha` | cddr，保留 |
| `sada` | `tiga` | list：一捆木棍/石头 |
| `tiga` | `tigamani` | length：用手/手指量一捆 |
| `si` | `si` | map：对每个 thing 操作 |
| `valo` | `valo` | filter：对不符合的 say bye |
| `mosi` | `mosi` | reduce：吃掉整个列表 |
| `tomo` | `tomo` | append：ally/join |
| `domu` | `domu` | reverse：dance/turn |
| `mito` | `mito` | assoc：know/查找 |
| `odomu` | `odomu` | member：friend/在不在 |
| `zido` | `untasi` | last：after-thing |
| `eleka` | `eleka` | nth：now/at index |
| `kuzi` | `kuzitomo` | and：strong-allies（避免和 `*` 冲突） |
| `todo` | `todo` | or：yield |
| `biat` | `biat` | not：hate/negate |
| `kundala` | `ika` | remove：enemy |
| `pupu` | `tigakundala` | flatten：拆开一捆 |
| `null` | `nyasi` | nothing-thing |
| `consp` | `upasi` | combined-thing |
| `atom` | `unusi` | one-thing |
| `listp` | `tigasi` | bundle-thing |
| `find` | `mitosi` | know-thing |
| `position` | `elekasi` | at-thing |
| `count` | `manisi` | hand-thing |
| `copy_list` | `sadasi` | solid-thing |
| `subseq` | `kuchasi` | rest-thing |
| `adjoin` | `tomosi` | join-thing |
| `union` | `tomotomo` | join-join |
| `intersection` | `lawatomo` | head-join / common |
| `set_difference` | `kundalasi` | take-thing |
| `subst` | `movosi` | move-thing |
| `every` | `tigatomo` | whole bundle |
| `some` | `tigakucha` | small bundle / at least one |
| `eq` | `shato` | 直接复用相等 |
| `eql` | `shato` | 直接复用相等 |
| `when` | `kasi` | if-thing |
| `unless` | `nyekasi` | not-if-thing |
| `prog1` | `inusi` | sequence-one |
| `prog2` | `indu` | sequence-two |

## 关键冲突与处理

- `kuzi` 同时被 `*` 和 logical `and` 需要。决定：`*` 保留 `kuzi`，`and` 用 `kuzitomo`（strong-allies）。`or` 继续用 `todo`。
- `mani` 是预定义数字 5，不能用作 length。length 用 `tigamani`；count 用 `manisi`。
- `zido` 从 last 改成 let；last 改用 `untasi`。
- `movo` 从 `-` 改成 setq；`-` 改用 `lata`。
- `sada` 从 list 改成 defun；list 改用 `tiga`。
- `tiga` 从 length 改成 list；length 改用 `tigamani`。
- `pupu` 不再使用；flatten 用 `tigakundala`。

## Builder check 决议

- `net.mita` / `http.mita` 只保留公开函数名，函数体照常 token 重命名。
- `null` / `consp` / `atom` 同时在 `odomu.mita`、`src/vm/builtins.rs` 的 `register_all()`、`src/compiler/mod.rs` 的 `builtin_index`、`src/backend/mlir.rs` 的内建分派中改名，不能只改 odomu。
- `sada` 不再表示“符号/原子”这个文档概念。符号/原子在文档中改称 `unusi`（one-thing / atom）；内部 Rust 方法 `Expr::get_sada()` 重命名为 `Expr::get_atom()`，避免名字误导。
- tree-only 的 `list` / `apply` 删除；`list` 由 `tiga` 取代，`apply` 无使用。
- `eq` / `eql` **删除定义**，统一使用 `shato`；不创建新别名。
- `tigakundala` / `kundalasi` 复用 `kundala` 词素是有意的：前者是“拆开一捆”，后者是“拿走某物”。命名审计按**完整 token** 扫描，不做子串匹配，因此不会误报。
- `bench/*.mita` 也在 token 重命名范围内。
- 用户可见错误信息里的旧名同步更新，例如 `malformed sada`、`movo: odd number of args`、`zido: no tiga param`、`sada: expected function name`。
- 算术隐喻补充：`celi`（火/升温）= `+`，`lata`（冰/降温）= `-`，`kuzi`（增强）= `*`，`gusha`（分菜/分享）= `/`；`movo`（移动）改作 setq。

## 不改的名字

- `da`、`nye`、`nya`、`unu`、`du`、`unudu`、`dudu`、`mani`。
- `sys_*`、`net_*`、`http_*`、`url_decode`、`handle_connection`、`serve_loop`、`run_server`。
- 数学/字符串/类型谓词中的 `mod/abs/max/min/floor/ceiling/truncate/round/expt/random/sqrt/zerop/plusp/minusp/evenp/oddp/numberp`。
- `mite` FFI。

## 文件影响

- `src/parser.rs`：特殊形式/关键字识别。
- `src/eval.rs`、`src/elementary.rs`：树解释器内建表。
- `src/compiler/mod.rs`：特殊形式分派、`builtin_index`。
- `src/vm/builtins.rs`：`register_all()` 和名称匹配。
- `src/backend/mlir.rs`：MLIR 后端的特殊形式、内建分派和 `DECLS`。
- `odomu.mita`：全部函数定义。
- `examples/*.mita`、`bench/*.mita`、`tests/*.rs`、`README.md`、`AGENTS.md`。
- `net.mita` / `http.mita`：公开函数名保持不变，函数体 token 重命名。
- `src/expr.rs`：`get_sada()` → `get_atom()`（内部可读性）。
- 用户可见错误信息字符串。

## 迁移方式

- 不允许顺序 `sed`：存在 `mita→muhe`、`muhe→sada`、`sada→tiga`、`tiga→tigamani`、`tido→zido`、`zido→untasi`、`plama→movo`、`movo→lata`、`celida→kuzi`、`kuzi→kuzitomo` 等链式替换。
- 使用**同时替换**的 token 映射：对 `.mita` 按符号 token 替换；对 Rust 只替换字符串字面量/名称映射，不改 Rust 函数名（`get_sada` 的改名是明确的内部重命名）。
- 先改核心 + 编译器/VM/native 后端，再改 `odomu.mita`，最后改 examples/bench/tests/docs。
- 新增一个命名审计测试：按完整 token 扫描所有 `.mita` 和关键 Rust 字符串，断言旧名不再作为用户可见符号出现（排除历史文档和 `target/`）；不做子串匹配，避免 `tigakundala` / `kundalasi` 误报。

## 非目标

- 不改 `sys_*`、net/http、数学/字符串底层名字。
- 不提供旧名字别名或迁移工具。
- 不改变任何运行时语义，只改名字。

## 验证

- `cargo test` 全部通过。
- `./target/release/mita odomu.mita examples/odomu_test.mita` 通过。
- 全部非服务端示例通过；两个 web server 示例用 curl 验证。
- `MITA_TEST_CROSS=1 cargo test --test native_build` 通过（native MLIR 后端仍可用新名字）。
- 命名审计测试通过，旧名不再出现。
- `README.md` / `AGENTS.md` 中的语言表和示例全部使用新名。
