# PROJECT KNOWLEDGE BASE

**Generated:** 2026-05-16
**Commit:** b551fcc
**Branch:** main

## OVERVIEW
MITA is a Rust LISP interpreter implementing a custom dialect with hilichurl-themed keywords (`lawa` = car, `kucha` = cdr, `upa` = cons). Derived from Rob Pike's pedagogical LISP. Rewritten from Go to Rust.

Now includes a **register-based bytecode VM** with an AST-to-bytecode compiler and **C FFI** support.

## STRUCTURE
```
.
├── Cargo.toml       # Rust project manifest (libloading dependency)
├── Cargo.lock       # Dependency lock file
├── src/             # Library source code
│   ├── lib.rs       # Library entry point (pub modules)
│   ├── main.rs      # CLI binary entry point
│   ├── token.rs     # Lexer and Token types
│   ├── parser.rs    # Parser (S-expression parser)
│   ├── expr.rs      # Expr enum (Nil, Atom, Cons)
│   ├── eval.rs      # Tree interpreter (Context, Scope, apply)
│   ├── elementary.rs # Built-in functions (lawa, kucha, celi, etc.)
│   ├── vm/          # Register-based VM
│   │   ├── mod.rs   # VM module exports
│   │   ├── value.rs # Value enum (Nil, Number, Bool, String, Symbol, Cons, Closure, Builtin)
│   │   ├── exec.rs  # VM execution loop, CallFrame, register file
│   │   ├── isa.rs   # Instruction encoding/decoding (RISC-V style)
│   │   └── builtins.rs # VM builtin functions (arithmetic, cons, FFI)
│   └── compiler/    # AST-to-bytecode compiler
│       └── mod.rs   # Compiler with register allocation
├── tests/           # Integration tests
│   ├── parse_tests.rs  # Parser unit tests
│   ├── eval_tests.rs   # Tree interpreter unit tests
│   ├── vm_tests.rs     # VM unit tests
│   ├── vm_debug.rs     # VM debug test
│   └── ffi_test_lib.c  # C test library for FFI
├── docs/            # Documentation
│   └── bytecode-compiler.md # Compiler design doc
├── examples/        # .mita sample programs
├── .github/         # CI workflows (Rust, not Go)
│   └── workflows/
│       ├── mita-test.yml # Main test workflow
│       └── rust.yml      # Rust build/test workflow
├── odomu.mita       # Standard library
└── AGENTS.md        # This file
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Add built-in function (tree interpreter) | `src/elementary.rs` + `src/eval.rs` | Register in `ELEMENTARY` HashMap, implement as `Context::xxx_func` |
| Add built-in function (VM) | `src/vm/builtins.rs` | Add function + register in `register_all()` |
| Add VM opcode | `src/vm/isa.rs` + `src/vm/exec.rs` | Add encoding, decoding, and execution |
| Change lexer/tokenizer | `src/token.rs` | `TokenType` enum and `Lexer` struct |
| Modify parser/AST | `src/parser.rs` + `src/expr.rs` | `Expr` enum and `Parser` methods |
| Change evaluation logic (tree) | `src/eval.rs` | `Context::eval()`, `Context::apply()`, stack management |
| Change evaluation logic (VM) | `src/vm/exec.rs` | `VM::run()`, `enter_closure()`, `tail_call()` |
| Modify compiler | `src/compiler/mod.rs` | `compile_expr()`, `compile_call()`, register allocation |
| Modify CLI behavior | `src/main.rs` | REPL, flags, file loading, panic recovery |
| Add language examples | `examples/*.mita` | MITA source files |
| Update stdlib | `odomu.mita` | Loaded at runtime by CLI |
| Add C FFI function | `src/vm/builtins.rs` | `builtin_ffi_exec()`, uses `libloading` |
| Fix CI | `.github/workflows/*.yml` | Rust toolchain, compile C test lib |

## CONVENTIONS
- **Library + binary**: `src/lib.rs` exports public API; `src/main.rs` is the CLI binary
- **Panic-driven errors**: `errorf()` panics with `panic_any(Error(...))`; CLI recovers via `catch_unwind`
- **MITA naming**: Built-ins use fictional language names (`celi` = +, `movo` = -, `shato` = ==)
- **External dependencies**: `libloading` for C FFI (only dependency)
- **Expr enum**: Uses `Rc<Expr>` for shared ownership (no GC, reference counted)
- **VM Value enum**: Uses `Rc<str>`, `Rc<(Value, Value)>`, `Rc<Closure>` for shared ownership
- **Register allocation**: Lambda params mapped to X18+ to avoid arg register clobbering
- **Special forms**: `mita` (lambda), `dala` (cond), `plata` (quote), `muhe` (defun), `tido` (let), `ka` (if), `in` (progn), `plama` (setq)

## ANTI-PATTERNS (THIS PROJECT)
- **Panics for normal control flow**: Parse errors, undefined symbols, stack overflow all panic rather than return errors
- **No automated .mita tests**: Integration test runs interpreter against `.mita` files with manual comment-based assertions (`; => expected`)

## COMMANDS
```bash
# Build CLI
cargo build --release

# Run tests
cargo test

# Run integration tests
./target/release/mita odomu.mita examples/odomu_test.mita

# Install from source
cargo install --path .

# Compile C FFI test library (for VM tests)
gcc -shared -fPIC -o tests/libmita_test.so tests/ffi_test_lib.c
```

## BUILT-IN FUNCTIONS
### Tree Interpreter (src/elementary.rs)
Arithmetic: `celi` (+), `movo` (-), `celida` (*), `movoda` (/)
Comparison: `shato` (==), `nyeshato` (!=), `aba` (<), `unta` (>), `abashato` (<=), `untashato` (>=)
List: `lawa` (car), `kucha` (cdr), `upa` (cons)

### VM Builtins (src/vm/builtins.rs)
Same arithmetic and list functions as tree interpreter.
Plus: `mite` (C FFI) - call C functions from shared libraries

## C FFI (mite)
Syntax: `(mite "libpath.so" "func_name" arg1 arg2)`
- Loads shared library dynamically (cached per-VM)
- Calls C function with 0-2 `int64_t` arguments
- Returns `int64_t` as MITA number
- Example: `(mite "libm.so" "pow" 2 3)` → `8`

## VM ARCHITECTURE
- **Register file**: 32 registers (X0=zero, X3=temp, X10-X17=arg regs, X18+=saved/local regs)
- **Instruction set**: RISC-V style encoding (op[31:24], rd[23:18], rs1[17:12], rs2[11:6], funct3[5:0])
- **Opcodes**: MV, LI, ADD, SUB, MUL, DIV, REM, ADDI, SEQ, SNE, SLT, SGT, SLE, SGE, CONS, CAR, CDR, LW, SW, BEQ, BNE, BLT, BGE, JAL, J, CALL, TAIL, RET, BUILTIN, SETGLOBAL
- **Calling convention**: Args in X10+, result in caller-specified rd, saved regs X18+ restored on return
- **Tail call optimization**: Reuses current frame for tail calls

## COMPILER
- **Register allocation**: Linear scan, `next_reg` increments per allocation
- **Param mapping**: Lambda params → X18+ (saved registers)
- **Arg evaluation**: Args evaluated before function loaded (prevents clobbering)
- **Cond compilation**: Unified result register, branch to end after each clause
- **Tail detection**: `is_tail && !is_toplevel` determines CALL vs TAIL vs RET

## LIBRARY FUNCTIONS (odomu.mita)
All library functions now use Hilichurlian (丘丘语) names:

| Function | Hilichurlian | Lisp Equivalent |
|----------|-------------|-----------------|
| cadr | `lawakucha` | (car (cdr x)) |
| caddr | `lawakuchakucha` | (car (cdr (cdr x))) |
| cddr | `kuchakucha` | (cdr (cdr x)) |
| list | `sada` | list |
| length | `tiga` | length |
| map | `si` | map |
| filter | `valo` | filter |
| reduce | `mosi` | reduce |
| append | `tomo` | append |
| reverse | `domu` | reverse |
| assoc | `mito` | assoc |
| member | `odomu` | member |
| last | `zido` | last |
| nth | `eleka` | nth |
| and | `kuzi` | and |
| or | `todo` | or |
| not | `biat` | not |
| remove | `kundala` | remove |
| flatten | `pupu` | flatten |

## NOTES
- ~~Self-hosted `riscv-builders` runner used in CI (non-standard)~~ → Now uses `ubuntu-latest`
- `odomu.mita` uses Hilichurlian names for all library functions (not English)
- `eval_condition` treats the last clause as an implicit else (returns unevaluated if no remaining clauses)
- `shato` uses structural equality via `equal_expr` (not numeric equality)
- `nil` and `nya` are treated as equal in `equal_expr` (matching Go's `isNya()` behavior)
- `kuzi`/`todo` are variadic via `mita args` pattern (single atom formal captures entire arg list)
- `ELEMENTARY` uses `std::sync::OnceLock` (no unsafe code)
- VM uses `libloading` (unsafe required for `Library::new` and `Symbol::get`)
- String lexer fixed: quotes no longer included in `Token.text` for `TokenType::String`
