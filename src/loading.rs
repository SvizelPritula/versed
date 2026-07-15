//! The public interface of the language frontend.
//!
//! Handles parsing through [`crate::syntax`] and name resolution and some checks through [`crate::preprocessing`].
//! Also prints all generated [`ariadne::Report`]s, for lifetime reasons.

use std::{fs, path::Path};

use crate::{
    ast::{Migration, TypeSet},
    error::{Error, ResultExt},
    preprocessing::{BasicMetadata, preprocess, preprocess_migration},
    reports::{Reports, handle_reports},
    syntax::{parse_migration, parse_schema},
};

/// Loads and parses a schema file, printing any errors.
pub fn load_file(file: &Path) -> Result<TypeSet<BasicMetadata>, Error> {
    load_file_with_source(file).map(|(types, _)| types)
}

/// Loads and parses a schema file, printing any errors, returning the source code as well as the AST.
pub fn load_file_with_source(file: &Path) -> Result<(TypeSet<BasicMetadata>, String), Error> {
    let filename = file.to_string_lossy();
    let src = fs::read_to_string(file).with_path(file)?;
    let mut reports = Reports::default();

    let ast = parse_schema(&src, &mut reports, &filename);
    let ast = ast.map(|types| preprocess(types, &mut reports, &filename));

    handle_reports(&reports, &filename, &src)?;
    ast.ok_or(Error::MalformedFile).map(|ast| (ast, src))
}

/// Loads and parses a migration file, printing any errors.
pub fn load_migration(file: &Path) -> Result<Migration<BasicMetadata>, Error> {
    let filename = file.to_string_lossy();
    let src = fs::read_to_string(file).with_path(file)?;
    let mut reports = Reports::default();

    let migration = parse_migration(&src, &mut reports, &filename);
    let migration = migration.map(|m| preprocess_migration(m, &mut reports, &filename));

    handle_reports(&reports, &filename, &src)?;
    migration.ok_or(Error::MalformedFile)
}
