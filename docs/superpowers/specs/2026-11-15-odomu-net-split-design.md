# odomu 网络部分拆分设计

- 日期：2026-11-15
- 状态：已实施
- 范围：小改动

## 意图

让 `odomu.mita` 回到“通用 Lisp 标准库”的定位，把网络原语移到独立的 `net.mita`。不引入模块系统，保持现有 CLI 多文件加载方式。

## 目标

- `odomu.mita` 删除全部网络函数，只保留通用列表、集合、逻辑和控制流函数。
- 新建 `net.mita`，包含 `net_listen`、`net_accept`、`net_read`、`net_write`、`net_close`。
- 更新受影响的示例、README、AGENTS.md 和 CI 冒烟检查。
- 保持 `mita odomu.mita examples/odomu_test.mita` 现有用法不变。

## 非目标

- 不实现 `(load ...)`、`import` 或任何模块系统。
- 不把通用库进一步拆成 `core.mita` / `list.mita`。
- 不修改 VM 或编译器。
- 不处理闭包上值捕获等语言能力缺口。

## 方案

考虑过三种方案：

- **A. 最小拆分（已选）**：只把网络函数移出 `odomu.mita`，调用方显式加载 `odomu.mita net.mita`。不改语言和运行时。
- B. 按职责拆成 `core` / `list` / `net` 多文件，调用方加载更多文件。文件更细，但当前没有加载器，使用成本高。
- C. 在 B 的基础上增加 `(load "path")`，保留单入口。体验最好，但需要新增文件加载/编译逻辑，超出本次范围。

## 设计细节

### `net.mita`

包含 5 个薄封装：

- `net_listen`
- `net_accept`
- `net_read`
- `net_write`
- `net_close`

实现直接调用现有 `sys_*` 内建函数，不依赖 `libmita_net.so`。`net.mita` 依赖 `odomu.mita` 的 `lawakucha`，因此加载顺序为：

```bash
mita odomu.mita net.mita app.mita
```

### 删除 `serve_once`

`serve_once` 依赖已经不存在于 MITA 层的 `net_get_buffer` 和 `http_parse_path`，而 `http.mita` 的 `handle_connection` 已经覆盖“接受连接、解析请求、返回响应”的用途。因此直接删除，不迁移到 `net.mita`，避免把 HTTP 逻辑重新混进网络原语。

### 示例更新

- `examples/web_server_test.mita`：改用 `net_listen` + `http.mita` 的 `handle_connection`，响应通过 `http_ok` / `http_not_found` 生成。
- `examples/web_server_test2.mita`：移除已废弃的 `net_get_buffer`、`http_parse_path`，改为现代 API 的底层 socket 示例。
- `examples/web_server.mita`：无需改动，继续使用 `http.mita`。

### 文档与 CI

- `README.md`：补充 `net.mita` 的用途和加载方式。
- `AGENTS.md`：更新目录结构、网络库归属和示例命令。
- CI 增加冒烟步骤：`./target/release/mita odomu.mita net.mita http.mita`，确保三个库可以一起加载。

## 验证

- `cargo test` 全部通过。
- `./target/release/mita odomu.mita examples/odomu_test.mita` 通过。
- `./target/release/mita odomu.mita net.mita http.mita` 退出码为 0。
- 所有不监听端口的 `examples/*.mita` 正常运行。
