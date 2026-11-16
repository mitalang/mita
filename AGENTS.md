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
│   ├── compiler/    # AST-to-bytecode compiler
│   │   └── mod.rs   # Compiler with register allocation
│   └── backend/     # Native (MLIR/LLVM) codegen + build pipeline
│       ├── mod.rs   # Backend module exports
│       ├── mlir.rs  # MITA -> LLVM-dialect MLIR codegen
│       └── build.rs # `mita build` pipeline (mlir-translate/llc/cc/link)
├── tests/           # Integration tests
│   ├── parse_tests.rs  # Parser unit tests
│   ├── eval_tests.rs   # Tree interpreter unit tests
│   ├── vm_tests.rs     # VM unit tests
│   ├── library_test.rs # Standard-library split tests
│   ├── vm_debug.rs     # VM debug test
│   └── ffi_test_lib.c  # C test library for FFI
├── docs/            # Documentation
│   └── bytecode-compiler.md # Compiler design doc
├── examples/        # .mita sample programs
├── .github/         # CI workflows (Rust, not Go)
│   └── workflows/
│       ├── mita-test.yml # Main test workflow
│       └── rust.yml      # Rust build/test workflow
├── runtime/         # Embedded C runtime (mita_runtime.c/.h)
├── odomu.mita       # Generic standard library
├── net.mita         # Socket primitives
├── http.mita        # HTTP server library
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
| Update stdlib | `odomu.mita` (core), `net.mita` (sockets), `http.mita` (HTTP) | Loaded in that order by CLI |
| Add C FFI function | `src/vm/builtins.rs` | `builtin_ffi_exec()`, uses `libloading` |
| Add native codegen | `src/backend/mlir.rs` | `compile()` emits LLVM-dialect MLIR |
| Change native build pipeline | `src/backend/build.rs` | `build_files()`, `cc_for_target()`, `llc_target_args()` |
| Change C runtime | `runtime/mita_runtime.c` + `runtime/mita_runtime.h` | Embedded via `include_str!` in `build.rs` |
| Fix CI | `.github/workflows/*.yml` | Rust toolchain, compile C test lib, cross toolchain + qemu |

## CONVENTIONS
- **Library + binary**: `src/lib.rs` exports public API; `src/main.rs` is the CLI binary
- **Panic-driven errors**: `errorf()` panics with `panic_any(Error(...))`; CLI recovers via `catch_unwind`
- **MITA naming**: Built-ins use fictional language names (`celi` = +, `movo` = -, `shato` = ==)
- **External dependencies**: `libloading` for C FFI (only dependency)
- **Expr enum**: Uses `Rc<Expr>` for shared ownership (no GC, reference counted)
- **VM Value enum**: Uses `Rc<str>`, `Rc<(Value, Value)>`, `Rc<Closure>` for shared ownership
- **Register allocation**: Lambda params mapped to X18+ to avoid arg register clobbering
- **Special forms**: `muhe` (lambda), `dala` (cond), `plata` (quote), `sada` (defun), `zido` (let), `ka` (if), `in` (progn), `movo` (setq)

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

# Native build (MITA -> MLIR -> LLVM -> ELF), per target
./target/release/mita build --target x86_64-unknown-linux-gnu app.mita -o app
./target/release/mita build --target aarch64-unknown-linux-gnu app.mita -o app-arm64
./target/release/mita build --target riscv64-unknown-linux-gnu app.mita -o app-riscv64

