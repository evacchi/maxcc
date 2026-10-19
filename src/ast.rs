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
    /// `*ptr = value;`
    Store { ptr: String, value: u32 },
}
