//! Pass: L0 -> L0-flat. YOURS. See docs/ch03.md.

use crate::ast::{self, Expr};
use crate::flat::{self, Atom, Stmt, Temp};

pub fn flatten(p: &ast::Program) -> flat::Program {
    let k = &p.kernel;
    let mut f = Flattener {
        body: Vec::new(),
        next: 0,
    };
    for s in &k.body {
        f.stmt(s);
    }

    flat::Program {
        kernel: flat::Kernel {
            name: k.name.clone(),
            params: k.params.clone(),
            body: f.body,
        },
    }
}

struct Flattener {
    body: Vec<Stmt>,
    next: u32,
}

impl Flattener {
    /// A temporary never used before.
    fn fresh(&mut self) -> Temp {
        let t = Temp(self.next);
        self.next += 1;
        t
    }

    fn stmt(&mut self, s: &ast::Stmt) {
        let ast::Stmt::Store { ptr, index, value } = s;

        // Source order: index before value.
        let index = self.atom(index);
        let value = self.atom(value);
        self.body.push(Stmt::Store {
            ptr: ptr.clone(),
            index,
            value,
        });
    }

    /// `e` as an atom. Emits one `Stmt::Bin` per operator into `self.body`,
    /// operands before the operator that uses them.
    fn atom(&mut self, e: &Expr) -> Atom {
        // TODO(human)
        todo!()
    }
}
