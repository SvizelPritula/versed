use std::{
    fs::{self, File},
    io::{BufWriter, Result, Write},
    path::{Path, PathBuf},
};

use ariadne::{Color, Label, Report, ReportKind};

use crate::{
    ast::TypeSet,
    codegen::file_patching::{add_extention, apply_add_edits},
    migrations::annotate::annotate,
    preprocessing::BasicMetadata,
    reports::Reports,
};

mod annotate;

const OLD_EXTENSION: &str = ".old";

pub fn old_schema_path(new_path: &Path) -> PathBuf {
    add_extention(new_path, OLD_EXTENSION)
}

pub fn begin(types: &TypeSet<BasicMetadata>, src: &str, path: &Path) -> Result<()> {
    let edits = annotate(types);

    let old_path = old_schema_path(path);

    let mut file = BufWriter::new(File::create_new(&old_path)?);
    apply_add_edits(&mut file, src, edits)?;
    file.flush()?;

    fs::copy(&old_path, path)?;

    Ok(())
}

pub fn finish(
    new_types: &TypeSet<BasicMetadata>,
    new_src: &str,
    old_src: &str,
    new_path: &Path,
    old_path: &Path,
    migration_path: &Path,
) -> Result<()> {
    // TODO: Strip annotations
    let _ = new_types;
    let _ = new_path;

    let mut migration_file = BufWriter::new(File::create(migration_path)?);

    migration_file.write_all(old_src.as_bytes())?;
    migration_file.write_all(new_src.as_bytes())?;

    migration_file.flush()?;

    fs::remove_file(old_path)?;

    Ok(())
}

pub fn check_versions<'filename>(
    new: &TypeSet<BasicMetadata>,
    old: &TypeSet<BasicMetadata>,
    filename: &'filename str,
) -> Reports<'filename> {
    let mut reports = Reports::default();

    if new.version == old.version {
        let message = "the new schema has the same version as the old schema";

        let report = Report::build(
            ReportKind::Error,
            (filename, new.metadata.span.version.into_range()),
        )
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
        .with_message(message)
        .with_label(
            Label::new((filename, new.metadata.span.version.into_range()))
                .with_message(message)
                .with_color(Color::Red),
        )
        .finish();

        reports.add_fatal(report);
    }

    reports
}
