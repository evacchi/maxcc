//! Pass: L0 -> L1. YOURS. Makes addressing explicit. See docs/ch02.md.

use crate::{ast, l1};

/// Bytes per element. Only `*u32` exists so far.
const ELEM_SIZE: u32 = 4;

pub fn address(p: &ast::Program) -> Result<l1::Program, String> {
    let k = &p.kernel;
    let body = k.body.iter().map(|s| stmt(k, s)).collect::<Result<_, _>>()?;
    Ok(l1::Program { kernel: l1::Kernel { name: k.name.clone(), nparams: k.params.len(), body } })
}

/// `ptr` -> its parameter index.
fn param(k: &ast::Kernel, ptr: &str) -> Result<usize, String> {
    k.params.iter().position(|p| p.name == ptr).ok_or_else(|| format!("unknown pointer '{ptr}'"))
}

fn stmt(k: &ast::Kernel, s: &ast::Stmt) -> Result<l1::Stmt, String> {
    let ast::Stmt::Store { ptr, index, value } = s;
    let param = param(k, ptr)?;

    // TODO(human): build the L1 store for `ptr[index] = value`.
    let _ = (param, index, value, ELEM_SIZE);
    todo!()
}
