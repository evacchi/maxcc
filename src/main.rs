//! usage: maxxcc [--lex|--parse|--flatten|--address|--select] <file.k>
//! Prints PTX (or the requested stage) to stdout.

mod address;
mod ast;
mod diag;
mod driver;
mod emit;
mod flat;
mod flatten;
mod l1;
mod lexer;
mod parser;
mod ptx;
mod select;

use std::process::ExitCode;

use driver::Stage;

const USAGE: &str = "usage: maxxcc [--lex|--parse|--flatten|--address|--select] <file.k>";

fn main() -> ExitCode {
    let mut stage = Stage::Emit;
    let mut path = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--lex" => stage = Stage::Lex,
            "--parse" => stage = Stage::Parse,
            "--flatten" => stage = Stage::Flatten,
            "--address" => stage = Stage::Address,
            "--select" => stage = Stage::Select,
            _ => path = Some(arg),
        }
    }

    let Some(path) = path else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };

    let src = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    match driver::compile(&src, stage) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{path}:{e}");
            ExitCode::FAILURE
        }
    }
}
