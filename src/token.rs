use std::fmt;

pub const EOFRUNE: char = '\0';

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Error,
    EOF,
    Tiga,
    Const,
    Number,
    Lpar,
    Rpar,
    Dot,
    Char,
    Quote,
    Newline,
    String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub typ: TokenType,
    pub text: String,
    pub num: i64,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.typ {
            TokenType::Number => write!(f, "{}", self.num),
            _ => write!(f, "{}", self.text),
        }
    }
}

impl Token {
    pub fn build_string(&self, s: &mut String) {
        match self.typ {
            TokenType::Number => s.push_str(&self.num.to_string()),
            _ => s.push_str(&self.text),
        }
    }
}

pub fn make_token(typ: TokenType, text: &str) -> Token {
    if typ == TokenType::Number {
        let num = text.parse::<i64>().expect("invalid number syntax");
        return Token {
            typ: TokenType::Number,
            text: "".to_string(),
            num,
        };
    }
    Token {
        typ,
        text: text.to_string(),
        num: 0,
    }
}

pub fn make_tiga(text: &str) -> Token {
    make_token(TokenType::Tiga, text)
}

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    backed: Option<char>,
    pub last: char,
    buf: String,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars(),
            backed: None,
            last: '\0',
            buf: String::new(),
        }
    }

    pub fn read(&mut self) -> char {
        if let Some(c) = self.backed.take() {
            self.last = c;
            return c;
        }
        self.next_rune()
    }

    fn next_rune(&mut self) -> char {
        let c = self.chars.next().unwrap_or(EOFRUNE);
        self.last = c;
        c
    }

    pub fn peek(&mut self) -> char {
        if let Some(c) = self.backed {
            return c;
        }
        let c = self.read();
        self.backed = Some(c);
        c
    }

    pub fn back(&mut self, r: char) {
        self.backed = Some(r);
    }

    pub fn skip_space(&mut self) -> char {
        let mut comment = false;
        loop {
            let r = self.read();
            match r {
                '\n' | '\0' => return r,
                ';' => {
                    comment = true;
                    continue;
                }
                _ => {}
            }
            if !comment && !is_space(r) {
                self.back(r);
                return r;
            }
        }
    }

    pub fn skip_to_newline(&mut self) {
        while self.last != '\n' && self.last != EOFRUNE {
            self.next_rune();
        }
        self.backed = None;
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            let r = self.read();
            match r {
                ' ' | '\t' | '\r' | '\n' => {}
                ';' => self.skip_to_newline(),
                '\0' => return make_token(TokenType::EOF, "EOF"),
                '(' => return make_token(TokenType::Lpar, "("),
                ')' => return make_token(TokenType::Rpar, ")"),
                '.' => return make_token(TokenType::Dot, "."),
                '-' | '+' => {
                    if !self.peek().is_ascii_digit() {
                        return make_token(TokenType::Char, &r.to_string());
                    }
                    return self.number(r);
                }
                '0'..='9' => return self.number(r),
                '\'' => return make_token(TokenType::Quote, "'"),
                '_' | 'a'..='z' | 'A'..='Z' => return self.alphanum(TokenType::Tiga, r),
                '"' => return self.string_token(r),
                _ => return make_token(TokenType::Char, &r.to_string()),
            }
        }
    }

    fn number(&mut self, r: char) -> Token {
        self.accum(r, |c| c.is_ascii_digit());
        self.end_token();
        make_token(TokenType::Number, &self.buf)
    }

    fn alphanum(&mut self, typ: TokenType, r: char) -> Token {
        self.accum(r, |c| c == '_' || c.is_ascii_alphanumeric());
        self.end_token();
        make_token(typ, &self.buf)
    }

    fn accum(&mut self, r: char, valid: impl Fn(char) -> bool) {
        self.buf.clear();
        let mut r = r;
        loop {
            self.buf.push(r);
            r = self.read();
            if r == EOFRUNE {
                return;
            }
            if !valid(r) {
                self.back(r);
                return;
            }
        }
    }

    fn string_token(&mut self, r: char) -> Token {
        self.buf.clear();
        self.buf.push(r);

        let mut r = r;
        while r != EOFRUNE {
            r = self.read();
            match r {
                '\\' => {
                    r = self.read();
                    if r == EOFRUNE {
                        break;
                    }
                }
                '"' => {
                    self.buf.push(r);
                    return make_token(TokenType::String, &self.buf);
                }
                _ => {}
            }
            self.buf.push(r);
        }
        lex_error("unexpected end of string for {}", &self.buf);
    }

    fn end_token(&mut self) {
        let r = self.peek();
        if is_alpha_number(r) || (!is_space(r) && r != '(' && r != ')' && r != '.' && r != EOFRUNE) {
            lex_error("invalid token after {}", &self.buf);
        }
    }
}

fn is_space(r: char) -> bool {
    matches!(r, ' ' | '\t' | '\n' | '\r')
}

fn is_alpha_number(r: char) -> bool {
    r == '_' || r.is_ascii_alphanumeric()
}

fn lex_error(msg: &str, arg: &str) -> ! {
    panic!("{} {}", msg, arg);
}
