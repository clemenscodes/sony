mod application;

use crate::application::CommandLine;
use clap::Parser;
use std::io::{stderr, stdout};
use std::process::ExitCode;

fn main() -> ExitCode {
    let line = CommandLine::parse();
    let mut output = stdout();
    let mut errors = stderr();
    match line.try_act(&mut output, &mut errors) {
        Ok(outcome) => ExitCode::from(outcome),
        Err(_) => ExitCode::FAILURE,
    }
}
