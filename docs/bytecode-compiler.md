# 方案C：MITA 字节码 VM + JIT 编译器

## 概述

将 MITA 从纯树遍历解释器演进为**字节码 VM + 可选 JIT** 架构。

```
阶段1: MITA AST ──→ 字节码 (ByteCode)
阶段2: 字节码 ──→ VM 解释执行
阶段3: 字节码 ──→ MLIR/LLVM ──→ 机器码 (JIT/AOT)
```

## 为什么先走字节码

| 直接 S-expr → MLIR | 字节码 → VM → MLIR |
|---|---|
| AST 是树， lowering 复杂 | 字节码是线性 IR， lowering 简单 |
| 每次编译都重新分析控制流 | 字节码已显式化控制流 (jmp/call/ret) |
| 调试困难 | VM 可单步、可 dump、可验证 |
| 没有 fallback | VM 是 fallback，JIT 是可选优化 |

## 字节码设计

### 架构：栈式虚拟机

- **操作数栈**：所有运算在栈上进行
- **局部变量表**：函数参数 + let 绑定 + 临时变量
- **常量池**：数字、字符串、符号名
- **调用栈**：函数调用链

### 指令集 (v1)

#### 栈操作
| 指令 | 操作 | 说明 |
|------|------|------|
| `NOP` | - | 空操作 |
| `POP` | 弹出栈顶 | 丢弃值 |
| `DUP` | 复制栈顶 | a → a a |
| `SWAP` | 交换栈顶两个 | a b → b a |

#### 常量加载
| 指令 | 操作数 | 说明 |
|------|--------|------|
| `LOAD_CONST u16` | idx | 从常量池加载常量到栈 |
| `LOAD_NIL` | - | 加载 nil |
| `LOAD_TRUE` | - | 加载 da |
| `LOAD_FALSE` | - | 加载 nye |
| `LOAD_NUM i64` | num | 加载立即数整数 |

#### 变量访问
| 指令 | 操作数 | 说明 |
|------|--------|------|
| `LOAD_LOCAL u16` | idx | 加载局部变量 |
| `STORE_LOCAL u16` | idx | 存储到局部变量 |
| `LOAD_GLOBAL u16` | idx | 加载全局变量 (通过名称索引) |
| `STORE_GLOBAL u16` | idx | 存储全局变量 |
| `LOAD_UPVAL u16` | idx | 加载闭包上值 |
| `STORE_UPVAL u16` | idx | 存储闭包上值 |

#### 算术 (弹出2个，压入1个)
| 指令 | 说明 |
|------|------|
| `ADD` | 加法 |
| `SUB` | 减法 |
| `MUL` | 乘法 |
| `DIV` | 除法 |
| `MOD` | 取模 |

#### 比较 (弹出2个，压入 bool)
| 指令 | 说明 |
|------|------|
| `EQ` | 等于 (shato) |
| `NE` | 不等于 (nyeshato) |
| `LT` | 小于 (aba) |
| `GT` | 大于 (unta) |
| `LE` | 小于等于 (abashato) |
| `GE` | 大于等于 (untashato) |

#### 列表操作
| 指令 | 操作数 | 说明 |
|------|--------|------|
| `CONS` | - | 弹出 car cdr，压入 cons cell |
| `CAR` | - | 弹出 list，压入 car |
| `CDR` | - | 弹出 list，压入 cdr |
| `LIST u16` | n | 弹出 n 个值，压入列表 |

#### 控制流
| 指令 | 操作数 | 说明 |
|------|--------|------|
| `JMP i32` | offset | 无条件跳转 (相对当前 ip) |
| `JMP_IF i32` | offset | 栈顶为 true 则跳转 |
| `JMP_IF_NOT i32` | offset | 栈顶为 false 则跳转 |

#### 函数调用
| 指令 | 操作数 | 说明 |
|------|--------|------|
| `CALL u16` | argc | 调用栈顶函数，参数在下方 |
| `TAIL_CALL u16` | argc | 尾调用 (复用当前栈帧) |
| `RET` | - | 返回栈顶值给调用者 |
| `MAKE_CLOSURE` | - | 弹出函数对象，创建闭包 (捕获上值) |

#### 内建函数
| 指令 | 说明 |
|------|------|
| `BUILTIN_PRINT` | 打印栈顶 |
| `BUILTIN_TYPE` | 返回类型标签 |

### 常量池条目

```rust
enum Const {
    Number(i64),
    String(String),   // 用于 quote 字符串
    Symbol(String),   // 符号名（用于全局变量查找）
    Nil,
}
```

