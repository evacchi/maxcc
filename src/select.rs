//! Pass: L1 -> L-ptx. YOURS. See docs/ch02.md.

use crate::ptx::Inst::{Add, Mov, MulWide};
use crate::ptx::{Addr, Entry, Inst, Module, Operand, Param, Reg, RegClass, Space, Type};
use crate::{l1, ptx};

pub fn select(p: &l1::Program) -> Result<Module, String> {
    let k = &p.kernel;

    // One `.param .u64` per kernel parameter: pointers are 64-bit (.address_size 64).
    let params = (0..k.nparams)
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
    kernel: &'a l1::Kernel,
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

    fn addr(&mut self, a: &l1::Addr) -> Addr {
        match *a {
            l1::Addr::Base { param, offset } => {
                let base = self.base(param); // your ld.param + cvta, returns tgt
                if offset == 0 {
                    Addr::Reg(base)
                } else {
                    Addr::Offset(base, offset)
                }
            }
            l1::Addr::Scaled {
                param,
                index,
                scale,
            } => {
                let base = self.base(param);
                let offset = self.fresh(RegClass::B64);
                let address = self.fresh(RegClass::B64);
                let idx = self.reg(index);
                self.body.push(MulWide {
                    ty: Type::U32,
                    dst: offset,
                    a: idx,
                    b: Operand::Imm(scale.into()),
                });
                self.body.push(Add {
                    ty: Type::U64,
                    dst: address,
                    a: offset,
                    b: Operand::Reg(base),
                });
                Addr::Reg(address)
            }
        }
    }

    /// `v` in a 32-bit register. Every value gets a `mov`.
    fn reg(&mut self, v: l1::Val) -> Reg {
        let src = match v {
            l1::Val::Imm(i) => Operand::Imm(i.into()),
            l1::Val::Tid => Operand::Sreg(ptx::Sreg::TidX),
        };
        let r = self.fresh(RegClass::B32);
        self.body.push(Mov {
            ty: Type::U32,
            dst: r,
            src,
        });
        r
    }

    /// `v` as an operand. Immediates stay inline; the rest goes through `reg`.
    fn val(&mut self, v: l1::Val) -> Operand {
        match v {
            l1::Val::Imm(imm) => Operand::Imm(imm.into()),
            l1::Val::Tid => Operand::Reg(self.reg(v)),
        }
    }

    fn stmt(&mut self, stmt: &l1::Stmt) -> Result<(), String> {
        match stmt {
            l1::Stmt::Store { addr, value } => {
                let addr = self.addr(addr);
                let src = self.val(*value);

                self.body.push(Inst::St {
                    space: Space::Global,
                    ty: Type::U32,
                    addr,
                    src,
                })
            }
        }
        Ok(())
    }

    fn base(&mut self, i: usize) -> Reg {
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
        tgt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{address, lexer, parser};
    use std::collections::{HashMap, HashSet};

    fn select_src(src: &str) -> Result<Module, String> {
        let toks = lexer::lex(src).map_err(|e| e.to_string())?;
        let ast = parser::parse(toks).map_err(|e| e.to_string())?;
        select(&address::address(&ast)?)
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
        validate_regs(&body);

        assert!(
            !body
                .iter()
                .any(|i| matches!(i, Inst::CvtaTo { dst, .. } if dst.class != RegClass::B64)),
            "CvtaTo(dst) must be RegClass::B64"
        );

        let cvtas: HashMap<&ptx::Reg, usize> = body
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

    #[test]
    fn iota_invariants() {
        let body = select_src("kernel k(out: *u32) { out[thread_idx.x] = thread_idx.x; }")
            .unwrap()
            .entry
            .body;

        assert!(
            !body.iter().any(|ins| matches!(
                ins,
                Inst::St {
                    src: Operand::Sreg(_),
                    ..
                }
            )),
            "St src operand must not be Sreg"
        );

        assert!(
            !body.iter().any(|ins| matches!(
                ins,
                Inst::St {
                    addr: Addr::Reg(r) | Addr::Offset(r, _) ,
                    ..
                } if r.class != RegClass::B64
            )),
            "St Addr reg must be B64"
        );

        assert!(
            !body.iter().any(|ins| matches!(
                ins,
                Inst::MulWide { dst, .. } | Inst::Add { dst, .. } if dst.class != RegClass::B64
            )),
            "MulWide dst must be B64"
        );
        validate_regs(&body);
    }

    /// Defined before use: every register read was written by an earlier
    /// instruction. Straight-line code only; branches (ch 4) need the CFG.
    fn validate_regs(body: &[Inst]) {
        let mut defined = HashSet::new();
        for (i, inst) in body.iter().enumerate() {
            for r in inst.uses() {
                assert!(
                    defined.contains(&r),
                    "#{i} {inst:?} reads {r:?} before any write"
                );
            }
            defined.extend(inst.defs());
        }
    }
}
