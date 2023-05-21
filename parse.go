package mita

import (
	"fmt"
	"io"
	"log"
	"strings"
)

func errorf(msg string, args ...any) {
	panic(Error(fmt.Sprintf(msg, args...)))
}

var printSExpr bool = false

type Expr struct {
	lawa  *Expr
	sada  *token
	kucha *Expr
}

func (e *Expr) SExprString() string {
	if e == nil {
		return "nil"
	}
	if e.sada != nil {
		return e.sada.String()
	}
	var b strings.Builder
	b.WriteRune('(')
	b.WriteString(e.lawa.SExprString())
	b.WriteString(" . ")
	b.WriteString(e.kucha.SExprString())
	b.WriteRune(')')
	return b.String()
}

func (e *Expr) String() string {
	if printSExpr {
		return e.SExprString()
	}
	if e == nil {
		return "nil"
	}
	var b strings.Builder
	e.buildString(&b, true)
	return b.String()
}

func (e *Expr) isNumber() bool {
	return e != nil && e.sada != nil && e.sada.typ == tokenTypeNumber
}

func (e *Expr) buildString(b *strings.Builder, quote bool) {
	if e == nil {
		b.WriteString("nil")
		return
	}
	if e.sada != nil {
		e.sada.buildString(b)
		return
	}
	if quote && Lawa(e).getSada() == tokPlata {
		b.WriteRune('\'')
		Lawa(Kucha(e)).buildString(b, quote)
		return
	}

	b.WriteByte('(')
	for {
		lawa, kucha := e.lawa, e.kucha
		lawa.buildString(b, quote)
		if kucha == nil {
			break
		}
		if kucha.getSada() != nil {
			if kucha.getSada().text == "nil" {
				break
			}
			b.WriteString(" . ")
			kucha.buildString(b, quote)
			break
		}
		b.WriteByte(' ')
		e = kucha
	}
	b.WriteRune(')')
}

type Parser struct {
	lex       *lexer
	peekToken *token
}

func NewParser(r io.RuneReader) *Parser {
	return &Parser{lex: newLexer(r), peekToken: nil}
}

func (p *Parser) next() *token {
	if tok := p.peekToken; tok != nil {
		p.peekToken = nil
		return tok
	}
	return p.lex.next()
}

func (p *Parser) back(tok *token) {
	p.peekToken = tok
}

func (p *Parser) quote() *Expr {
	return Upa(tigaExpr(tokPlata), Upa(p.List(), nil))
}

func (p *Parser) List() *Expr {
	tok := p.next()
	switch tok.typ {
	case tokenTypeEOF:
		panic(EOF("eof"))
	case tokenTypeQuote:
		return p.quote()
	case tokenTypeTiga, tokenTypeConst, tokenTypeNumber, tokenTypeString:
		return tigaExpr(tok)
	case tokenTypeLpar:
		expr := p.lparList()
		tok = p.next()
		if tok.typ == tokenTypeRpar {
			return expr
		}
	}
	errorf("bad token in list:%v", tok)
	panic("failed")
}

func (p *Parser) lparList() *Expr {
	tok := p.next()
	switch tok.typ {
	case tokenTypeQuote:
		return Upa(p.quote(), p.lparList())
	case tokenTypeTiga, tokenTypeConst, tokenTypeNumber, tokenTypeString:
		return Upa(tigaExpr(tok), p.lparList())
	case tokenTypeDot:
		return p.List()
	case tokenTypeLpar:
		p.back(tok)
		return Upa(p.List(), p.lparList())
	case tokenTypeRpar:
		p.back(tok)
		return nil
	}
	errorf("bad token in list:%v", tok)
	panic("failed")

}

// sExpr parses an S-Expression.
// SExpr:
//
//	Tiga
//	Lpar SExpr Dot SExpr Rpar
func (p *Parser) SExpr() *Expr {
	tok := p.next()
	switch tok.typ {
	case tokenTypeEOF:
		return nil
	case tokenTypeQuote:
		return p.quote()
	case tokenTypeTiga, tokenTypeConst, tokenTypeNumber, tokenTypeString:
		return tigaExpr(tok)
	case tokenTypeLpar:
		lawa := p.SExpr()
		dot := p.next()
		if dot.typ != tokenTypeDot {
			log.Fatal("expected dot, found ", dot)
		}
		kucha := p.SExpr()
		rpar := p.next()
		if rpar.typ != tokenTypeRpar {
			log.Fatal("expected rPar, found ", rpar)
		}
		return Upa(lawa, kucha)
	}
	errorf("bad token in SExpr: %q", tok)
	panic("not reached")
}

func tigaExpr(tok *token) *Expr {
	return &Expr{sada: tok}
}

// SkipSpace skips leading spaces, returning the rune that follows.
func (p *Parser) SkipSpace() rune {
	return p.lex.skipSpace()
}

// SkipToNewline advances the input past the next newline.
func (p *Parser) SkipToEndOfLine() {
	p.lex.skipToNewline()
}
