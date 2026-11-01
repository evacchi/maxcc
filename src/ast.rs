//! L0: source AST.

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

/// Always `*u32` for now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// `ptr[index] = value;` (`*ptr = v` parses as `ptr[0] = v`)
    Store {
        ptr: String,
        index: Expr,
        value: Expr,
    },
}

/// A u32 value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Int(u32),
    Special(Special),
    /// `lhs op rhs`, wrapping on overflow.
    Bin {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

/// Read-only per-thread values the hardware provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Special {
    /// `thread_idx.x`: thread index within the block.
    ThreadIdxX,
    /// `block_idx.x`: block index within the grid.
    BlockIdxX,
    /// `block_dim.x`: threads per block.
    BlockDimX,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Mul,
}
