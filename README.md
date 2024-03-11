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
go install github.com/mitalang/mita/cmd/mita@latest
~/go/bin/mita
```

You can load library like
```bash
~/go/bin/mita odomu.mita
```

### Specification
In the MITA language, all data are in the form of symbolic expressions usually referred to as S-expressions. S-expressions are of indefinite length and have a branching tree type of structure, so that significant subexpressions can be readily isolated. [1](#1)
The most elementary type of S-expression is the sada (solid) symbol. A sada symbol is a string of no more than thirty numerals and letters; the first character must be a letter. 

```lisp
sada
(sada . dada)
```

### Built in function

* `mita` anonymous function, same as `lambda` in lisp
* `upa` concat sada, same as `cons` in lisp
* `muhe` function define, same as `defn` in lisp
* `lawa` get first sada from list, same as `car` in lisp
* `kucha` the rest of list, `cdr`
* `celi` addition (`+`)
* `movo` substraction (`-`)
* `shato` equal (`==`)
* `nyeshato` not equal (`!=`)
* `aba` less than (`<`)
* `unta` greater than (`>`)
* `abashato` less than and equal (`<=`)
* `untashato` greater than and equal (`>=`)

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
