# odomu 网络拆分 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 `odomu.mita` 中的网络函数拆到独立的 `net.mita`，让 `odomu.mita` 只保留通用 Lisp 函数。

**Architecture:** 新建一个只包含 `sys_*` 薄封装的 `net.mita`；从 `odomu.mita` 删除网络段和已失效的 `serve_once`；示例改为按 `odomu.mita → net.mita → http.mita` 顺序加载，并同步文档与 CI。

**Tech Stack:** Rust + MITA 字节码 VM、现有 `sys_*` 网络内建函数、GitHub Actions。

**Spec:** `docs/superpowers/specs/2026-11-15-odomu-net-split-design.md`

## Global Constraints

- 不实现 `(load ...)`、`import` 或任何模块系统。
- 不把通用库继续拆成 `core.mita` / `list.mita`。
- 不修改 VM 或编译器。
- 不处理闭包上值捕获。
- `net.mita` 依赖 `odomu.mita` 的 `lawakucha`，加载顺序固定为 `odomu.mita → net.mita → http.mita`。
- `serve_once` 直接删除，不迁移到 `net.mita`。

## Review Focus

- `net.mita` 单独加载会因缺少 `lawakucha` 失败；文档和示例必须始终先加载 `odomu.mita`。
- 示例固定监听 `18080`；验证前要确认端口空闲，避免把“端口占用”误判为代码失败。
- `serve_once` 删除是破坏性变更；旧命令 `mita odomu.mita examples/web_server_test.mita` 必须更新为新的多文件加载方式。
- HTTP handler 收到的 request 形状是 `(method path query version headers body)`；示例必须用 `lawakucha` 取 path，不能再用旧的 `lawa`。
- CI 使用 release 二进制；新增的 `net.mita` 必须真实存在并能和另外两个库一起加载。

---

### Task 1: 拆分 net.mita 并加回归测试

**Files:**
- Create: `tests/library_test.rs`
- Create: `net.mita`
- Modify: `odomu.mita:213-250`

**Interfaces:**
- Consumes: `VM::new()`, `mita::vm::builtins::register_all`, `Compiler::compile_toplevel`, `VM::define_global`, `VM::run`。
- Produces: `net.mita` 中的 `net_listen`、`net_accept`、`net_read`、`net_write`、`net_close`；`odomu.mita` 不再包含任何 `net_*` 或 `serve_once` 定义。

- [ ] **Step 1: 写失败测试**

创建 `tests/library_test.rs`：

```rust
use std::rc::Rc;

use mita::compiler::Compiler;
use mita::vm::value::Closure;
use mita::vm::VM;
use mita::Parser;

fn parse_source(source: &str) -> Vec<Rc<mita::Expr>> {
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
    exprs
}

fn load_file(vm: &mut VM, path: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    let exprs = parse_source(&source);
    let (func, globals) = Compiler::compile_toplevel(&exprs);
    for (name, val) in globals {
        vm.define_global(&name, val);
    }
    vm.run(Rc::new(Closure {
        func: Rc::new(func),
        upvalues: Vec::new(),
    }));
}

fn eval(vm: &mut VM, source: &str) -> String {
    let exprs = parse_source(source);
    let (func, _) = Compiler::compile_toplevel(&exprs);
    let result = vm.run(Rc::new(Closure {
        func: Rc::new(func),
        upvalues: Vec::new(),
    }));
    format!("{}", result)
}

fn new_vm() -> VM {
    let mut vm = VM::new();
    mita::vm::builtins::register_all(&mut vm);
    vm
}

#[test]
fn odomu_excludes_network_functions() {
    let source = std::fs::read_to_string("odomu.mita").unwrap();
    for name in [
        "net_listen",
        "net_accept",
        "net_read",
        "net_write",
        "net_close",
        "serve_once",
    ] {
        assert!(
            !source.contains(&format!("({}", name)),
            "odomu.mita still defines {}",
            name
        );
    }
}

#[test]
fn net_library_loads_after_odomu() {
    let mut vm = new_vm();
    load_file(&mut vm, "odomu.mita");
    load_file(&mut vm, "net.mita");
    assert_eq!(eval(&mut vm, "(net_close 999)"), "nil");
}

#[test]
fn libraries_load_together() {
    let mut vm = new_vm();
    load_file(&mut vm, "odomu.mita");
    load_file(&mut vm, "net.mita");
    load_file(&mut vm, "http.mita");
    assert_eq!(eval(&mut vm, "(net_close 999)"), "nil");
}
```

- [ ] **Step 2: 运行测试，确认失败**

