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
    /// `thread_idx.x`
    ThreadIdxX,
}
