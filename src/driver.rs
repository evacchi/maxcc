//! The pipeline. Stops after `stage` and prints that stage's output.
//!
//!   source --lex--> tokens --parse--> L0 --select--> L-ptx --emit--> .ptx

use crate::{emit, lexer, parser, select};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Lex,
    Parse,
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

    let ptx = select::select(&ast)?;
    if stage == Stage::Select {
        return Ok(format!("{ptx:#?}"));
    }

    Ok(emit::emit(&ptx))
}
