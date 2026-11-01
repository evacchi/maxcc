//! Tokens -> L0.
//!
//! program := kernel EOF
//! kernel  := 'kernel' IDENT '(' (param (',' param)*)? ')' '{' stmt* '}'
//! param   := IDENT ':' '*' 'u32'
//! stmt    := '*' IDENT '=' expr ';'          (sugar for IDENT '[' 0 ']')
//!          | IDENT '[' expr ']' '=' expr ';'
//! expr    := term ('+' term)*
//! term    := atom ('*' atom)*
//! atom    := INT | special | '(' expr ')'
//! special := ('thread_idx' | 'block_idx' | 'block_dim') '.' 'x'

use crate::ast::{BinOp, Expr, Kernel, Param, Program, Special, Stmt};
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
        // `*p` is `p[0]`.
        let (ptr, index) = if self.eat(&Tok::Star) {
            (self.ident()?, Expr::Int(0))
        } else {
            let ptr = self.ident()?;
            self.expect(Tok::LBracket)?;
            let index = self.expr()?;
            self.expect(Tok::RBracket)?;
            (ptr, index)
        };

        self.expect(Tok::Assign)?;
        let value = self.expr()?;
        self.expect(Tok::Semi)?;

        Ok(Stmt::Store { ptr, index, value })
    }

    /// One loop per precedence level: `+` binds looser than `*`,
    /// both associate left. `1 + 2 * 3` is `1 + (2 * 3)`.
    fn expr(&mut self) -> Result<Expr, Error> {
        let mut lhs = self.term()?;
        while self.eat(&Tok::Plus) {
            lhs = bin(BinOp::Add, lhs, self.term()?);
        }
        Ok(lhs)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut lhs = self.atom()?;
        while self.eat(&Tok::Star) {
            lhs = bin(BinOp::Mul, lhs, self.atom()?);
        }
        Ok(lhs)
    }

    fn atom(&mut self) -> Result<Expr, Error> {
        let pos = self.toks[self.at].pos;
        match self.bump() {
            Tok::Int(v) => Ok(Expr::Int(v)),
            Tok::LParen => {
                let e = self.expr()?;
                self.expect(Tok::RParen)?;
                Ok(e)
            }
            Tok::Ident(name) => self.special(&name).map(Expr::Special),
            t => Err(Error::new(pos, format!("expected expression, found {t:?}"))),
        }
    }

    /// `name` has been consumed; parses the `.x` that follows.
    fn special(&mut self, name: &str) -> Result<Special, Error> {
        let pos = self.toks[self.at - 1].pos;
        let s = match name {
            "thread_idx" => Special::ThreadIdxX,
            "block_idx" => Special::BlockIdxX,
            "block_dim" => Special::BlockDimX,
            _ => return Err(Error::new(pos, format!("unknown name '{name}'"))),
        };
        self.expect(Tok::Dot)?;
        self.expect(Tok::Ident("x".into()))?;
        Ok(s)
    }
}

fn bin(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr::Bin {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
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
            body: vec![Stmt::Store {
                ptr: "out".into(),
                index: Expr::Int(0),
                value: Expr::Int(42),
            }],
        };
        assert_eq!(p.kernel, want);
    }

    #[test]
    fn parses_iota() {
        let p = parse_src(include_str!("../examples/iota.k")).unwrap();
        let want = Stmt::Store {
            ptr: "out".into(),
            index: Expr::Special(Special::ThreadIdxX),
            value: Expr::Special(Special::ThreadIdxX),
        };
        assert_eq!(p.kernel.body, [want]);
    }

    /// The index of the single store in `src`.
    fn index_of(src: &str) -> Expr {
        let p = parse_src(src).unwrap();
        let [Stmt::Store { index, .. }] = &p.kernel.body[..] else {
            panic!("expected one store");
        };
        index.clone()
    }

    #[test]
    fn mul_binds_tighter_than_add() {
        let want = bin(
            BinOp::Add,
            Expr::Int(1),
            bin(BinOp::Mul, Expr::Int(2), Expr::Int(3)),
        );
        assert_eq!(index_of("kernel k(p: *u32) { p[1 + 2 * 3] = 0; }"), want);
    }

    #[test]
    fn parens_and_left_assoc() {
        let want = bin(
            BinOp::Mul,
            bin(BinOp::Add, Expr::Int(1), Expr::Int(2)),
            Expr::Int(3),
        );
        assert_eq!(index_of("kernel k(p: *u32) { p[(1 + 2) * 3] = 0; }"), want);

        let want = bin(
            BinOp::Add,
            bin(BinOp::Add, Expr::Int(1), Expr::Int(2)),
            Expr::Int(3),
        );
        assert_eq!(index_of("kernel k(p: *u32) { p[1 + 2 + 3] = 0; }"), want);
    }

    #[test]
    fn parses_global_index() {
        let s = |sp| Expr::Special(sp);
        let want = bin(
            BinOp::Add,
            bin(BinOp::Mul, s(Special::BlockIdxX), s(Special::BlockDimX)),
            s(Special::ThreadIdxX),
        );
        let src = "kernel k(p: *u32) { p[block_idx.x * block_dim.x + thread_idx.x] = 0; }";
        assert_eq!(index_of(src), want);
    }

    #[test]
    fn rejects_unknown_special() {
        assert!(parse_src("kernel k(p: *u32) { p[thread_idx.y] = 1; }").is_err());
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
