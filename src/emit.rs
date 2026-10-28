//! L-ptx -> PTX text.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::ptx::{Addr, Entry, Inst, Module, Operand, Reg, RegClass, Space, Sreg, Type};

const PTX_VERSION: &str = "7.0";
const SM_TARGET: &str = "sm_70";
const ADDRESS_SIZE: u32 = 64;

pub fn emit(m: &Module) -> String {
    let mut out = String::new();
    writeln!(out, ".version {PTX_VERSION}").unwrap();
    writeln!(out, ".target {SM_TARGET}").unwrap();
    writeln!(out, ".address_size {ADDRESS_SIZE}").unwrap();
    writeln!(out).unwrap();
    entry(&mut out, &m.entry);
    out
}

fn entry(out: &mut String, e: &Entry) {
    // Signature.
    writeln!(out, ".visible .entry {}(", e.name).unwrap();
    let params: Vec<String> = e
        .params
        .iter()
        .map(|p| format!("\t.param {} {}", ty(p.ty), p.name))
        .collect();
    writeln!(out, "{}", params.join(",\n")).unwrap();
    writeln!(out, ")").unwrap();
    writeln!(out, "{{").unwrap();

    // `.reg .b32 %r<N>;` declares %r0..%r(N-1): N = highest used + 1.
    let mut decls: BTreeMap<&str, (&str, u32)> = BTreeMap::new();
    for r in e.body.iter().flat_map(regs) {
        let (prefix, decl) = class(r.class);
        let n = &mut decls.entry(prefix).or_insert((decl, 0)).1;
        *n = (*n).max(r.n + 1);
    }
    for (prefix, (decl, n)) in &decls {
        writeln!(out, "\t.reg {decl} \t{prefix}<{n}>;").unwrap();
    }
    writeln!(out).unwrap();

    for i in &e.body {
        writeln!(out, "\t{};", inst(i)).unwrap();
    }
    writeln!(out, "}}").unwrap();
}

fn inst(i: &Inst) -> String {
    match i {
        Inst::Ld {
            space: sp,
            ty: t,
            dst,
            addr: a,
        } => format!("ld{}{} \t{}, {}", space(*sp), ty(*t), reg(*dst), addr(a)),
        Inst::St {
            space: sp,
            ty: t,
            addr: a,
            src,
        } => format!("st{}{} \t{}, {}", space(*sp), ty(*t), addr(a), operand(src)),
        Inst::Mov { ty: t, dst, src } => format!("mov{} \t{}, {}", ty(*t), reg(*dst), operand(src)),
        Inst::CvtaTo {
            space: sp,
            dst,
            src,
        } => format!("cvta.to{}.u64 \t{}, {}", space(*sp), reg(*dst), reg(*src)),
        Inst::Add { ty: t, dst, a, b } => {
            format!("add{} \t{}, {}, {}", ty(*t), reg(*dst), reg(*a), operand(b))
        }
        Inst::MulWide { ty: t, dst, a, b } => {
            format!(
                "mul.wide{} \t{}, {}, {}",
                ty(*t),
                reg(*dst),
                reg(*a),
                operand(b)
            )
        }
        Inst::Ret => "ret".into(),
    }
}

/// Every register an instruction mentions.
fn regs(i: &Inst) -> Vec<Reg> {
    i.defs().into_iter().chain(i.uses()).collect()
}

/// (register name prefix, `.reg` declaration type)
fn class(c: RegClass) -> (&'static str, &'static str) {
    match c {
        RegClass::B32 => ("%r", ".b32"),
        RegClass::B64 => ("%rd", ".b64"),
    }
}

fn reg(r: Reg) -> String {
    format!("{}{}", class(r.class).0, r.n)
}

fn ty(t: Type) -> &'static str {
    match t {
        Type::U32 => ".u32",
        Type::U64 => ".u64",
    }
}

fn space(s: Space) -> &'static str {
    match s {
        Space::Param => ".param",
        Space::Global => ".global",
    }
}

fn addr(a: &Addr) -> String {
    match a {
        Addr::Param(name) => format!("[{name}]"),
        Addr::Reg(r) => format!("[{}]", reg(*r)),
        Addr::Offset(r, off) => format!("[{}+{off}]", reg(*r)),
    }
}

fn operand(o: &Operand) -> String {
    match o {
        Operand::Reg(r) => reg(*r),
        Operand::Imm(v) => v.to_string(),
        Operand::Sreg(s) => sreg(*s).into(),
    }
}

fn sreg(s: Sreg) -> &'static str {
    match s {
        Sreg::TidX => "%tid.x",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ptx::Param;

    #[test]
    fn emits_entry() {
        let rd = |n| Reg {
            class: RegClass::B64,
            n,
        };
        let r = |n| Reg {
            class: RegClass::B32,
            n,
        };
        let m = Module {
            entry: Entry {
                name: "k".into(),
                params: vec![Param {
                    name: "k_param_0".into(),
                    ty: Type::U64,
                }],
                body: vec![
                    Inst::Ld {
                        space: Space::Param,
                        ty: Type::U64,
                        dst: rd(1),
                        addr: Addr::Param("k_param_0".into()),
                    },
                    Inst::St {
                        space: Space::Global,
                        ty: Type::U32,
                        addr: Addr::Reg(rd(1)),
                        src: Operand::Imm(7),
                    },
                    Inst::Mov {
                        ty: Type::U32,
                        dst: r(2),
                        src: Operand::Sreg(Sreg::TidX),
                    },
                    Inst::MulWide {
                        ty: Type::U32,
                        dst: rd(3),
                        a: r(2),
                        b: Operand::Imm(4),
                    },
                    Inst::Add {
                        ty: Type::U64,
                        dst: rd(4),
                        a: rd(1),
                        b: Operand::Reg(rd(3)),
                    },
                    Inst::St {
                        space: Space::Global,
                        ty: Type::U32,
                        addr: Addr::Offset(rd(4), 8),
                        src: Operand::Reg(r(2)),
                    },
                    Inst::Ret,
                ],
            },
        };

        let want = "\
.version 7.0
.target sm_70
.address_size 64

.visible .entry k(
\t.param .u64 k_param_0
)
{
\t.reg .b32 \t%r<3>;
\t.reg .b64 \t%rd<5>;

\tld.param.u64 \t%rd1, [k_param_0];
\tst.global.u32 \t[%rd1], 7;
\tmov.u32 \t%r2, %tid.x;
\tmul.wide.u32 \t%rd3, %r2, 4;
\tadd.u64 \t%rd4, %rd1, %rd3;
\tst.global.u32 \t[%rd4+8], %r2;
\tret;
}
";
        assert_eq!(emit(&m), want);
    }
}
