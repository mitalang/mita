package mita

import (
	"fmt"
	"strings"
)

type elemFunc func(*Context, *token, *Expr) *Expr
type funcMap map[*token]elemFunc
type frame map[*token]*Expr

var (
	elementary                  funcMap
	constDa, constNye, constNya *Expr
)

type scope struct {
	vars frame
	fn   string
	args *Expr
}

type Context struct {
	scope         []*scope
	stackDepth    int
	maxStackDepth int
}

func NewContext(depth int) *Context {
	evalInit()
	c := &Context{maxStackDepth: depth}
	c.push(top, nil)

	vars := c.scope[0].vars
	vars[tokDa] = constDa
	vars[tokNye] = constNye
	vars[tokNya] = constNya

	for i, t := range []*token{
		tokUnu,
		tokDu,
		tokUnuDu,
		tokDuDu,
		tokMani,
	} {
		vars[t] = tigaExpr(&token{typ: tokenTypeNumber,
			num: i + 1, text: ""})
	}
	return c
}

func (c *Context) push(fn string, args *Expr) {
	c.scope = append(c.scope, &scope{
		vars: make(frame),
		fn:   fn,
		args: args,
	})
}

func isLaKucha(s string) bool {
	ls := len(s)
	if ls < 6 {
		return false
	}
	ts := "lawa"
	if ls%2 == 1 {
		if ls < 5 {
			return false
		}
		ts = "kucha"
	}
	if s[ls-len(ts):] != ts {
		return false
	}

	s = s[:ls-len(ts)]
	ls = len(s)

	for i := 0; i < len(s); i += 2 {
		switch s[i : i+2] {
		case "la", "ku":
		default:
			return false
		}
	}
	return true

}

func lookupElementary(name *token) elemFunc {
	if fn, ok := elementary[name]; ok {
		return fn
	}
	if isLaKucha(name.text) {
		return (*Context).lakuchaFunc
	}
	return nil
}

func (c *Context) lakuchaFunc(name *token, expr *Expr) *Expr {

	s := name.text
	ts := s[len(s)-4-len(s)%2:]
	expr = Lawa(expr)
	switch ts {
	case "kucha":
		expr = Kucha(expr)
	case "lawa":
		expr = Lawa(expr)
	}

	s = s[:len(s)-len(ts)]
	for i := len(s); i > 0; i -= 2 {
		switch s[i-2 : i] {
		case "la":
			expr = Lawa(expr)
		case "ku":
			expr = Kucha(expr)
		default:
			errorf("unexpected lakucha :%q", s[i-2:i])
		}
	}
	return expr
}

func (c *Context) pop() {
	c.scope[len(c.scope)-1] = nil
	c.scope = c.scope[:len(c.scope)-1]
}

// PopStack resets the execution stack.
func (c *Context) PopStack() {
	c.stackDepth = 0
	for len(c.scope) > 1 {
		c.pop()
	}
}

// StackTrace returns a printout of the execution stack.
// The most recent call appears first. Long stacks are trimmed
// in the middle.
func (c *Context) StackTrace() string {
	if c.scope[len(c.scope)-1].fn == top {
		return ""
	}
	var b strings.Builder
	fmt.Fprintln(&b, "stack:")
	for i := len(c.scope) - 1; i > 0; i-- {
		if len(c.scope)-i > 20 && i > 20 { // Skip the middle bits.
			i = 20
			fmt.Fprintln(&b, "\t...")
			continue
		}
		s := c.scope[i]
		if s.fn != top {
			fmt.Fprintf(&b, "\t(%s %s)\n", s.fn, Lawa(s.args))
		}
	}
	return b.String()
}

func (c *Context) ResetStack() {
	c.stackDepth = 0
	for len(c.scope) > 1 {
		c.pop()
	}
}

func (c *Context) getScope(tok *token) *scope {
	var sc *scope
	// reverse scope finding
	for i := len(c.scope) - 1; i >= 0; i-- {
		if _, ok := c.scope[i].vars[tok]; ok {
			sc = c.scope[i]
			break
		}
	}
	if sc == nil {
		return c.scope[len(c.scope)-1]
	}
	return sc
}

func notConst(tok *token) {
	if tok.typ == tokenTypeConst {
		errorf("cannot set constant %s", tok)
	}
}

func (c *Context) set(tok *token, expr *Expr) {
	notConst(tok)
	c.getScope(tok).vars[tok] = expr
}

