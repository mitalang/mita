package mita

import (
	"strings"
	"testing"
)

var parseTests = []struct {
	s string
	l string
}{
	{"nil", "nil"},
	{"a", "a"},
	{"(a . nil)", "(a)"},
	{"(a . b)", "(a . b)"},
	{"(a . (b . nil))", "(a b)"},
	{"((a . nil) . nil)", "((a))"},
	{"(a . (b . (c . nil)))", "(a b c)"},
	{"(a . (b . (c . (d . nil))))", "(a b c d)"},
	{"(a . (b . (c . (d . (e . nil)))))", "(a b c d e)"},
	{"((a . (b . nil)) . (c . nil))", "((a b) c)"},
	{"(a . (b . ((c . (d . nil)) . nil)))", "(a b (c d))"},
	{"(a . ((b . c) . nil))", "(a (b . c))"},
}

func TestSExprParse(t *testing.T) {
	for _, test := range parseTests {
		p := NewParser(strings.NewReader(test.s))
		expr := p.SExpr()
		str := expr.SExprString()
		if str != test.s {
			t.Errorf("%q.SExprString() = %q", test.s, str)
		}
		str = expr.String()
		if str != test.l {
			t.Errorf("%q.String() = %q, expected %q", test.s, str, test.l)
		}
	}
}

func TestListParse(t *testing.T) {
	for _, test := range parseTests {
		t.Log(test.l)
		p := NewParser(strings.NewReader(test.l))
		expr := p.List()
		str := expr.SExprString()
		if str != test.s {
			t.Errorf("%q.SExprString() = %q; expected %q", test.l, str, test.s)
		}
		str = expr.String()
		if str != test.l {
			t.Errorf("%q.String() = %q, expected %q", test.l, str, test.l)
		}
	}
}

var quoteTests = []struct {
	l         string
	s         string
	quoted    string
	nonquoted string
}{
	{"()", "nil", "nil", "nil"}, // Do () while we're here; it's not a valid SExpr.
	{"a", "a", "a", "a"},
	{"'a", "(plata . (a . nil))", "'a", "(plata a)"},
	{"'(a)", "(plata . ((a . nil) . nil))", "'(a)", "(plata (a))"},
	{"''a", "(plata . ((plata . (a . nil)) . nil))", "''a", "(plata (plata a))"},
	{"''(a)", "(plata . ((plata . ((a . nil) . nil)) . nil))", "''(a)", "(plata (plata (a)))"},
	{"('a 'b 'c)", "((plata . (a . nil)) . ((plata . (b . nil)) . ((plata . (c . nil)) . nil)))", "('a 'b 'c)", "((plata a) (plata b) (plata c))"},
}

func (e *Expr) stringNoQuote() string {
	var b strings.Builder
	e.buildString(&b, false)
	return b.String()
}

func TestParseQuote(t *testing.T) {
	for _, test := range quoteTests {
		t.Log(test.l)
		p := NewParser(strings.NewReader(test.l))
		expr := p.List()
		str := expr.SExprString()
		if str != test.s {
			t.Errorf("%q.SExprString() = %q", test.s, str)
		}
		str = expr.String()
		if str != test.quoted {
			t.Errorf("%q.String() = %q, expected %q", test.l, str, test.quoted)
		}
		str = expr.stringNoQuote()
		if str != test.nonquoted {
			t.Errorf("%q.stringNoQuote() = %q, expected %q", test.l, str, test.nonquoted)
		}
	}
}
