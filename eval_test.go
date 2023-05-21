package mita

import (
	"strings"
	"testing"
)

func TestIsLawaKucha(t *testing.T) {
	for _, tc := range []struct {
		gold   string
		expect bool
	}{
		{"lakuwa", false},
		{"lalawa", true},
		{"lakukukuwa", false},
		{"lalalalawa", true},
		{"lalakulawa", true},
		{"kulakucha", true},
		{"lalwa", false},
	} {

		if got := isLaKucha(tc.gold); tc.expect != got {
			t.Errorf("%s expect %v got %v", tc.gold, tc.expect, got)
		}
	}
}

var consTests = []struct {
	a, b string
	c    string
}{
	{"a", "b", "(a . b)"},
	{"(a . b)", "c", "((a . b) . c)"},
	{"a", "(b . c)", "(a . (b . c))"},
}

func TestUpa(t *testing.T) {
	for _, test := range consTests {
		a := NewParser(strings.NewReader(test.a)).SExpr()
		b := NewParser(strings.NewReader(test.b)).SExpr()
		c := Upa(a, b)
		str := c.SExprString()
		if str != test.c {
			t.Errorf("upa(%s, %s) = %s, expected %s", test.a, test.b, str, test.c)
		}
	}
}

func strEval(str string, t *testing.T) string {
	p := NewParser(strings.NewReader(str))
	list := p.List()
	return NewContext(0).Eval(list).String()
}

var upaEvalTests = []struct {
	in  string
	out string
}{
	// A nice little set from
	// https://medium.com/@aleksandrasays/my-other-car-is-a-cdr-3058e6743c15
	{"(upa 1 2)", "(1 . 2)"},
	{"(upa 'a (upa 'b (upa 'c '())))", "(a b c)"},
	{"(list 'a 'b 'c)", "(a b c)"}, // Original has c, not 'c, can't be right.
	{"(upa 1 '(2 3 4))", "(1 2 3 4)"},
	{"(upa '(a b c) ())", "((a b c))"},
	{"(upa '(a b c) '(d))", "((a b c) d)"},

	{"unu", "1"},
	{"du", "2"},
	{"unudu", "3"},
	{"dudu", "4"},
	{"mani", "5"},
}

func TestUpaEval(t *testing.T) {
	for _, test := range upaEvalTests {
		if got := strEval(test.in, t); got != test.out {
			t.Errorf("%s = %s, expected %s", test.in, got, test.out)
		}
	}
}

var stringTests = []struct {
	in  string
	out string
}{
	{`"ohla odomu!"`, `"ohla odomu!"`},
}

func TestStrings(t *testing.T) {
	for _, test := range stringTests {
		if got := strEval(test.in, t); got != test.out {
			t.Errorf("%s = %s, expected %s", test.in, got, test.out)
		}
	}
}

var condEvalTests = []struct {
	in  string
	out string
}{
	{"(celi 3 2)", "5"},
	{"(movo 3 2)", "1"},

	{"(celida 10 2)", "20"},
	{"(movoda 6 3)", "2"},

	{"(aba 2 3)", "da"},
	{"(dala ((aba 2 3) 'UNTA) (da 'ABA))", "UNTA"},
	{"(aba 3 2)", "nye"},
	{"(dala ((aba 3 2) 'UNTA) (da 'ABA))", "ABA"},

	{"(unta 3 2)", "da"},
	{"(dala ((unta 3 2) 'UNTA) (da 'ABA))", "UNTA"},
	{"(unta 2 3)", "nye"},
	{"(dala ((unta 2 3) 'UNTA) (da 'ABA))", "ABA"},

	{"(dala ((shato 6 3) 'DA) (da 'NYE))", "NYE"},
	{"(dala ((shato 3 3) 'DA) (da 'NYE))", "DA"},

	{"(dala ((nyeshato 6 3) 'DA) (da 'NYE))", "DA"},
	{"(dala ((nyeshato 3 3) 'DA) (da 'NYE))", "NYE"},
}

func TestCondEval(t *testing.T) {
	for _, test := range condEvalTests {
		if got := strEval(test.in, t); got != test.out {
			t.Errorf("%s = %s, expected %s", test.in, got, test.out)
		}
	}
}

func TestApply(t *testing.T) {
	l := "(mita (x y) (upa (lawa x) y))"
	lambda := NewParser(strings.NewReader(l)).List()
	a := "((a b) (c d))"
	args := NewParser(strings.NewReader(a)).List()
	c := NewContext(0)
	expr := c.apply(l, lambda, args)
	const want = "(a c d)"
	if expr.String() != want {
		t.Fatal(expr)
	}
}

var examples = []struct {
	name string
	fn   string
	in   string
	out  string
}{
	{
		"(yafib)",
		`(muhe(
			(yafib (mita (si) 
				(dala ((shato si 0) 0)
					(da (dala ((aba si du) unu)
						(da (celi (yafib(movo si du)) (yafib(movo si unu))))
					))
				)
			))
		))`,
		"(yafib 10)",
		"55",
	},
	{
		"(testlalalakukucha)",
		`(muhe(
			(testlalalakukucha (mita (si) (lalalakukucha si)))
		))`,
		"(testlalalakukucha '((1 2) (3 4) ((5 6)) (7 8)))",
		"5",
	},
	//(da (celi (yafib (movo si unu)) (yafib (movo si du)))))
}

func TestExample(t *testing.T) {
	for _, test := range examples {
		c := NewContext(0)
		p := NewParser(strings.NewReader(test.fn))
		l := p.List()
		t.Log(l)
		if got := c.Eval(l).String(); got != test.name {
			t.Errorf("%s = %s, expected %s", test.fn, got, test.name)
		}
		p = NewParser(strings.NewReader(test.in))
		l = p.List()
		t.Log(l)
		if got := c.Eval(l).String(); got != test.out {
			t.Errorf("%s = %s, expected %s", test.in, got, test.out)
		}
	}
}

func TestStackTrace(t *testing.T) {
	const prog = `
	(muhe(
                (error (mita (x) 
			(dala ((shato x 0) (movoda 0 0))
                        	(da (error (movo x 1)))
                	)
		))
        ))`
	const crash = `(error 5)`
	c := NewContext(0)
	p := NewParser(strings.NewReader(prog))
	if got := c.Eval(p.List()).String(); got != "(error)" {
		t.Fatal("did not declare error")
	}
	p = NewParser(strings.NewReader(crash))
	defer func() {
		e := recover()
		_, ok := e.(Error)
		if !ok {
			t.Fatal("no error")
		}
		const expect = "stack: (error 0) (error 1) (error 2) (error 3) (error 4) (error 5)"
		stack := c.StackTrace()
		if strings.Join(strings.Fields(stack), " ") != expect {
			t.Fatal(stack)
		}
	}()
	c.Eval(p.List())
	t.Fatal("did not crash")
}
