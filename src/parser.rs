//! Tokens -> L0.
//!
//! program := kernel EOF
//! kernel  := 'kernel' IDENT '(' (param (',' param)*)? ')' '{' stmt* '}'
//! param   := IDENT ':' '*' 'u32'
//! stmt    := '*' IDENT '=' INT ';'

use crate::ast::{Kernel, Param, Program, Stmt};
use crate::diag::Error;
use crate::lexer::{Tok, Token};

pub fn parse(toks: Vec<Token>) -> Result<Program, Error> {
    let mut p = Parser { toks, at: 0 };
    let kernel = p.kernel()?;
    p.expect(Tok::Eof)?;
    Ok(Program { kernel })
}

struct Parser {
    toks: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.at].tok
    }

    fn bump(&mut self) -> Tok {
        let t = self.toks[self.at].tok.clone();
        if t != Tok::Eof {
            self.at += 1;
        }
        t
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() != t {
            return false;
        }
        self.bump();
        true
    }

    fn expect(&mut self, t: Tok) -> Result<(), Error> {
        if self.eat(&t) {
            return Ok(());
        }
        Err(self.err(format!("expected {t:?}, found {:?}", self.peek())))
    }

    fn ident(&mut self) -> Result<String, Error> {
        let pos = self.toks[self.at].pos;
        match self.bump() {
            Tok::Ident(s) => Ok(s),
            t => Err(Error::new(pos, format!("expected identifier, found {t:?}"))),
        }
    }

    fn err(&self, msg: String) -> Error {
        Error::new(self.toks[self.at].pos, msg)
    }

    fn kernel(&mut self) -> Result<Kernel, Error> {
        self.expect(Tok::Kernel)?;
        let name = self.ident()?;

        self.expect(Tok::LParen)?;
        let mut params = Vec::new();
        while !self.eat(&Tok::RParen) {
            if !params.is_empty() {
                self.expect(Tok::Comma)?;
            }
            params.push(self.param()?);
        }

        self.expect(Tok::LBrace)?;
        let mut body = Vec::new();
        while !self.eat(&Tok::RBrace) {
            body.push(self.stmt()?);
        }

        Ok(Kernel { name, params, body })
    }

    fn param(&mut self) -> Result<Param, Error> {
        let name = self.ident()?;
        self.expect(Tok::Colon)?;
        self.expect(Tok::Star)?;
        self.expect(Tok::Ident("u32".into()))?;
        Ok(Param { name })
    }

    fn stmt(&mut self) -> Result<Stmt, Error> {
        self.expect(Tok::Star)?;
        let ptr = self.ident()?;
        self.expect(Tok::Assign)?;

        let Tok::Int(value) = self.bump() else {
            return Err(self.err("expected integer literal".into()));
        };
        self.expect(Tok::Semi)?;

        Ok(Stmt::Store { ptr, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_src(src: &str) -> Result<Program, Error> {
        parse(lex(src)?)
    }

    #[test]
    fn parses_answer() {
        let p = parse_src(include_str!("../examples/answer.k")).unwrap();
        let want = Kernel {
            name: "answer".into(),
            params: vec![Param { name: "out".into() }],
            body: vec![Stmt::Store { ptr: "out".into(), value: 42 }],
        };
        assert_eq!(p.kernel, want);
    }

    #[test]
    fn rejects_trailing_tokens() {
        assert!(parse_src("kernel k() {} kernel j() {}").is_err());
    }

    #[test]
    fn rejects_missing_semicolon() {
        assert!(parse_src("kernel k(p: *u32) { *p = 1 }").is_err());
    }
}