# Cross-build tests (requires qemu-user + cross gcc)
MITA_TEST_CROSS=1 cargo test --test native_build -- --nocapture
```

## BUILT-IN FUNCTIONS
### Tree Interpreter (src/elementary.rs)
Arithmetic: `celi` (+), `lata` (-), `kuzi` (*), `gusha` (/)
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

## NATIVE BUILD (mita build)
`mita build --target <triple> file.mita -o out` compiles MITA to a native Linux ELF:

MITA source → MLIR (LLVM dialect, `src/backend/mlir.rs`) → `mlir-translate --mlir-to-llvmir` → `llc -O2 -filetype=obj -mtriple=<triple> -relocation-model=pic` → object. The embedded C runtime (`runtime/mita_runtime.c/.h`, `include_str!`-ed into the binary) is compiled by the target `cc` and linked into the ELF.

| triple | `llc -mtriple` | CC | run on host |
|---|---|---|---|
| `x86_64-unknown-linux-gnu` | same | `cc` | native |
| `aarch64-unknown-linux-gnu` | same | `aarch64-linux-gnu-gcc` | `qemu-aarch64 -L /usr/aarch64-linux-gnu` |
| `riscv64-unknown-linux-gnu` | same | `riscv64-linux-gnu-gcc` | `qemu-riscv64 -L /usr/riscv64-linux-gnu` |

- Tool discovery: `MLIR_TRANSLATE` / `LLC` / `CC` env vars → `PATH` → `/usr/lib/llvm-*/bin/`. Unknown triples return a clear error.
- riscv64: `llc` defaults to soft-float `rv64i`, but Debian's `riscv64-linux-gnu-gcc` is hard double-float (`lp64d`/`rv64imafdc`); `build.rs` adds `-mattr=+m,+a,+f,+d,+c` to `llc` to match.
- PIE: generated object uses `-relocation-model=pic`; runtime compiled with target cc defaults (PIE on Debian/Ubuntu); final link is PIE. All consistent.
- Last top-level expression result is auto-printed (matches CLI `load()` behavior).
- `--emit-mlir <path>` writes the MLIR; `--keep-temps` keeps intermediates.

## VM ARCHITECTURE
- **Register file**: 64 registers (X0=zero, X3=temp, X10-X17=arg regs, X18+=saved/local regs)
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
All library functions use Hilichurlian (丘丘语) names aligned with their meanings:

| Function | Hilichurlian | Lisp Equivalent |
|----------|-------------|-----------------|
| cadr | `lawakucha` | (car (cdr x)) |
| caddr | `lawakuchakucha` | (car (cdr (cdr x))) |
| cddr | `kuchakucha` | (cdr (cdr x)) |
| list | `tiga` | list |
| length | `tigamani` | length |
| map | `si` | map |
| filter | `valo` | filter |
| reduce | `mosi` | reduce |
| append | `tomo` | append |
| reverse | `domu` | reverse |
| assoc | `mito` | assoc |
| member | `odomu` | member |
| last | `untasi` | last |
| nth | `eleka` | nth |
| and | `kuzitomo` | and |
| or | `todo` | or |
| not | `biat` | not |
| remove | `ika` | remove |
| flatten | `tigakundala` | flatten |
| null | `nyasi` | null |
| consp | `upasi` | consp |
| atom | `unusi` | atom |
| listp | `tigasi` | listp |
| find | `mitosi` | find |
| position | `elekasi` | position |
| count | `manisi` | count |
| copy list | `sadasi` | copy-list |
| subseq | `kuchasi` | subseq |
| adjoin | `tomosi` | adjoin |
| union | `tomotomo` | union |
| intersection | `lawatomo` | intersection |
| set difference | `kundalasi` | set-difference |
| subst | `movosi` | subst |
| every | `tigatomo` | every |
| some | `tigakucha` | some |
| when | `kasi` | when |
| unless | `nyekasi` | unless |
| prog1 | `inusi` | prog1 |
| prog2 | `indu` | prog2 |

## NETWORKING & WEB SERVER
- **Network library** (`net.mita`): `net_listen`, `net_accept`, `net_read`, `net_write`, `net_close`
- **HTTP library** (`http.mita`): `http_ok`, `http_not_found`, `url_decode`, `handle_connection`, `serve_loop`, `run_server`
- **Load order**: `odomu.mita` → `net.mita` → `http.mita`
- **Web server examples**: `examples/web_server.mita`, `web_server_test.mita`, `web_server_test2.mita`
- **VM `print` builtin**: index 15, prints values to stdout

## RECENT FIXES
- **OP_TAIL inline rewrite**: Builtin branch pops only current frame, restores saved_regs, continues at return_pc (fixes toplevel sequencing)
- **Variadic lambdas**: VM `enter_closure` builds cons list from X10..X10+argc-1 when `FuncObject.variadic=true`
- **Register file expanded**: 32 → 64 registers to support complex odomu.mita functions
- **`enter_closure` register clearing removed**: No longer clobbers caller registers (was causing nil args)
- **String lexer fixed**: quotes no longer included in `Token.text` for `TokenType::String`

## NOTES
- ~~Self-hosted `riscv-builders` runner used in CI (non-standard)~~ → Now uses `ubuntu-latest`
- `odomu.mita` uses Hilichurlian names for all library functions (not English)
- `sada` is now defun (solidify); the atom/symbol concept is `unusi` (one-thing)
- `eval_condition` treats the last clause as an implicit else (returns unevaluated if no remaining clauses)
- `shato` uses structural equality via `equal_expr` (not numeric equality)
- `nil` and `nya` are treated as equal in `equal_expr` (matching Go's `isNya()` behavior)
- `kuzitomo`/`todo` are variadic via `muhe args` pattern (single atom formal captures entire arg list)
- `zido` takes a single body form; use `in` to sequence multiple body expressions
- Parenthesize lambda formals: `(muhe (x) ...)` is fixed-arity, `(muhe x ...)` is variadic
- `ELEMENTARY` uses `std::sync::OnceLock` (no unsafe code)
- VM uses `libloading` (unsafe required for `Library::new` and `Symbol::get`)
