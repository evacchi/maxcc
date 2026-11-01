//! Pass: L0 -> L1. YOURS. Makes addressing explicit. See docs/ch02.md.

use crate::ast::{Expr, Special};
use crate::l1::{Addr, Val};
use crate::{ast, l1};

/// Bytes per element. Only `*u32` exists so far.
const ELEM_SIZE: u32 = 4;

pub fn address(p: &ast::Program) -> Result<l1::Program, String> {
    let k = &p.kernel;
    let body = k
        .body
        .iter()
        .map(|s| stmt(k, s))
        .collect::<Result<_, _>>()?;
    Ok(l1::Program {
        kernel: l1::Kernel {
            name: k.name.clone(),
            nparams: k.params.len(),
            body,
        },
    })
}

/// `ptr` -> its parameter index.
fn param(k: &ast::Kernel, ptr: &str) -> Result<usize, String> {
    k.params
        .iter()
        .position(|p| p.name == ptr)
        .ok_or_else(|| format!("unknown pointer '{ptr}'"))
}

fn stmt(k: &ast::Kernel, s: &ast::Stmt) -> Result<l1::Stmt, String> {
    let ast::Stmt::Store { ptr, index, value } = s;
    let param = param(k, ptr)?;

    let addr = match index {
        Expr::Int(i) => Addr::Base {
            param,
            offset: i
                .checked_mul(ELEM_SIZE)
                .ok_or_else(|| format!("index {i} overflows the byte offset"))?,
        },
        Expr::Special(Special::ThreadIdxX) => Addr::Scaled {
            param,
            index: Val::Tid,
            scale: ELEM_SIZE,
        },
        _ => todo!("ch3: address consumes flat IR"),
    };

    Ok(l1::Stmt::Store {
        addr,
        value: val(value),
    })
}

fn val(value: &Expr) -> Val {
    match value {
        Expr::Int(x) => Val::Imm(*x),
        Expr::Special(Special::ThreadIdxX) => Val::Tid,
        _ => todo!("ch3: address consumes flat IR"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn compile(src: &str) -> Result<l1::Program, String> {
        address(&parse(lex(src).unwrap()).unwrap())
    }

    #[test]
    fn constant_index_folds_to_base_offset() {
        let p = compile("kernel k(out: *u32) { out[3] = 7; }").unwrap();
        assert_eq!(
            p.kernel.body,
            vec![l1::Stmt::Store {
                addr: Addr::Base {
                    param: 0,
                    offset: 12
                },
                value: Val::Imm(7),
            }]
        );
    }

    #[test]
    fn deref_means_index_zero() {
        let p = compile("kernel k(out: *u32) { *out = 42; }").unwrap();
        assert_eq!(
            p.kernel.body,
            vec![l1::Stmt::Store {
                addr: Addr::Base {
                    param: 0,
                    offset: 0
                },
                value: Val::Imm(42),
            }]
        );
    }

    #[test]
    fn thread_index_becomes_scaled() {
        let p = compile("kernel k(out: *u32) { out[thread_idx.x] = thread_idx.x; }").unwrap();
        assert_eq!(
            p.kernel.body,
            vec![l1::Stmt::Store {
                addr: Addr::Scaled {
                    param: 0,
                    index: Val::Tid,
                    scale: 4
                },
                value: Val::Tid,
            }]
        );
    }

    #[test]
    fn multiple_params_resolve_to_indexes() {
        let p = compile("kernel k(a: *u32, b: *u32) { b[1] = 0; }").unwrap();
        assert_eq!(p.kernel.nparams, 2);
        assert_eq!(
            p.kernel.body,
            vec![l1::Stmt::Store {
                addr: Addr::Base {
                    param: 1,
                    offset: 4
                },
                value: Val::Imm(0),
            }]
        );
    }

    #[test]
    fn unknown_pointer_is_an_error() {
        assert_eq!(
            compile("kernel k(out: *u32) { bogus[0] = 1; }"),
            Err("unknown pointer 'bogus'".to_string())
        );
    }

    #[test]
    fn largest_index_fits() {
        let lim_index = u32::MAX / ELEM_SIZE;
        let p = compile(&format!("kernel k(out: *u32) {{ out[{lim_index}] = 1; }}"));
        assert_eq!(
            p.unwrap().kernel.body,
            vec![l1::Stmt::Store {
                addr: Addr::Base {
                    param: 0,
                    offset: 4294967292
                },
                value: Val::Imm(1),
            }]
        );
    }

    #[test]
    fn offset_overflow_is_an_error() {
        let overflow_index = u32::MAX / ELEM_SIZE + 1;
        let p = compile(&format!(
            "kernel k(out: *u32) {{ out[{overflow_index}] = 1; }}"
        ));
        assert_eq!(
            p.unwrap_err(),
            format!("index {overflow_index} overflows the byte offset")
        );
    }
}
