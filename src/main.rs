use std::{
    fmt::Display,
    fs,
    io::{self, BufWriter, Write, stdout},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anstream::stderr;
use anstyle::{AnsiColor, Color, Style};
use ariadne::Source;
use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{Generator, Shell};

use crate::{
    ast::TypeSet,
    preprocessing::{BasicMetadata, preprocess},
    reports::Reports,
    rust::RustOptions,
    syntax::parse,
};

pub mod ast;
pub mod codegen;
pub mod metadata;
pub mod preprocessing;
pub mod reports;
pub mod rust;
pub mod syntax;
pub mod typescript;

/// A tool for generating DTOs from schema descriptions
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
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
    },
    /// Print the schema version from the header of a schema file
    Version {
        /// The path to the schema file
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
    },
    /// Commands related to Rust
    Rust {
        #[command(subcommand)]
        command: RustCommand,
    },
    /// Commands related to TypeScript
    #[command(name = "typescript")]
    TypeScript {
        #[command(subcommand)]
        command: TypeScriptCommand,
    },
    /// Generate a tab-completion script for your shell
    Completions {
        /// The shell to target
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
enum RustCommand {
    /// Generate type declarations
    ///
    /// The generated types will be written to a new file inside the output directory named after
    /// the version of the schema.
    /// A corresponding module reference will be added to mod.rs.
    Types {
        /// The path to the schema file
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
        /// The path to the directory in which to create a file with the generated types
        #[arg(value_hint = ValueHint::AnyPath)]
        output: PathBuf,
        /// Interpret <OUTPUT> as a file instead of as a directory
        #[arg(short = 'f', long)]
        to_file: bool,
        /// Derive another trait
        #[arg(short = 'd', long)]
        derive: Vec<String>,
        /// Derive Serialize and Deserialize and add appropriate attributes from the serde crate
        #[arg(short = 's', long)]
        serde: bool,
    },
}

#[derive(Subcommand, Debug)]
enum TypeScriptCommand {
    /// Generate type declarations
    ///
    /// The generated types will be written to a new file inside the output directory named after
    /// the version of the schema.
    /// A corresponding import statement will be added to index.ts.
    Types {
        /// The path to the schema file
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
        /// The path to the directory in which to create a file with the generated types
        #[arg(value_hint = ValueHint::AnyPath)]
        output: PathBuf,
        /// Interpret <OUTPUT> as a file instead of as a directory
        #[arg(short = 'f', long)]
        to_file: bool,
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
            Ok(TypeSet { version, .. }) => handle_io_result({
                let mut file = stdout().lock();
                writeln!(file, "{version}").and_then(|()| file.flush())
            }),
            Err(code) => code,
        },
        Command::Rust {
            command:
                RustCommand::Types {
                    file,
                    output,
                    to_file,
                    derive,
                    serde,
                },
        } => match load_file(&file) {
            Ok(types) => {
                let options = RustOptions::new(serde, derive);
                handle_io_result(rust::generate_types(types, &options, &output, to_file))
            }
            Err(code) => code,
        },
        Command::TypeScript {
            command:
                TypeScriptCommand::Types {
                    file,
                    output,
                    to_file,
                },
        } => match load_file(&file) {
            Ok(types) => handle_io_result(typescript::generate_types(types, &output, to_file)),
            Err(code) => code,
        },
        Command::Completions { shell } => handle_io_result({
            let mut command = Args::command();
            command.set_bin_name(command.get_name().to_string());
            command.build();

            let mut file = stdout().lock();

            shell
                .try_generate(&command, &mut file)
                .and_then(|()| file.flush())
        }),
    }
}

fn print_error<E: Display>(error: &E) {
    const STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

    let mut stream = BufWriter::new(stderr().lock());
    let _ = writeln!(stream, "{STYLE}Error:{STYLE:#} {error}");
}

fn handle_io_result(result: io::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(&error);
            ExitCode::from(exit_codes::IO)
        }
    }
}

/// Loads and parses the file, printing any errors
fn load_file(file: &Path) -> Result<TypeSet<BasicMetadata>, ExitCode> {
    let filename = file.to_string_lossy();
    let src = fs::read_to_string(file)
        .inspect_err(print_error)
        .map_err(|_| ExitCode::from(exit_codes::IO))?;
    let mut reports = Reports::default();

    let ast = parse(&src, &mut reports, &filename);

    let ast = if let Some(ast) = ast {
        let ast = preprocess(ast, &mut reports, &filename);
        Some(ast)
    } else {
        None
    };

    if reports.has_any() {
        let mut stream = BufWriter::new(stderr().lock());
        let mut cache = (filename.as_ref(), Source::from(src.as_str()));

        for report in &reports {
            report
                .write(&mut cache, &mut stream)
                // If writing the report failed, then printing the error will probably fail too, but might as well try
                .inspect_err(print_error)
                .map_err(|_| ExitCode::from(exit_codes::IO))?;
        }
    }

    if let Some(ast) = ast
        && !reports.has_fatal()
    {
        Ok(ast)
    } else {
        Err(ExitCode::from(exit_codes::MALFORMED_FILE))
    }
}
