# PROJECT KNOWLEDGE BASE

**Generated:** 2026-05-12
**Commit:** f030045
**Branch:** main

## OVERVIEW
MITA is a Rust LISP interpreter implementing a custom dialect with hilichurl-themed keywords (`lawa` = car, `kucha` = cdr, `upa` = cons). Derived from Rob Pike's pedagogical LISP. Rewritten from Go to Rust.

## STRUCTURE
```
.
├── Cargo.toml       # Rust project manifest
├── Cargo.lock       # Dependency lock file
├── src/             # Library source code
│   ├── lib.rs       # Library entry point (pub modules)
│   ├── main.rs      # CLI binary entry point
│   ├── token.rs     # Lexer and Token types
│   ├── parser.rs    # Parser (S-expression parser)
│   ├── expr.rs      # Expr enum (Nil, Atom, Cons)
│   ├── eval.rs      # Evaluator (Context, Scope, apply)
│   └── elementary.rs # Built-in functions (lawa, kucha, celi, etc.)
├── tests/           # Integration tests
│   ├── parse_tests.rs # Parser unit tests (ported from Go)
│   └── eval_tests.rs  # Evaluator unit tests (ported from Go)
├── examples/        # .mita sample programs
├── .github/         # CI workflows
├── odomu.mita       # Standard library
└── AGENTS.md        # This file
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Add built-in function | `src/elementary.rs` + `src/eval.rs` | Register in `ELEMENTARY` HashMap, implement as `Context::xxx_func` |
| Change lexer/tokenizer | `src/token.rs` | `TokenType` enum and `Lexer` struct |
| Modify parser/AST | `src/parser.rs` + `src/expr.rs` | `Expr` enum and `Parser` methods |
| Change evaluation logic | `src/eval.rs` | `Context::eval()`, `Context::apply()`, stack management |
| Modify CLI behavior | `src/main.rs` | REPL, flags, file loading, panic recovery |
| Add language examples | `examples/*.mita` | MITA source files |
| Update stdlib | `odomu.mita` | Loaded at runtime by CLI |

## CONVENTIONS
- **Library + binary**: `src/lib.rs` exports public API; `src/main.rs` is the CLI binary
- **Panic-driven errors**: `errorf()` panics with `panic_any(Error(...))`; CLI recovers via `catch_unwind`
- **MITA naming**: Built-ins use fictional language names (`celi` = +, `movo` = -, `shato` = ==)
- **No external dependencies**: Standard library only (no `cargo` dependencies)
- **Expr enum**: Uses `Rc<Expr>` for shared ownership (no GC, reference counted)
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
```

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
- Self-hosted `riscv-builders` runner used in CI (non-standard)
- `odomu.mita` uses Hilichurlian names for all library functions (not English)
- `eval_condition` treats the last clause as an implicit else (returns unevaluated if no remaining clauses)
- `shato` uses structural equality via `equal_expr` (not numeric equality)
- `nil` and `nya` are treated as equal in `equal_expr` (matching Go's `isNya()` behavior)
- `kuzi`/`todo` are variadic via `mita args` pattern (single atom formal captures entire arg list)
- `ELEMENTARY` uses `std::sync::OnceLock` (no unsafe code)