### 函数对象 (FuncObject)

```rust
struct FuncObject {
    name: String,
    bytecode: Vec<u8>,       // 指令序列
    const_pool: Vec<Const>,  // 常量池
    num_params: u16,         // 参数数量
    num_locals: u16,         // 局部变量数量 (含参数)
    has_varargs: bool,       // 是否变参 (formals 为 atom)
    upvalues: Vec<UpvalueDesc>, // 需要捕获的上值描述
}
```

### 闭包 (Closure)

```rust
struct Closure {
    func: Rc<FuncObject>,
    upvalues: Vec<Rc<RefCell<Value>>>, // 捕获的变量
}
```

### 值表示 (Value)

运行时值需要 tag + payload，因为 MITA 是动态类型：

```rust
enum Value {
    Nil,
    Number(i64),
    Bool(bool),
    String(String),
    Symbol(String),
    Cons(Rc<(Value, Value)>),    // (car . cdr)
    Closure(Rc<Closure>),
    Builtin(fn(&mut VM, &[Value]) -> Value),
}
```

## VM 设计

### 执行状态

```rust
struct VM {
    // 当前执行帧
    frames: Vec<CallFrame>,
    // 操作数栈
    stack: Vec<Value>,
    // 全局变量
    globals: HashMap<String, Value>,
}

struct CallFrame {
    closure: Rc<Closure>,
    ip: usize,           // 指令指针
    stack_base: usize,   // 当前帧在操作数栈中的基址
    locals: Vec<Value>,  // 局部变量 (也可用 stack[base..] 的一部分)
}
```

### 执行循环

```rust
fn run(&mut self) {
    loop {
        let opcode = self.read_byte();
        match opcode {
            OP_LOAD_CONST => {
                let idx = self.read_u16();
                let val = self.frame().closure.func.const_pool[idx].clone();
                self.push(val);
            }
            OP_ADD => {
                let b = self.pop_number();
                let a = self.pop_number();
                self.push(Value::Number(a + b));
            }
            OP_CALL => {
                let argc = self.read_u16();
                self.call_function(argc);
            }
            OP_TAIL_CALL => {
                let argc = self.read_u16();
                self.tail_call(argc);
            }
            OP_RET => {
                let val = self.pop();
                if self.frames.len() == 1 {
                    return val; // 顶层返回
                }
                self.frames.pop();
                self.push(val);
            }
            OP_JMP_IF_NOT => {
                let offset = self.read_i32();
                if !self.pop().is_truthy() {
                    self.frame().ip += offset as usize;
                }
            }
            // ...
        }
    }
}
```

## 编译器 (AST → 字节码)

### 编译流程

```
MITA AST
  ├── 常量收集 (收集所有数字、字符串、符号到常量池)
  ├── 作用域分析 (确定局部变量索引、上值捕获)
  ├── 字节码生成 (遍历 AST，生成指令)
  └── 后处理 (回填跳转偏移量)
```

### AST 到字节码的映射

| AST 节点 | 字节码 |
|----------|--------|
| `Atom(Number(42))` | `LOAD_CONST <42>` |
| `Atom(Symbol("x"))` | `LOAD_LOCAL <x_idx>` 或 `LOAD_GLOBAL <"x">` |
| `(celi a b)` | `<compile a>` `<compile b>` `ADD` |
| `(ka test then else)` | `<test>` `JMP_IF_NOT <else_label>` `<then>` `JMP <end>` `else_label:` `<else>` `end:` |
| `(tido ((x 1)) body)` | `LOAD_CONST <1>` `STORE_LOCAL <x_idx>` `<body>` |
| `(mita (x y) body)` | 生成 FuncObject，body 编译为字节码 |
| `(f a b)` | `LOAD_GLOBAL <"f">` `<a>` `<b>` `CALL 2` |
| `(f a b)` 尾调用 | `LOAD_GLOBAL <"f">` `<a>` `<b>` `TAIL_CALL 2` |

### 尾调用检测

在编译函数体时，跟踪当前是否在**尾位置**：
- 函数体的最后一个表达式 → 尾位置
- `ka` 的 then/else 分支 → 尾位置
- `in` 的最后一个表达式 → 尾位置
- `dala` 的匹配分支 → 尾位置

在尾位置遇到函数调用 → 生成 `TAIL_CALL` 而非 `CALL`。

### 上值捕获 (闭包)

当函数引用外部局部变量时：
1. 作用域分析标记该变量为"上值"
2. 外层函数在局部变量创建时分配 `Upvalue` 对象
3. 内层函数通过 `LOAD_UPVAL` 访问

