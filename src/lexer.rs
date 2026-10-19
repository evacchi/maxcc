//! Source text -> tokens.

use crate::diag::{Error, Pos};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Kernel,
    Ident(String),
    Int(u32),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Semi,
    Star,
    Assign,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub pos: Pos,
}

pub fn lex(src: &str) -> Result<Vec<Token>, Error> {
    let mut lx = Lexer { chars: src.chars().collect(), at: 0, pos: Pos { line: 1, col: 1 } };
    let mut out = Vec::new();

    loop {
        lx.skip_trivia();
        let pos = lx.pos;
        let Some(c) = lx.peek() else {
            out.push(Token { tok: Tok::Eof, pos });
            return Ok(out);
        };

        let tok = if c.is_ascii_alphabetic() || c == '_' {
            lx.word()
        } else if c.is_ascii_digit() {
            lx.number()?
        } else {
            lx.punct()?
        };
        out.push(Token { tok, pos });
    }
}

struct Lexer {
    chars: Vec<char>,
    at: usize,
    pos: Pos,
}

impl Lexer {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.at + 1).copied()
    }

    fn bump(&mut self) -> char {
        let c = self.chars[self.at];
        self.at += 1;
        if c == '\n' {
            self.pos.line += 1;
            self.pos.col = 1;
        } else {
            self.pos.col += 1;
        }
        c
    }

    fn take_while(&mut self, f: impl Fn(char) -> bool) -> String {
        let mut s = String::new();
        while self.peek().is_some_and(&f) {
            s.push(self.bump());
        }
        s
    }

    /// Whitespace and `//` comments.
    fn skip_trivia(&mut self) {
        loop {
            self.take_while(char::is_whitespace);
            if self.peek() != Some('/') || self.peek2() != Some('/') {
                return;
            }
            self.take_while(|c| c != '\n');
        }
    }

    fn word(&mut self) -> Tok {
        let s = self.take_while(|c| c.is_ascii_alphanumeric() || c == '_');
        if s == "kernel" {
            return Tok::Kernel;
        }
        Tok::Ident(s)
    }

    fn number(&mut self) -> Result<Tok, Error> {
        let pos = self.pos;
        let s = self.take_while(|c| c.is_ascii_alphanumeric() || c == '_');

        // `123abc` is one bad token, not `123` then `abc`.
        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::new(pos, format!("invalid number '{s}'")));
        }
        s.parse().map(Tok::Int).map_err(|_| Error::new(pos, format!("'{s}' does not fit in u32")))
    }

    fn punct(&mut self) -> Result<Tok, Error> {
        let pos = self.pos;
        let tok = match self.bump() {
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            '{' => Tok::LBrace,
            '}' => Tok::RBrace,
            ':' => Tok::Colon,
            ',' => Tok::Comma,
            ';' => Tok::Semi,
            '*' => Tok::Star,
            '=' => Tok::Assign,
            c => return Err(Error::new(pos, format!("unexpected character '{c}'"))),
        };
        Ok(tok)
    }
}
