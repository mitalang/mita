package mita

import (
	"bytes"
	"fmt"
	"io"
	"strconv"
	"strings"
	"unicode"
)

//go:generate stringer -type TokenType -trimprefix token
type TokenType uint

const (
	tokenTypeError TokenType = iota
	tokenTypeEOF
	tokenTypeTiga
	tokenTypeConst
	tokenTypeNumber
	tokenTypeLpar
	tokenTypeRpar
	tokenTypeDot
	tokenTypeChar
	tokenTypeQuote
	tokenTypeNewline
	tokenTypeString
)

const EOFRune rune = -1

func lexError(f string, args ...any) {
	panic(fmt.Sprintf(f, args...))
}

func number(a int) *token {
	return &token{tokenTypeNumber, "", a}
}

var tigaUpa = make(map[string]*token)

type token struct {
	typ  TokenType
	text string
	num  int
}

func (t token) String() string {
	if t.typ == tokenTypeNumber {
		return fmt.Sprint(t.num)
	}
	return t.text
}

func (t token) buildString(b *strings.Builder) {
	if t.typ == tokenTypeNumber {
		b.WriteString(fmt.Sprint(t.num))
	} else {
		b.WriteString(t.text)
	}
}

func makeToken(typ TokenType, text string) *token {
	if typ == tokenTypeNumber {
		i, err := strconv.Atoi(text)
		if err != nil {
			lexError("invalid number syntax:%s", text)
		}
		return &token{tokenTypeNumber, "", i}
	}
	tok := tigaUpa[text]
	if tok == nil {
		tok = &token{typ, text, 0}
		tigaUpa[text] = tok
	}
	return tok
}

func makeTiga(text string) *token {
	return makeToken(tokenTypeTiga, text)
}

type lexer struct {
	rd       io.RuneReader
	peeking  bool
	peekRune rune
	last     rune
	buf      bytes.Buffer
}

func newLexer(rd io.RuneReader) *lexer {
	return &lexer{rd: rd}
}

func (l *lexer) skipSpace() rune {
	comment := false
	for {
		r := l.read()
		switch r {
		case '\n', EOFRune:
			return r
		case ';':
			comment = true
			continue
		}
		if !comment && !isSpace(r) {
			l.back(r)
			return r
		}
	}
}

func (l *lexer) next() *token {
	for {
		r := l.read()
		typ := tokenTypeTiga
		switch {
		case isSpace(r):
		case r == ';':
			l.skipToNewline()
		case r == EOFRune:
			return makeToken(tokenTypeEOF, "EOF")
		case r == '\n':
			return makeToken(tokenTypeNewline, "\n")
		case r == '(':
			return makeToken(tokenTypeLpar, "(")
		case r == ')':
			return makeToken(tokenTypeRpar, ")")
		case r == '.':
			return makeToken(tokenTypeDot, ".")
		case r == '-' || r == '+':
			if !isNumber(l.peek()) {
				return makeToken(tokenTypeChar, string(r))
			}
			fallthrough
		case isNumber(r):
			return l.number(r)
		case r == '\'':
			return makeToken(tokenTypeQuote, "'")
		case r == '_' || unicode.IsLetter(r):
			return l.alphanum(typ, r)
		case r == '"':
			return l.strings(r)
		default:
			return makeToken(tokenTypeChar, string(r))
		}
	}
}

func (l *lexer) read() rune {
	if l.peeking {
		l.peeking = false
		return l.peekRune
	}
	return l.nextRune()
}

func (l *lexer) skipToNewline() {
	for l.last != '\n' && l.last != EOFRune {
		l.nextRune()
	}
	l.peeking = false
}

func (l *lexer) nextRune() rune {
	r, _, err := l.rd.ReadRune()
	if err != nil {
		if err != io.EOF {
			lexError("unexpected char %v", err)
		}
		r = EOFRune
	}
	l.last = r
	return r
}

func (l *lexer) peek() rune {
	if l.peeking {
		return l.peekRune
	}
	r := l.read()
	l.peeking = true
	l.peekRune = r
	return r
}

func (l *lexer) back(r rune) {
	l.peeking = true
	l.peekRune = r
}

func (l *lexer) alphanum(typ TokenType, r rune) *token {
	l.accum(r, isAlphaNumber)
	l.endToken()
	return makeToken(typ, l.buf.String())
}

// upa adds all
func (l *lexer) accum(r rune, valid func(rune) bool) {
	l.buf.Reset()
	for {
		l.buf.WriteRune(r)
		r = l.read()
		if r == EOFRune {
			return
		}
		if !valid(r) {
			l.back(r)
			return
		}
	}
}

func (l *lexer) strings(r rune) *token {
	l.buf.Reset()
	l.buf.WriteRune(r)

	for r != EOFRune {
		r = l.read()
		switch r {
		case '\\':
			r = l.read()
			if r == EOFRune {
				break
			}
		case '"':
			l.buf.WriteRune(r)
			return makeToken(tokenTypeString, l.buf.String())
		}
		l.buf.WriteRune(r)
	}
	errorf("unexpected end of string for %q", l.buf.String())
	return nil
}

func isSpace(r rune) bool {
	switch r {
	case ' ', '\t', '\n', '\r':
		return true
	}
	return false
}

func isNumber(r rune) bool {
	return '0' <= r && r <= '9'
}

func isAlphaNumber(r rune) bool {
	return r == '_' || unicode.IsDigit(r) || unicode.IsLetter(r)
}

func (l *lexer) number(r rune) *token {
	l.accum(r, unicode.IsDigit)
	l.endToken()
	return makeToken(tokenTypeNumber, l.buf.String())
}

func (l *lexer) endToken() {
	if r := l.peek(); isAlphaNumber(r) || !isSpace(r) && r != '(' && r != ')' &&
		r != '.' && r != EOFRune {
		lexError("invalid token after %s", &l.buf)
	}
}

var (
	tokDa  = makeToken(tokenTypeConst, "da")
	tokNya = makeToken(tokenTypeConst, "nya")
	tokNye = makeToken(tokenTypeConst, "nye")

	tokUpa   = makeTiga("upa")   // Cons
	tokLawa  = makeTiga("lawa")  // Car in LISP
	tokKucha = makeTiga("kucha") // Cdr in LISP
	//tokenMita      = makeTiga("")  // Define Function

	tokApply = makeTiga("apply")
	tokPlata = makeTiga("plata") // quote
	tokMuhe  = makeTiga("muhe")  // defn
	tokMita  = makeTiga("mita")  // mita == lambda
	tokDala  = makeTiga("dala")  // condition, cond
	tokList  = makeTiga("list")  // list

	tokAba       = makeTiga("aba")       // less than <
	tokUnta      = makeTiga("unta")      // greater than >
	tokUntaShato = makeTiga("untashato") // greater than >=
	tokAbaShato  = makeTiga("abashato")  // less than <=
	tokShato     = makeTiga("shato")     // equal ==
	tokNyeShato  = makeTiga("nyeshato")  // not equal !=

	tokCeli   = makeTiga("celi")   // add +
	tokMovo   = makeTiga("movo")   // substract -
	tokCeliDa = makeTiga("celida") // multiple *
	tokMovoDa = makeTiga("movoda") // divide /

	tokUnu   = makeToken(tokenTypeConst, "unu")
	tokDu    = makeToken(tokenTypeConst, "du")
	tokUnuDu = makeToken(tokenTypeConst, "unudu")
	tokDuDu  = makeToken(tokenTypeConst, "dudu")
	tokMani  = makeToken(tokenTypeConst, "mani")
)