Run: `cargo test --test library_test`
Expected: FAIL —— `odomu.mita still defines net_listen`，以及 `net.mita` 文件不存在。

- [ ] **Step 3: 创建 net.mita**

创建 `net.mita`：

```lisp
; net.mita — MITA socket primitives

(muhe (
  ; net_listen: create socket and listen on host:port, returns fd
  (net_listen (mita args
    (dala ((shato (kucha args) nya)
      (sys_listen (lawa args)))
      (da (sys_listen (lawakucha args) (lawa args))))))

  ; net_accept: accept connection, returns client fd
  (net_accept (mita (listen_fd)
    (sys_accept listen_fd)))

  ; net_read: read data from socket, returns string
  (net_read (mita (fd)
    (sys_read fd)))

  ; net_write: write string data to socket, returns bytes written
  (net_write (mita (fd data)
    (sys_write fd data)))

  ; net_close: close socket
  (net_close (mita (fd)
    (sys_close fd)))
))
```

- [ ] **Step 4: 从 odomu.mita 删除网络段**

删除 `odomu.mita` 中从 `; ===== Network Primitives =====` 到文件末尾 `serve_once` 定义之前的全部内容，保留最后的 `))`。删除范围包括 `net_listen`、`net_accept`、`net_read`、`net_write`、`net_close`、`serve_once`。

- [ ] **Step 5: 运行测试，确认通过**

Run: `cargo test --test library_test`
Expected: PASS（3 tests）

- [ ] **Step 6: 跑全量测试**

Run: `cargo test`
Expected: 全部通过，0 failed。

- [ ] **Step 7: 提交**

```bash
git add tests/library_test.rs net.mita odomu.mita
git commit -m "refactor: split network primitives out of odomu"
```

---

### Task 2: 更新 web server 示例

**Files:**
- Modify: `examples/web_server_test.mita`
- Modify: `examples/web_server_test2.mita`

**Interfaces:**
- Consumes: Task 1 的 `net_listen`、`net_accept`、`net_read`、`net_write`、`net_close`；`http.mita` 的 `http_ok`、`http_not_found`、`handle_connection`；`odomu.mita` 的 `sada`、`lawakucha`。
- Produces: 两个可用新加载方式运行的示例：
  - `mita odomu.mita net.mita http.mita examples/web_server_test.mita`
  - `mita odomu.mita net.mita http.mita examples/web_server_test2.mita`

- [ ] **Step 1: 重写 web_server_test.mita**

将文件内容替换为：

```lisp
(muhe ((test_handler (mita (req)
  (tido ((path (lawakucha req)))
    (dala ((shato path "/") (http_ok "Hello from MITA"))
      (da (http_not_found "404"))))))))

(tido ((fd (net_listen 18080)))
  (dala ((aba fd 0) nye)
    (da (in
      (print (sada "fd" fd))
      (handle_connection fd test_handler)
      (net_close fd)
      da))))
```

- [ ] **Step 2: 重写 web_server_test2.mita**

将文件内容替换为：

```lisp
(muhe (
  (test_handler (mita (req)
    (tido ((path (lawakucha req)))
      (dala ((shato path "/") "Hello from MITA")
        (da "404")))))

  (handle_client (mita (client)
    (in
      (tido ((raw (net_read client)))
        (in
          (print (sada "raw" raw))
          (net_write client
            (http_ok
              (test_handler (sys_http_parse_request raw))))))
      (net_close client)
      da)))

  (serve_one (mita (fd)
    (in
      (print (sada "fd" fd))
      (tido ((client (net_accept fd)))
        (in
          (print (sada "client" client))
          (handle_client client)))
      (net_close fd)
      da))))

(tido ((fd (net_listen 18080)))
  (serve_one fd))
```

> 实现说明：`tido` 只有单个 body 形式，多表达式需要 `in`；`(mita (req) ...)` 才是固定参数，`(mita req ...)` 是变参；`web_server_test2` 拆成函数以避免单条顶层表达式超过 64 个寄存器。

- [ ] **Step 3: 构建 release 二进制并确认端口空闲**

Run:

```bash
cargo build --release
! ss -ltn | grep -q ':18080 '
```

Expected: 构建成功；端口检查退出码 0（没有进程监听 18080）。

- [ ] **Step 4: 启动并验证 web_server_test.mita**

Run:

```bash
timeout 10 ./target/release/mita odomu.mita net.mita http.mita examples/web_server_test.mita >/tmp/opencode/web_test.log 2>&1 &
pid=$!
body=""
for i in $(seq 1 20); do
  body=$(curl -sS http://127.0.0.1:18080/ 2>/dev/null) && break
  sleep 0.1
done
printf '%s\n' "$body"
kill $pid 2>/dev/null || true
wait $pid 2>/dev/null || true
```

