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

/// A tool for generating DTOs from schema descriptions
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

fn main() -> ExitCode {
    let args = Args::parse();

    match args.command {
        Command::Check { file } => match load_file(&file) {
            Ok(_) => ExitCode::SUCCESS,
            Err(code) => code,
        },
        Command::Version { file } => match load_file(&file) {
            Ok(TypeSet { version, .. }) => handle_result({
                let mut file = stdout().lock();
                writeln!(file, "{version}")
                    .and_then(|()| file.flush())
                    .with_stdout()
            }),
            Err(code) => code,
        },
        Command::Migration {
            command: MigrationCommand::Begin { file },
        } => match load_file_with_source(&file) {
            Ok((types, src)) => handle_result(migrations::begin(&types, &src, &file)),
            Err(code) => code,
        },
        Command::Migration {
            command:
                MigrationCommand::Finish {
                    file: new_file,
                    migration: migration_file,
                },
        } => {
            let old_file = migrations::old_schema_path(&new_file);

            let (new_types, new_src) = match load_file_with_source(&new_file) {
                Ok(res) => res,
                Err(code) => return code,
            };
            let (old_types, old_src) = match load_file_with_source(&old_file) {
                Ok(res) => res,
                Err(code) => return code,
            };

            let filename = new_file.to_string_lossy();
            let reports = migrations::check_versions(&new_types, &old_types, &filename);

            match print_all_reports(&reports, &filename, &new_src) {
                Ok(()) => {}
                Err(code) => return code,
            }

            handle_result(migrations::finish(
                &new_types,
                &new_src,
                &old_src,
                &new_file,
                &old_file,
                &migration_file,
            ))
        }
        Command::Migration {
            command: MigrationCommand::Check { file },
        } => match load_migration(&file) {
            Ok(_) => ExitCode::SUCCESS,
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
                handle_result(rust::generate_types(types, &options, &output, to_file))
            }
            Err(code) => code,
        },
        Command::Rust {
            command:
                RustCommand::Migration {
                    file,
                    output,
                    to_file,
                },
        } => match load_migration(&file) {
            Ok(migration) => handle_result(rust::generate_migration(migration, &output, to_file)),
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
            Ok(types) => handle_result(typescript::generate_types(types, &output, to_file)),
            Err(code) => code,
        },
        Command::Completions { shell } => handle_result({
            let mut command = Args::command();
            command.set_bin_name(command.get_name().to_string());
            command.build();

            let mut file = stdout().lock();

            shell
                .try_generate(&command, &mut file)
                .and_then(|()| file.flush())
                .with_stdout()
        }),
    }
}

fn print_error<E: Display>(error: &E) {
    const STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

    let mut stream = BufWriter::new(stderr().lock());
    let _ = writeln!(stream, "{STYLE}Error:{STYLE:#} {error}");
}

fn handle_result(result: Result<(), Error>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(&error);

            ExitCode::from(match error {
                Error::Io { .. } => exit_codes::IO,
            })
        }
    }
}

/// Loads and parses the file, printing any errors
fn load_file(file: &Path) -> Result<TypeSet<BasicMetadata>, ExitCode> {
    load_file_with_source(file).map(|(types, _)| types)
}

fn load_file_with_source(file: &Path) -> Result<(TypeSet<BasicMetadata>, String), ExitCode> {
    let filename = file.to_string_lossy();
    let src = fs::read_to_string(file)
        .with_path(file)
        .inspect_err(print_error)
        .map_err(|_| ExitCode::from(exit_codes::IO))?;
    let mut reports = Reports::default();

    let ast = parse_schema(&src, &mut reports, &filename);

    let ast = ast.map(|types| preprocess(types, &mut reports, &filename));

    print_all_reports(&reports, &filename, &src)?;

    if let Some(ast) = ast {
        Ok((ast, src))
    } else {
        Err(ExitCode::from(exit_codes::MALFORMED_FILE))
    }
}

fn load_migration(file: &Path) -> Result<Migration<BasicMetadata>, ExitCode> {
    let filename = file.to_string_lossy();
    let src = fs::read_to_string(file)
        .with_path(file)
        .inspect_err(print_error)
        .map_err(|_| ExitCode::from(exit_codes::IO))?;
    let mut reports = Reports::default();

    let migration = parse_migration(&src, &mut reports, &filename);

    let migration = migration.map(|m| m.map(|types| preprocess(types, &mut reports, &filename)));

    print_all_reports(&reports, &filename, &src)?;

    if let Some(migration) = migration {
        Ok(migration)
    } else {
        Err(ExitCode::from(exit_codes::MALFORMED_FILE))
    }
}

fn print_all_reports(reports: &Reports, filename: &str, src: &str) -> Result<(), ExitCode> {
    if reports.has_any() {
        let mut stream = BufWriter::new(stderr().lock());
        let mut cache = (filename, Source::from(src));

        for report in reports {
            report
                .write(&mut cache, &mut stream)
                .with_stderr()
                // If writing the report failed, then printing the error will probably fail too, but might as well try
                .inspect_err(print_error)
                .map_err(|_| ExitCode::from(exit_codes::IO))?;
        }
    }

    if reports.has_fatal() {
        Err(ExitCode::from(exit_codes::MALFORMED_FILE))
    } else {
        Ok(())
    }
}
