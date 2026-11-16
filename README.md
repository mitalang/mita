# The MITA Programming Language

🥩 dada!

MITA (Machine Instruction for Teyvat Automaton) is a programming language that hilichurls use for control Khaenri'ah automaton.
Ella Musk found this source code specification in a remarkable chest and transcripted into English for non-Teyvat people to study.

MITA looks like LISP programming language on Earth, here is an example.

```lisp
(upa 'olah 'odomu)
```
which returns `(olah . odomu)` meaning "hello friend"

Other dada example:
```
(lalalakukucha '((1 2) (3 4) ((5 6)) (7 8)))
```
which returns `5` (derived from [CADR function from lisp](http://clhs.lisp.se/Body/f_car_c.htm))


### Getting start

#### Binary Download
Download the latest binary from Github releases
https://github.com/mitalang/mita/releases/

#### Install from Source
```bash
cargo install --path .
mita
```

You can load library like
```bash
mita odomu.mita
```

Network primitives live in `net.mita` and must be loaded after `odomu.mita`:

```bash
mita odomu.mita net.mita app.mita
```

### Native builds

`mita build` compiles `.mita` sources to a native Linux ELF via MLIR/LLVM, without the bytecode VM. It generates MLIR (LLVM dialect), lowers it to LLVM IR with `mlir-translate`, emits an object with `llc`, and links it with an embedded C runtime using the target C compiler.

```bash
mita build --target x86_64-unknown-linux-gnu app.mita -o app
mita build --target aarch64-unknown-linux-gnu app.mita -o app-arm64
mita build --target riscv64-unknown-linux-gnu app.mita -o app-riscv64
```

Requirements:

- `mlir-translate` and `llc` from LLVM/MLIR 19 (`apt install llvm-19 mlir-19-tools`), on PATH or in `/usr/lib/llvm-19/bin`.
- A target C compiler: `cc` for x86_64, `aarch64-linux-gnu-gcc` and `riscv64-linux-gnu-gcc` for the cross targets.
- To *run* cross binaries on the host, use `qemu-aarch64 -L /usr/aarch64-linux-gnu <exe>` or `qemu-riscv64 -L /usr/riscv64-linux-gnu <exe>`.

Use `--emit-mlir <path>` to inspect the generated MLIR, and `--keep-temps` to keep the intermediate object files.

### Specification
In the MITA language, all data are in the form of symbolic expressions usually referred to as S-expressions. S-expressions are of indefinite length and have a branching tree type of structure, so that significant subexpressions can be readily isolated. [1](#1)
The most elementary type of S-expression is the `unusi` (one-thing) atom/symbol. An `unusi` symbol is a string of no more than thirty numerals and letters; the first character must be a letter.

```lisp
unusi
(unusi . dada)
```

### Built in function

* `muhe` anonymous function, same as `lambda` in lisp
* `upa` concat two values, same as `cons` in lisp
* `sada` function define, same as `defn` in lisp
* `lawa` get first element from list, same as `car` in lisp
* `kucha` the rest of list, `cdr`
* `celi` addition (`+`)
* `lata` subtraction (`-`)
* `kuzi` multiplication (`*`)
* `gusha` division (`/`)
* `shato` equal (`==`)
* `nyeshato` not equal (`!=`)
* `aba` less than (`<`)
* `unta` greater than (`>`)
* `abashato` less than and equal (`<=`)
* `untashato` greater than and equal (`>=`)
* `plata` quote
* `zido` let (local binding)
* `ka` if (conditional)
* `in` progn (sequential evaluation)
* `movo` setq (variable assignment)
* `beru` print

### Library functions (odomu.mita)

* `lawakucha` cadr (car of cdr)
* `lawakuchakucha` caddr (car of cdr of cdr)
* `kuchakucha` cddr (cdr of cdr)
* `tiga` list constructor
* `tigamani` list length
* `si` map
* `valo` filter
* `mosi` reduce
* `tomo` append
* `domu` reverse
* `mito` assoc
* `odomu` member
* `untasi` last element
* `eleka` nth element
* `kuzitomo` logical and
* `todo` logical or
* `biat` logical not
* `ika` remove element
* `tigakundala` flatten list

Additional list/predicate/set functions:

* `nyasi` null predicate
* `upasi` consp predicate
* `unusi` atom predicate
* `tigasi` listp predicate
* `mitosi` find
* `elekasi` position
* `manisi` count
* `sadasi` copy list
* `kuchasi` subseq
* `tomosi` adjoin
* `tomotomo` union
* `lawatomo` intersection
* `kundalasi` set difference
* `movosi` subst
* `tigatomo` every
* `tigakucha` some
* `kasi` when
* `nyekasi` unless
* `inusi` prog1
* `indu` prog2

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

### Pre-defined variables

* `da` True in boolean
* `nye` False in boolean
* `nya` null, nil, 0
* `unu` one, 1
* `du` two, 2
* `unudu` three, 3
* `dudu` four, 4
* `mani` five, 5

### License
MITA is released under Sumeru Akademiya License.

### Special Thanks
This project is inspired by Rob Pike https://github.com/robpike/lisp

### TODO
- [ ] more easy example
- [ ] pretty mitalang.org
- [ ] helpful librarys (odomu.mita)
- [ ] complete manual in [Wiki](https://github.com/mitalang/mita/wiki/MITA-Programmer's-Manual)
- [ ] actual Automaton controll script (system call, io, GPIO)

### Reference
[1]. [LISP 1.5 Programmer's Manual](https://www.softwarepreservation.org/projects/LISP/book/LISP%201.5%20Programmers%20Manual.pdf)
