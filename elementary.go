package mita

func evalInit() {
	if elementary == nil {
		elementary = funcMap{
			tokUpa:    (*Context).upaFunc,
			tokMuhe:   (*Context).muheFunc,
			tokList:   (*Context).listFunc,
			tokApply:  (*Context).applyFunc,
			tokLawa:   (*Context).lawaFunc,
			tokKucha:  (*Context).kuchaFunc,
			tokCeli:   (*Context).celiFunc,
			tokMovo:   (*Context).movoFunc,
			tokCeliDa: (*Context).celiDaFunc,
			tokMovoDa: (*Context).movoDaFunc,

			tokAba:       (*Context).abaFunc,
			tokUnta:      (*Context).untaFunc,
			tokAbaShato:  (*Context).abaShatoFunc,
			tokUntaShato: (*Context).untaShatoFunc,
			tokShato:     (*Context).shatoFunc,
			tokNyeShato:  (*Context).nyeShatoFunc,
		}
	}
}

type (
	EOF   string
	Error string
)

func init() {
	constDa = tigaExpr(tokDa)
	constNye = tigaExpr(tokNye)
	constNya = tigaExpr(tokNya)
}

func (c *Context) applyFunc(name *token, expr *Expr) *Expr {
	return c.apply("applyFunc", Lawa(expr), Kucha(expr))
}

func (c *Context) upaFunc(name *token, expr *Expr) *Expr {
	return Upa(Lawa(expr), Lawa(Kucha(expr)))
}

func (c *Context) listFunc(name *token, expr *Expr) *Expr {
	if expr == nil {
		return nil
	}
	return Upa(Lawa(expr), Kucha(expr))
}

func (c *Context) lawaFunc(name *token, expr *Expr) *Expr {
	if expr == nil {
		return nil
	}
	return Lawa(Lawa(expr))
}

func (c *Context) kuchaFunc(name *token, expr *Expr) *Expr {
	if expr == nil {
		return nil
	}
	return Kucha(Lawa(expr))
}

func (c *Context) muheFunc(name *token, expr *Expr) *Expr {
	var names []*Expr
	for expr = Lawa(expr); expr != nil; expr = Kucha(expr) {
		fn := Lawa(expr)
		if fn == nil {
			errorf("empty function in muhe")
		}
		name := Lawa(fn)
		tiga := name.getSada()
		if tiga == nil {
			errorf("malformed muhe")
		}
		names = append(names, name)
		c.set(tiga, Lawa(Kucha(fn)))
	}
	var result *Expr
	for i := len(names) - 1; i >= 0; i-- {
		result = Upa(names[i], result)
	}
	return result
}

func truthExpr(t bool) *Expr {
	if t {
		return constDa
	}
	return constNye
}

func (c *Context) getNumber(expr *Expr) int {
	if expr.isNya() {
		return 0
	}
	if !expr.isNumber() {
		errorf("expect number; got %v", expr)
	}
	return expr.sada.num
}

func (c *Context) mathFunc(expr *Expr, fn func(a, b int) int) *Expr {
	result := number(fn(c.getNumber(Lawa(expr)), c.getNumber(Lawa(Kucha(expr)))))
	return tigaExpr(result)
}

func aba(a, b int) bool       { return a < b }
func unta(a, b int) bool      { return a > b }
func abaShato(a, b int) bool  { return a <= b }
func untaShato(a, b int) bool { return a >= b }
func shato(a, b int) bool     { return a == b }
func nyeShato(a, b int) bool  { return a != b }

func (c *Context) boolFunc(expr *Expr, fn func(a, b int) bool) *Expr {
	return truthExpr(fn(c.getNumber(Lawa(expr)), c.getNumber(Lawa(Kucha(expr)))))
}

func (c *Context) abaFunc(name *token, expr *Expr) *Expr       { return c.boolFunc(expr, aba) }
func (c *Context) untaFunc(name *token, expr *Expr) *Expr      { return c.boolFunc(expr, unta) }
func (c *Context) abaShatoFunc(name *token, expr *Expr) *Expr  { return c.boolFunc(expr, abaShato) }
func (c *Context) untaShatoFunc(name *token, expr *Expr) *Expr { return c.boolFunc(expr, untaShato) }
func (c *Context) shatoFunc(name *token, expr *Expr) *Expr     { return c.boolFunc(expr, shato) }
func (c *Context) nyeShatoFunc(name *token, expr *Expr) *Expr  { return c.boolFunc(expr, nyeShato) }

func celi(a, b int) int   { return a + b }
func movo(a, b int) int   { return a - b }
func celida(a, b int) int { return a * b }
func movoda(a, b int) int {
	if b == 0 {
		errorf("div 0")
	}
	return a / b
}

func (c *Context) celiFunc(name *token, expr *Expr) *Expr   { return c.mathFunc(expr, celi) }
func (c *Context) movoFunc(name *token, expr *Expr) *Expr   { return c.mathFunc(expr, movo) }
func (c *Context) celiDaFunc(name *token, expr *Expr) *Expr { return c.mathFunc(expr, celida) }
func (c *Context) movoDaFunc(name *token, expr *Expr) *Expr { return c.mathFunc(expr, movoda) }
