//! L1: L0 with explicit addressing.
//!
//! Names are resolved to parameter indexes, element types are gone:
//! only byte offsets and scales remain.
//!
//!   out[3]            -> Base   { param: 0, offset: 12 }
//!   out[thread_idx.x] -> Scaled { param: 0, index: Tid, scale: 4 }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub kernel: Kernel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kernel {
    pub name: String,
    /// Number of pointer parameters.
    pub nparams: usize,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// Store a 32-bit value.
    Store { addr: Addr, value: Val },
}

/// A 32-bit value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Val {
    Imm(u32),
    /// `thread_idx.x`
    Tid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Addr {
    /// param + offset bytes.
    Base { param: usize, offset: u32 },
    /// param + index * scale bytes. `index` is never `Imm`: constants fold into `Base`.
    Scaled { param: usize, index: Val, scale: u32 },
}