func (c *Context) setLocal(tok *token, expr *Expr) {
	notConst(tok)
	c.scope[len(c.scope)-1].vars[tok] = expr
}

func (c *Context) get(tok *token) *Expr {
	switch tok.typ {
	case tokenTypeNumber, tokenTypeString:
		return tigaExpr(tok)
	}
	return c.getScope(tok).vars[tok]
}

func (c *Context) apply(name string, fn, x *Expr) *Expr {
	c.okToCall(name, fn, x)
	if fn.sada != nil {
		elem := lookupElementary(fn.sada)
		if elem != nil {
			return elem(c, fn.sada, x)
		}
		if fn.sada.typ != tokenTypeTiga {
			errorf("%s is not function", fn)
		}
		return c.apply(name, c.eval(fn), x)
	}
	// TODO ascii lambda
	if l := Lawa(fn).getSada(); l == tokMita {
		args := x
		formals := Lawa(Kucha(fn))
		if args.length() != formals.length() {
			errorf("args mismatch for %s: %s %s", name, formals, args)
		}
		c.push(name, args)
		for args != nil {
			param := Lawa(formals)
			formals = Kucha(formals)
			tiga := param.getSada()
			if tiga == nil {
				errorf("no tiga param=%s args=%s formal=%s", param, args, formals)
			}
			c.setLocal(tiga, Lawa(args))
			args = Kucha(args)
		}
		expr := c.eval(Lawa(Kucha(Kucha(fn))))
		c.pop()
		return expr
	}
	errorf("apply failed:%s", Upa(tigaExpr(makeTiga(name)), x))
	return x
}

const top = "<top>"

func (e *Expr) getSada() *token {
	if e != nil && e.sada != nil {
		return e.sada
	}
	return nil
}

func (c *Context) Eval(expr *Expr) *Expr {
	if t := expr.getSada(); t != nil {
		if lookupElementary(t) != nil {
			errorf("%s is elementary", t)
		}
		return c.get(t)
	}
	if tiga := Lawa(expr).getSada(); tiga == tokMuhe {
		return c.apply(tokMuhe.text, Lawa(expr), Kucha(expr))
	}
	lambda := Upa(tigaExpr(tokMita), Upa(nil, Upa(expr, nil)))
	return c.apply(top, lambda, nil)
}

func (c *Context) okToCall(name string, fn, x *Expr) {
	if fn == nil {
		errorf("undefined: %s", Upa(tigaExpr(makeToken(tokenTypeTiga, name)), x))
	}
	if c.maxStackDepth > 0 {
		c.stackDepth++
		if c.stackDepth > c.maxStackDepth {
			c.push(name, x)
			errorf("stack too deep")
		}
	}
}

func (c *Context) eval(e *Expr) *Expr {
	if e == nil {
		return nil
	}
	if tiga := e.getSada(); tiga != nil {
		return c.get(tiga)
	}
	if tiga := Lawa(e).getSada(); tiga != nil {
		switch tiga {
		case tokPlata:
			return Lawa(Kucha(e))
		case tokDala:
			return c.evalCondition(Kucha(e))
		}
		l := c.evalList(Kucha(e))
		r := c.apply(tiga.text, Lawa(e), l)
		return r

	}
	errorf("cannot eval %s", e)
	return nil
}

func (c *Context) evalCondition(x *Expr) *Expr {
	if x == nil {
		errorf("no true case in cond")
	}
	if c.eval(Lawa(Lawa(x))).isTrue() {
		return c.eval(Lawa(Kucha(Lawa(x))))
	}
	return c.evalCondition(Kucha(x))
}

func (c *Context) evalList(m *Expr) *Expr {
	if m == nil {
		return nil
	}
	return Upa(c.eval(Lawa(m)), c.evalList(Kucha(m)))
}

func Lawa(e *Expr) *Expr {
	if e == nil || e.sada != nil {
		return nil
	}
	return e.lawa
}

func Kucha(e *Expr) *Expr {
	if e == nil || e.sada != nil {
		return nil
	}
	return e.kucha
}

func Upa(lawa, kucha *Expr) *Expr {
	return &Expr{
		lawa:  lawa,
		kucha: kucha,
	}
}

func (e *Expr) isTrue() bool {
	return e != nil && e.sada == tokDa
}

func (e *Expr) isNya() bool {
	return e == nil || e.sada == tokNya
}

func (e *Expr) length() int {
	if e == nil {
		return 0
	}
	return 1 + Kucha(e).length()
}