Expected: curl 输出 `Hello from MITA`。

- [ ] **Step 5: 启动并验证 web_server_test2.mita**

Run:

```bash
timeout 10 ./target/release/mita odomu.mita net.mita http.mita examples/web_server_test2.mita >/tmp/opencode/web_test2.log 2>&1 &
pid=$!
body=""
for i in $(seq 1 20); do
  body=$(curl -sS http://127.0.0.1:18080/ 2>/dev/null) && break
  sleep 0.1
done
printf '%s\n' "$body"
kill $pid 2>/dev/null || true
wait $pid 2>/dev/null || true
```

Expected: curl 输出 `Hello from MITA`。

- [ ] **Step 6: 提交**

```bash
git add examples/web_server_test.mita examples/web_server_test2.mita
git commit -m "docs: update web server examples for net.mita split"
```

---

### Task 3: 文档、CI 与最终验证

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `.github/workflows/mita-test.yml`
- Modify: `docs/superpowers/specs/2026-11-15-odomu-net-split-design.md`（状态改为“已实施”）

**Interfaces:**
- Consumes: Task 1 的 `net.mita`，Task 2 的示例加载方式。
- Produces: 文档、CI 与代码一致。

- [ ] **Step 1: 更新 README**

在 “You can load library like” 的代码块后补充：

```markdown
Network primitives live in `net.mita` and must be loaded after `odomu.mita`:

```bash
mita odomu.mita net.mita app.mita
```
```

在 “Library functions (odomu.mita)” 之后新增一节：

```markdown
### Network library (net.mita)

* `net_listen` create a listening socket
* `net_accept` accept a connection
* `net_read` read from a socket
* `net_write` write to a socket
* `net_close` close a socket

HTTP helpers live in `http.mita`; load it after `net.mita`:

```bash
mita odomu.mita net.mita http.mita app.mita
```
```

- [ ] **Step 2: 更新 AGENTS.md**

- 在 `STRUCTURE` 中加入：

```text
├── net.mita          # Socket primitives
├── http.mita         # HTTP server library
```

- 更新 `WHERE TO LOOK` 中标准库一行：

```text
| Update stdlib | `odomu.mita` (core), `net.mita` (sockets), `http.mita` (HTTP) | Loaded in that order by CLI |
```

- 将 `NETWORKING & WEB SERVER` 段替换为：

```markdown
## NETWORKING & WEB SERVER
- **Network library** (`net.mita`): `net_listen`, `net_accept`, `net_read`, `net_write`, `net_close`
- **HTTP library** (`http.mita`): `http_ok`, `http_not_found`, `url_decode`, `handle_connection`, `serve_loop`, `run_server`
- **Load order**: `odomu.mita` → `net.mita` → `http.mita`
- **Web server examples**: `examples/web_server.mita`, `web_server_test.mita`, `web_server_test2.mita`
```

- 删除或更新所有关于 `serve_once`、`net_get_buffer`、`http_parse_path` 的旧描述。

- [ ] **Step 3: 更新 CI**

在 `.github/workflows/mita-test.yml` 的 “Run odomu.mita LISP tests” 之前加入：

```yaml
      - name: Load libraries
        run: ./target/release/mita odomu.mita net.mita http.mita
```

- [ ] **Step 4: 更新设计文档状态**

把 `docs/superpowers/specs/2026-11-15-odomu-net-split-design.md` 顶部状态改为：

```markdown
- 状态：已实施
```

- [ ] **Step 5: 最终验证**

Run:

```bash
cargo test
cargo build --release
./target/release/mita odomu.mita examples/odomu_test.mita
./target/release/mita odomu.mita net.mita http.mita
```

Expected: `cargo test` 0 failed；两个 mita 命令退出码均为 0。

- [ ] **Step 6: 跑全部非服务端示例**

Run:

```bash
for f in add car cdr compare cond defn factorial fibonacci hello higher-order if lambda let list progn quicksort setq odomu_test; do
  ./target/release/mita odomu.mita examples/$f.mita >/dev/null || exit 1
done
```

Expected: 退出码 0。

- [ ] **Step 7: 提交**

```bash
git add README.md AGENTS.md .github/workflows/mita-test.yml \
  docs/superpowers/specs/2026-11-15-odomu-net-split-design.md \
  docs/superpowers/plans/2026-11-16-odomu-net-split.md
git commit -m "docs: document net.mita split and update CI"
```
