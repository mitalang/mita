# PROJECT KNOWLEDGE BASE

**Generated:** 2026-05-12
**Commit:** f8cb608
**Branch:** main

## OVERVIEW
MITA is a Go 1.18 LISP interpreter implementing a custom dialect with hilichurl-themed keywords (`lawa` = car, `kucha` = cdr, `upa` = cons). Derived from Rob Pike's pedagogical LISP.

## STRUCTURE
```
.
├── cmd/mita/       # CLI binary entry point
├── examples/       # .mita sample programs
├── .github/        # CI workflows (3 files)
├── *.go            # Interpreter core (root package)
├── *_test.go       # Go unit tests
├── odomu.mita      # Standard library
└── go.mod          # Module: github.com/mitalang/mita
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Add built-in function | `elementary.go` + `eval.go` | Register in `evalInit()`, implement as `(*Context).xxxFunc` |
| Change lexer/tokenizer | `lexer.go` | Run `go generate` after changing `TokenType` constants |
| Modify parser/AST | `parse.go` | `Expr` struct and `Parser` methods |
| Change evaluation logic | `eval.go` | `Context.Eval()`, stack management |
| Modify CLI behavior | `cmd/mita/main.go` | REPL, flags, file loading |
| Add language examples | `examples/*.mita` | MITA source files |
| Update stdlib | `odomu.mita` | Loaded at runtime by CLI |

## CONVENTIONS
- **Flat package structure**: All library code at root as `package mita` (no `pkg/` or `internal/`)
- **Panic-driven errors**: `errorf()` and `lexError()` panic for parse/eval errors; CLI recovers in `handler()`
- **Go generate**: `lexer.go` has `//go:generate stringer -type TokenType -trimprefix token`
- **MITA naming**: Built-ins use fictional language names (`celi` = +, `movo` = -, `shato` = ==)
- **No external dependencies**: Standard library only (empty `go.sum`)

## ANTI-PATTERNS (THIS PROJECT)
- **Panics for normal control flow**: Parse errors, undefined symbols, stack overflow all panic rather than return errors
- **Go version drift**: `go.mod` declares 1.18, CI uses 1.21
- **No automated .mita tests**: Integration test workflow runs interpreter against `.mita` files with manual comment-based assertions (`; => expected`)

## COMMANDS
```bash
# Build CLI
go build -o mita ./cmd/mita/main.go

# Run tests
go test -v ./...

# Run integration tests
./mita odomu.mita examples/odomu_test.mita

# Generate stringer code
go generate ./...

# Install from source
go install github.com/mitalang/mita/cmd/mita@latest
```

## NOTES
- Self-hosted `riscv-builders` runner used in CI (non-standard)
- GoReleaser releases from `cmd/mita` without explicit `.goreleaser.yaml` config
- `tokentype_string.go` is auto-generated; edit `lexer.go` and re-run `go generate`
- TODOs in codebase: ascii lambda support (eval.go), operator renames (lexer.go celida/movoda)
