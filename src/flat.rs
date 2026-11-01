//! L0-flat: L0 with nested expressions flattened to three-address form.
//! Every operator gets its own statement and a fresh temporary; operands are atoms.
//!
//!   out[block_idx.x * block_dim.x + thread_idx.x] = 7;
//!
//! becomes
//!
//!   t0 = block_idx.x * block_dim.x
//!   t1 = t0 + thread_idx.x
//!   out[t1] = 7

use crate::ast::{BinOp, Param, Special};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub kernel: Kernel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kernel {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// `dst = lhs op rhs`. Each `dst` is written exactly once.
    Bin {
        dst: Temp,
        op: BinOp,
        lhs: Atom,
        rhs: Atom,
    },
    /// `ptr[index] = value`
    Store {
        ptr: String,
        index: Atom,
        value: Atom,
    },
}

/// An operand that needs no computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Atom {
    Int(u32),
    Special(Special),
    Temp(Temp),
}

/// A compiler-generated name, `t<n>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Temp(pub u32);
