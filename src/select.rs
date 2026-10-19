//! Pass: L0 (ast) -> L-ptx. YOURS. See docs/ch01.md.

use crate::{ast, ptx};

pub fn select(p: &ast::Program) -> Result<ptx::Module, String> {
    let _ = p;
    todo!("chapter 1")
}
