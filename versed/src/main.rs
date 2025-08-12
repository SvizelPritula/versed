use std::{
    fmt::Display,
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anstream::{stderr, stdout};
use anstyle::{AnsiColor, Color, Style};
use ariadne::Source;
use clap::{Parser, Subcommand};

use crate::{
    ast::TypeSet,
    syntax::{SpanMetadata, parse},
};

pub mod ast;
pub mod c_sharp;
pub mod codegen;
pub mod r#macro;
pub mod metadata;
pub mod syntax;

/// A tool for generating DTO's and their migrations from schema descriptions
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Check if a file is syntactically and semantically well-formed
    ///
    /// Will exit with exit code 0 if it is and with exit code 1 if it isn't.
    Check {
        /// The path to the file to check
        file: PathBuf,
    },
    /// Print the schema version from the header of a schema file
    Version {
        /// The path to the schema file
        file: PathBuf,
    },
}

mod exit_codes {
    pub const MALFORMED_FILE: u8 = 1;
    pub const _USAGE: u8 = 2; // Used by clap
    pub const IO: u8 = 3;
}

fn main() -> ExitCode {
    let args = Args::parse();

    match args.command {
        Command::Check { file } => match load_file(&file) {
            Ok(_) => ExitCode::SUCCESS,
            Err(code) => code,
        },
        Command::Version { file } => match load_file(&file) {
            Ok(TypeSet { version, .. }) => {
                let result = writeln!(stdout().lock(), "{version}").inspect_err(print_error);

                if result.is_ok() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(exit_codes::IO)
                }
            }
            Err(code) => code,
        },
    }
}

fn print_error<E: Display>(error: &E) {
    const STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

    let mut stream = BufWriter::new(stderr().lock());
    let _ = writeln!(stream, "{STYLE}Error:{STYLE:#} {error}");
}

/// Loads and parses the file, printing any errors
fn load_file(file: &Path) -> Result<TypeSet<SpanMetadata>, ExitCode> {
    let filename = file.to_string_lossy();
    let src = fs::read_to_string(file)
        .inspect_err(print_error)
        .map_err(|_| ExitCode::from(exit_codes::IO))?;

    let (ast, reports) = parse(&src, &filename);
    let has_errors = !reports.is_empty();

    if !reports.is_empty() {
        let mut stream = BufWriter::new(stderr().lock());
        let mut cache = (filename.as_ref(), Source::from(src.as_str()));

        for report in reports {
            report
                .write(&mut cache, &mut stream)
                // If writing the report failed, then printing the error will probably fail too, but might as well try
                .inspect_err(print_error)
                .map_err(|_| ExitCode::from(exit_codes::IO))?;
        }
    }

    if let Some(ast) = ast
        && !has_errors
    {
        Ok(ast)
    } else {
        Err(ExitCode::from(exit_codes::MALFORMED_FILE))
    }
}
