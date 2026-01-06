use std::{
    fmt::Display,
    fs,
    io::{BufWriter, Write, stdout},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anstream::stderr;
use anstyle::{AnsiColor, Color, Style};
use ariadne::Source;
use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{Generator, Shell};

use crate::{
    ast::{Migration, TypeSet},
    error::{Error, ResultExt},
    preprocessing::{BasicMetadata, preprocess},
    reports::Reports,
    rust::RustOptions,
    syntax::{parse_migration, parse_schema},
};

pub mod ast;
pub mod codegen;
pub mod error;
pub mod metadata;
pub mod migrations;
pub mod preprocessing;
pub mod reports;
pub mod rust;
pub mod syntax;
pub mod typescript;

/// A tool for generating DTOs and their migrations from schema descriptions
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Check if a schema file is syntactically and semantically well-formed
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
    /// Commands for creating migrations
    Migration {
        #[command(subcommand)]
        command: MigrationCommand,
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
enum MigrationCommand {
    /// Start a new migration, annotating and copying the schema file
    Begin {
        /// The path to the schema file
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
    },
    /// Finish a new migration, comparing the new and old schema to produce a migration file
    Finish {
        /// The path to the schema file
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
        /// The path to the migration file to output
        #[arg(value_hint = ValueHint::FilePath)]
        migration: PathBuf,
    },
    /// Check if a migration file is syntactically and semantically well-formed
    ///
    /// Will exit with exit code 0 if it is and with exit code 1 if it isn't.
    Check {
        /// The path to the migration file to check
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
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
        /// Interpret \<OUTPUT\> as a file instead of as a directory
        #[arg(
            short = 'f',
            long,
            help = "Interpret <OUTPUT> as a file instead of as a directory"
        )]
        to_file: bool,
        /// Derive another trait
        #[arg(short = 'd', long)]
        derive: Vec<String>,
        /// Derive Serialize and Deserialize and add appropriate attributes from the serde crate
        #[arg(short = 's', long)]
        serde: bool,
    },
    /// Generate migration
    Migration {
        /// The path to the schema file
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
        /// The path to the directory with the previously generated types
        #[arg(value_hint = ValueHint::AnyPath)]
        output: PathBuf,
        /// Interpret \<OUTPUT\> as a file instead of as a directory
        #[arg(
            short = 'f',
            long,
            help = "Interpret <OUTPUT> as a file instead of as a directory"
        )]
        to_file: bool,
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
        /// Interpret \<OUTPUT\> as a file instead of as a directory
        #[arg(
            short = 'f',
            long,
            help = "Interpret <OUTPUT> as a file instead of as a directory"
        )]
        to_file: bool,
    },
}

mod exit_codes {
    pub const MALFORMED_FILE: u8 = 1;
    pub const _USAGE: u8 = 2; // Used by clap
    pub const IO: u8 = 3;
}

fn handle_result(result: Result<(), Error>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error @ Error::Io { .. }) => {
            print_error(&error);
            ExitCode::from(exit_codes::IO)
        }
        Err(Error::MalformedFile) => ExitCode::from(exit_codes::MALFORMED_FILE),
    }
}

fn print_error<E: Display>(error: &E) {
    const STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

    let mut stream = BufWriter::new(stderr().lock());
    let _ = writeln!(stream, "{STYLE}Error:{STYLE:#} {error}");
}

fn main() -> ExitCode {
    let args = Args::parse();
    handle_result(run_command(args))
}

fn run_command(args: Args) -> Result<(), Error> {
    match args.command {
        Command::Check { file } => {
            load_file(&file)?;
        }
        Command::Version { file } => print_version(&file)?,
        Command::Migration {
            command: MigrationCommand::Begin { file },
        } => migrations::begin(&file)?,
        Command::Migration {
            command: MigrationCommand::Finish { file, migration },
        } => migrations::finish(&file, &migration)?,
        Command::Migration {
            command: MigrationCommand::Check { file },
        } => {
            load_migration(&file)?;
        }
        Command::Rust {
            command:
                RustCommand::Types {
                    file,
                    output,
                    to_file,
                    derive,
                    serde,
                },
        } => rust::generate_types(&file, &output, to_file, &RustOptions::new(serde, derive))?,
        Command::Rust {
            command:
                RustCommand::Migration {
                    file,
                    output,
                    to_file,
                },
        } => rust::generate_migration(&file, &output, to_file)?,
        Command::TypeScript {
            command:
                TypeScriptCommand::Types {
                    file,
                    output,
                    to_file,
                },
        } => typescript::generate_types(&file, &output, to_file)?,
        Command::Completions { shell } => print_completions(shell)?,
    }

    Ok(())
}

fn print_version(path: &Path) -> Result<(), Error> {
    let TypeSet { version, .. } = load_file(path)?;
    let mut file = stdout().lock();

    writeln!(file, "{version}")
        .and_then(|()| file.flush())
        .with_stdout()?;

    Ok(())
}

fn print_completions(shell: Shell) -> Result<(), Error> {
    let mut command = Args::command();
    command.set_bin_name(command.get_name().to_string());
    command.build();

    let mut file = stdout().lock();

    shell
        .try_generate(&command, &mut file)
        .and_then(|()| file.flush())
        .with_stdout()
}

/// Loads and parses the file, printing any errors
fn load_file(file: &Path) -> Result<TypeSet<BasicMetadata>, Error> {
    load_file_with_source(file).map(|(types, _)| types)
}

fn load_file_with_source(file: &Path) -> Result<(TypeSet<BasicMetadata>, String), Error> {
    let filename = file.to_string_lossy();
    let src = fs::read_to_string(file).with_path(file)?;
    let mut reports = Reports::default();

    let ast = parse_schema(&src, &mut reports, &filename);
    let ast = ast.map(|types| preprocess(types, &mut reports, &filename));

    handle_reports(&reports, &filename, &src)?;
    ast.ok_or(Error::MalformedFile).map(|ast| (ast, src))
}

fn load_migration(file: &Path) -> Result<Migration<BasicMetadata>, Error> {
    let filename = file.to_string_lossy();
    let src = fs::read_to_string(file).with_path(file)?;
    let mut reports = Reports::default();

    let migration = parse_migration(&src, &mut reports, &filename);
    let migration = migration.map(|m| m.map(|types| preprocess(types, &mut reports, &filename)));

    handle_reports(&reports, &filename, &src)?;
    migration.ok_or(Error::MalformedFile)
}

fn handle_reports(reports: &Reports, filename: &str, src: &str) -> Result<(), Error> {
    if reports.has_any() {
        let mut stream = BufWriter::new(stderr().lock());
        let mut cache = (filename, Source::from(src));

        for report in reports {
            report.write(&mut cache, &mut stream).with_stderr()?;
        }
    }

    if reports.has_fatal() {
        Err(Error::MalformedFile)
    } else {
        Ok(())
    }
}
