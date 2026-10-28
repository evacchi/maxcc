//! L-ptx: PTX as data. One variant per instruction form we emit.
//! Printed by `emit`. See the PTX ISA: https://docs.nvidia.com/cuda/parallel-thread-execution/

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub entry: Entry,
}

/// `.visible .entry name(.param ...) { body }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Inst>,
}

/// `.param .<ty> <name>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

/// Instruction type suffix, e.g. the `.u32` in `st.global.u32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    U32,
    U64,
}

/// State space, e.g. the `.global` in `st.global.u32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Param,
    Global,
}

/// Register file. Printed as `%r<n>` (32-bit) or `%rd<n>` (64-bit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegClass {
    B32,
    B64,
}

/// A virtual register. `emit` declares one `.reg` range per class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg {
    pub class: RegClass,
    pub n: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Reg(Reg),
    Imm(u64),
    Sreg(Sreg),
}

/// Special (read-only, per-thread) register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sreg {
    /// `%tid.x`: thread index within the block.
    TidX,
}

/// The `[...]` in loads and stores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Addr {
    /// `[kernel_param_0]`
    Param(String),
    /// `[%rd2]`
    Reg(Reg),
    /// `[%rd2+12]`
    Offset(Reg, u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inst {
    /// `ld.<space>.<ty> dst, [addr];`
    Ld {
        space: Space,
        ty: Type,
        dst: Reg,
        addr: Addr,
    },
    /// `st.<space>.<ty> [addr], src;`
    St {
        space: Space,
        ty: Type,
        addr: Addr,
        src: Operand,
    },
    /// `mov.<ty> dst, src;`
    Mov { ty: Type, dst: Reg, src: Operand },
    /// `cvta.to.<space>.u64 dst, src;` generic address -> `space` address.
    CvtaTo { space: Space, dst: Reg, src: Reg },
    /// `add.<ty> dst, a, b;`
    Add {
        ty: Type,
        dst: Reg,
        a: Reg,
        b: Operand,
    },
    /// `mul.wide.<ty> dst, a, b;` dst is twice as wide as `ty`.
    MulWide {
        ty: Type,
        dst: Reg,
        a: Reg,
        b: Operand,
    },
    /// `ret;`
    Ret,
}

impl Inst {
    /// The register this instruction writes, if any.
    pub fn defs(&self) -> Option<Reg> {
        match self {
            Inst::Ld { dst, .. }
            | Inst::Mov { dst, .. }
            | Inst::CvtaTo { dst, .. }
            | Inst::Add { dst, .. }
            | Inst::MulWide { dst, .. } => Some(*dst),
            Inst::St { .. } | Inst::Ret => None,
        }
    }

    /// Every register this instruction reads: value operands, address
    /// registers, conversion inputs. Immediates and special registers are not uses.
    pub fn uses(&self) -> Vec<Reg> {
        match self {
            Inst::Ld { addr, .. } => addr.reg().into_iter().collect(),
            Inst::St { addr, src, .. } => addr.reg().into_iter().chain(src.reg()).collect(),
            Inst::Mov { src, .. } => src.reg().into_iter().collect(),
            Inst::CvtaTo { src, .. } => vec![*src],
            Inst::Add { a, b, .. } | Inst::MulWide { a, b, .. } => {
                [*a].into_iter().chain(b.reg()).collect()
            }
            Inst::Ret => vec![],
        }
    }
}

impl Addr {
    /// The register inside `[...]`, if any.
    fn reg(&self) -> Option<Reg> {
        match self {
            Addr::Reg(r) | Addr::Offset(r, _) => Some(*r),
            Addr::Param(_) => None,
        }
    }
}

impl Operand {
    fn reg(&self) -> Option<Reg> {
        match self {
            Operand::Reg(r) => Some(*r),
            Operand::Imm(_) | Operand::Sreg(_) => None,
        }
    }
}
