//! Pass: L0 (ast) -> L-ptx. YOURS. See docs/ch01.md.

use crate::ast;
use crate::ptx::{Addr, Entry, Inst, Module, Operand, Param, Reg, RegClass, Space, Type};

pub fn select(p: &ast::Program) -> Result<Module, String> {
    let k = &p.kernel;

    // One `.param .u64` per kernel parameter: pointers are 64-bit (.address_size 64).
    let params = (0..k.params.len())
        .map(|i| Param {
            name: param_name(&k.name, i),
            ty: Type::U64,
        })
        .collect();

    let mut s = Selector {
        kernel: k,
        body: Vec::new(),
        next: 1,
    };
    for stmt in &k.body {
        s.stmt(stmt)?;
    }
    s.body.push(Inst::Ret);

    Ok(Module {
        entry: Entry {
            name: k.name.clone(),
            params,
            body: s.body,
        },
    })
}

/// `answer`, 0 -> `answer_param_0`
fn param_name(kernel: &str, i: usize) -> String {
    format!("{kernel}_param_{i}")
}

struct Selector<'a> {
    kernel: &'a ast::Kernel,
    body: Vec<Inst>,
    next: u32,
}

impl Selector<'_> {
    /// A register never used before. Numbers are shared across classes;
    /// gaps are fine, `emit` declares up to the highest one.
    fn fresh(&mut self, class: RegClass) -> Reg {
        let r = Reg {
            class,
            n: self.next,
        };
        self.next += 1;
        r
    }

    fn stmt(&mut self, stmt: &ast::Stmt) -> Result<(), String> {
        match stmt {
            ast::Stmt::Store { ptr, value } => {
                // `ptr` must name a parameter, find index
                let Some(i) = self.kernel.params.iter().position(|p| &p.name == ptr) else {
                    return Err(format!("unknown pointer '{ptr}'"));
                };
                let param = param_name(&self.kernel.name, i);

                let addr = self.fresh(RegClass::B64);
                self.body.push(Inst::Ld {
                    space: Space::Param,
                    ty: Type::U64,
                    dst: addr,
                    addr: Addr::Param(param),
                });
                let tgt = self.fresh(RegClass::B64);
                self.body.push(Inst::CvtaTo {
                    space: Space::Global,
                    src: addr,
                    dst: tgt,
                });
                self.body.push(Inst::St {
                    space: Space::Global,
                    ty: Type::U32,
                    addr: Addr::Reg(tgt),
                    src: Operand::Imm((*value).into()),
                })
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer, parser};
    use std::collections::HashMap;

    fn select_src(src: &str) -> Result<Module, String> {
        let toks = lexer::lex(src).map_err(|e| e.to_string())?;
        let ast = parser::parse(toks).map_err(|e| e.to_string())?;
        select(&ast)
    }

    #[test]
    fn rejects_unknown_pointer() {
        assert!(select_src("kernel k(a: *u32) { *b = 1; }").is_err());
    }

    #[test]
    fn ensure_invariants() {
        // - exactly one Inst::Ret, and it's the last instruction;
        // - every St address register is the dst of an earlier CvtaTo;
        // - every register an address passes through is RegClass::B64;
        // - the stored values, in order, are [1, 2].

        let body = select_src("kernel k(a: *u32, b: *u32) { *a = 1; *b = 2; }")
            .unwrap()
            .entry
            .body;
        assert_eq!(
            body.iter().filter(|i| matches!(i, Inst::Ret)).count(),
            1,
            "exactly one ret"
        );
        assert_eq!(body.last(), Some(&Inst::Ret), "ret is last");

        assert!(
            !body
                .iter()
                .any(|i| matches!(i, Inst::CvtaTo { dst, .. } if dst.class != RegClass::B64)),
            "CvtaTo(dst) must be RegClass::B64"
        );

        let cvtas: HashMap<&Reg, usize> = body
            .iter()
            .enumerate()
            .filter_map(|(i, inst)| match inst {
                Inst::CvtaTo { dst, .. } => Some((dst, i)),
                _ => None,
            })
            .collect();

        assert!(
            !body.iter().enumerate().any(|(idx, inst)| matches!(
                inst,
                Inst::St { addr: Addr::Reg(r), .. } if !cvtas.get(r).is_some_and(|&at| at < idx)
            )),
            "St address must come from an earlier CvtaTo"
        );

        let imms = body
            .iter()
            .filter_map(|inst| match inst {
                Inst::St {
                    src: Operand::Imm(imm),
                    ..
                } => Some(*imm),
                _ => None,
            })
            .collect::<Vec<u64>>();
        assert_eq!(imms, [1, 2], "Immediates must be exactly 1 and 2.");
    }
}