## 分阶段实现计划

### 阶段 1：基础设施 (1-2 周)
- [ ] 定义 `Value` enum 和运行时类型系统
- [ ] 定义字节码格式 (`u8` opcode + 变长操作数)
- [ ] 实现常量池和 `FuncObject`
- [ ] 实现 VM 骨架（指令 fetch/decode，空执行循环）
- [ ] 实现栈操作指令 (PUSH/POP/DUP/SWAP)

### 阶段 2：基础 VM (1-2 周)
- [ ] 实现算术指令 (ADD/SUB/MUL/DIV)
- [ ] 实现比较指令 (EQ/LT/GT/...)
- [ ] 实现变量指令 (LOAD/STORE LOCAL/GLOBAL)
- [ ] 实现控制流 (JMP/JMP_IF/JMP_IF_NOT)
- [ ] 实现常量加载 (LOAD_CONST/LOAD_NIL/...)
- [ ] 实现简单的 REPL：`输入 MITA 表达式 → 编译 → VM 执行 → 输出`

### 阶段 3：AST 编译器 (2-3 周)
- [ ] 实现 AST 到字节码的编译器 (Compiler 结构体)
- [ ] 实现常量收集和作用域分析
- [ ] 编译基础表达式 (atom, arithmetic, comparison)
- [ ] 编译控制流 (if, cond)
- [ ] 编译局部绑定 (let)
- [ ] 编译函数定义 (lambda / mita)
- [ ] 编译函数调用 (call + tail call)
- [ ] 编译上值捕获 (closure)
- [ ] 编译特殊形式 (quote, progn, setq)

### 阶段 4：功能对齐 (1-2 周)
- [ ] 实现所有内建函数的内联/调用 (lawa, kucha, upa, etc.)
- [ ] 实现列表操作指令 (CONS, CAR, CDR, LIST)
- [ ] 支持变参函数
- [ ] 实现标准库加载 (odomu.mita 编译为字节码)
- [ ] 运行全部 examples/*.mita 验证功能等价

### 阶段 5：优化 (1 周)
- [ ] 常量折叠
- [ ] 死代码消除
- [ ] 内联小函数
- [ ] 栈操作优化 (peephole)

### 阶段 6：JIT → 机器码 (2-4 周，后续)
- [ ] 字节码 → MLIR 方言 (`mita` dialect)
- [ ] MLIR lowering → LLVM IR
- [ ] 热函数识别（执行计数）
- [ ] JIT 编译热函数
- [ ] fallback 到 VM 解释器

## 文件结构

```
src/
  lib.rs           # 导出公共 API
  main.rs          # CLI (REPL + 文件执行)
  token.rs         # 词法分析 (已有)
  parser.rs        # 语法分析 (已有)
  expr.rs          # AST (已有)
  eval.rs          # 树遍历解释器 (保留作为参考/对比)
  elementary.rs    # 内建函数 (保留)
  
  # 新增:
  bytecode/
    mod.rs         # 字节码定义
    instr.rs       # 指令集枚举
    func.rs        # FuncObject / Closure
    value.rs       # VM Value 类型
  compiler/
    mod.rs         # 编译器入口
    scope.rs       # 作用域分析
    codegen.rs     # 字节码生成
  vm/
    mod.rs         # VM 入口
    frame.rs       # 调用帧
    exec.rs        # 执行循环
    builtins.rs    # 内建函数 VM 实现
```

## 关键决策

1. **栈式 vs 寄存器式**：选栈式，简单、与 JVM/Lua 一致、lowering 到 MLIR 也容易
2. **Value  boxing**：所有值都 enum box，牺牲内存换简单。后续 JIT 可解 box
3. **尾调用**：VM 层面通过 `TAIL_CALL` 指令复用当前栈帧，与 TCO 语义一致
4. **GC**：先用 `Rc` + 循环引用不管（与当前解释器一致）。后续 JIT 阶段替换为 tracing GC
5. **保留解释器**：`eval.rs` 不删除，作为 golden reference 和 fallback

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| 编译器 bug 导致与解释器行为不一致 | 用同一套测试集，对比解释器 vs VM 输出 |
| 闭包/上值实现复杂 | 先实现无闭包的子集，再逐步添加 |
| 性能不如预期 | VM 只是中间层，最终靠 JIT 提性能 |
| 代码膨胀 | 用 Rust module 隔离，保持原解释器不受影响 |

## 下一步

请审阅此方案。确认后，我将按**阶段 1**开始实现。
