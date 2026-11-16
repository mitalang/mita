# MITA 丘丘语命名说明

MITA 的语言层名字按丘丘语本义、隐喻和复合词构造。丘丘语没有计算机/数学词汇，因此：

- **本义优先**：`da/nye`、`unu/du`、`lawa`（头/王）、`kucha`（小/剩余）、`shato`（相似）、`aba/unta`（前/后）、`beru`（说）、`mosi`（吃）、`valo`（告别）、`mito`（知道）。
- **隐喻其次**：`plata`（盾）→ quote，`mosi`（吃）→ reduce，`valo`（告别）→ filter。
- **复合词最后**：`untasi`（after-thing）→ last，`elekasi`（at-thing）→ position，`tigakundala`（拆开一捆）→ flatten。
- **系统层保留英文**：`sys_*`、`net_*`、`http_*`、数学/字符串底层内建、`mite` FFI。丘丘语没有对应概念，硬造词会降低可读性。

## 核心

| 概念 | 名字 | 说明 |
|---|---|---|
| lambda | `muhe` | want/desire，函数作为“愿望” |
| defun | `sada` | solidify，把名字做实 |
| let | `zido` | here，绑定在此 |
| if | `ka` | exist? |
| cond | `dala` | which? |
| progn | `in` | in sequence |
| setq | `movo` | move，把值移进变量 |
| quote | `plata` | shield，保护表达式不被求值 |
| cons | `upa` | combine/gather |
| car | `lawa` | king/head |
| cdr | `kucha` | small/rest |
| + | `celi` | heat/fire，升温 |
| - | `lata` | ice/cold，降温 |
| * | `kuzi` | strengthen/grow |
| / | `gusha` | share/divide vegetables |
| == / != | `shato` / `nyeshato` | similar / not similar |
| < / > | `aba` / `unta` | before / after |
| <= / >= | `abashato` / `untashato` | |
| not | `biat` | hate/negate |
| print | `beru` | speak |

## `odomu.mita`

| 概念 | 名字 | 说明 |
|---|---|---|
| list | `tiga` | 一捆木棍/石头 |
| length | `tigamani` | 用手/手指量一捆 |
| map | `si` | 对每个 thing 操作 |
| filter | `valo` | 对不符合的 say bye |
| reduce | `mosi` | 吃掉整个列表 |
| append | `tomo` | ally/join |
| reverse | `domu` | dance/turn |
| assoc | `mito` | know/查找 |
| member | `odomu` | friend/在不在 |
| last | `untasi` | after-thing |
| nth | `eleka` | now/at index |
| and | `kuzitomo` | strong-allies（`kuzi` 留给 `*`） |
| or | `todo` | yield |
| remove | `ika` | enemy |
| flatten | `tigakundala` | 拆开一捆 |
| null | `nyasi` | nothing-thing |
| consp | `upasi` | combined-thing |
| atom | `unusi` | one-thing |
| listp | `tigasi` | bundle-thing |
| find | `mitosi` | know-thing |
| position | `elekasi` | at-thing |
| count | `manisi` | hand-thing |
| copy list | `sadasi` | solid-thing |
| subseq | `kuchasi` | rest-thing |
| adjoin | `tomosi` | join-thing |
| union | `tomotomo` | join-join |
| intersection | `lawatomo` | head-join / common |
| set difference | `kundalasi` | take-thing |
| subst | `movosi` | move-thing |
| every | `tigatomo` | whole bundle |
| some | `tigakucha` | small bundle / at least one |
| when | `kasi` | if-thing |
| unless | `nyekasi` | not-if-thing |
| prog1 | `inusi` | sequence-one |
| prog2 | `indu` | sequence-two |

## 链式改名

以下名字在旧代码里同时存在，必须**同时替换**，不能顺序 `sed`：

```text
mita  -> muhe
muhe  -> sada
sada  -> tiga
tiga  -> tigamani
tido  -> zido
zido  -> untasi
plama -> movo
movo  -> lata
celida -> kuzi
kuzi  -> kuzitomo
```

例如旧 `(muhe ((f (mita ...))))` 变成 `(sada ((f (muhe ...))))`，不能先改 `mita` 再改 `muhe`。

## 复合词

复合词按“名词 + 动作/属性”拼接：

- `tiga`（一捆）+ `kundala`（拆/拿）= `tigakundala`（flatten）
- `tiga`（一捆）+ `mani`（手/量）= `tigamani`（length）
- `mito`（知道）+ `si`（thing）= `mitosi`（find）
- `eleka`（at/now）+ `si`（thing）= `elekasi`（position）
- `tomo`（join）+ `tomo`（join）= `tomotomo`（union）
- `in`（sequence）+ `usi`（one）= `inusi`（prog1）

## 已删除

- `tido`（旧 let）→ `zido`
- `plama`（旧 setq）→ `movo`
- `pupu`（旧 flatten）→ `tigakundala`
- `celida`（旧 *）→ `kuzi`
- `movoda`（旧 /）→ `gusha`
- `null` / `consp` / `atom` → `nyasi` / `upasi` / `unusi`
- `eq` / `eql` → 直接使用 `shato`
- tree-only `list` / `apply` → 删除；`list` 用 `tiga`，`apply` 不再提供

## 保留英文

`sys_*`、`net_listen`、`net_accept`、`net_read`、`net_write`、`net_close`、`http_ok`、`http_not_found`、`url_decode`、`handle_connection`、`serve_loop`、`run_server`、`mite`，以及 `mod/abs/max/min/floor/ceiling/truncate/round/expt/random/sqrt/zerop/plusp/minusp/evenp/oddp/numberp`。

## 注意

- `sada` 现在是 defun，不再表示“符号/原子”；符号/原子在文档中叫 `unusi`。
- `nil` 与 `nya` 在相等判断中仍等价。
- `net.mita` / `http.mita` 的函数体也使用新语言 token，但公开函数名不变。
