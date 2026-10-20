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
}

/// The `[...]` in loads and stores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Addr {
    /// `[kernel_param_0]`
    Param(String),
    /// `[%rd2]`
    Reg(Reg),
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
    /// `ret;`
    Ret,
}
