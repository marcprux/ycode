// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

mod cli;
mod project;
mod res;

use clap::Parser;
use std::{
    io::{self, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    let result = match cli::Cli::parse().command {
        cli::Command::Project(args) => project::run(args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(ycode_project::Error::Io(e)) if e.kind() == io::ErrorKind::BrokenPipe => {
            ExitCode::SUCCESS
        }
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "{}: {error}", res::str::error());
            ExitCode::FAILURE
        }
    }
}
