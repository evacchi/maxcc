//! The pipeline. Stops after `stage` and prints that stage's output.
//!
//!   source --lex--> tokens --parse--> L0 --address--> L1 --select--> L-ptx --emit--> .ptx
//!                                      \--flatten--> L0-flat  (ch3: not yet wired to address)

use crate::{address, emit, flatten, lexer, parser, select};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Lex,
    Parse,
    Flatten,
    Address,
    Select,
    Emit,
}

pub fn compile(src: &str, stage: Stage) -> Result<String, String> {
    let toks = lexer::lex(src).map_err(|e| e.to_string())?;
    if stage == Stage::Lex {
        return Ok(format!("{toks:#?}"));
    }

    let ast = parser::parse(toks).map_err(|e| e.to_string())?;
    if stage == Stage::Parse {
        return Ok(format!("{ast:#?}"));
    }

    if stage == Stage::Flatten {
        return Ok(format!("{:#?}", flatten::flatten(&ast)));
    }

    let l1 = address::address(&ast)?;
    if stage == Stage::Address {
        return Ok(format!("{l1:#?}"));
    }

    let ptx = select::select(&l1)?;
    if stage == Stage::Select {
        return Ok(format!("{ptx:#?}"));
    }

    Ok(emit::emit(&ptx))
}
