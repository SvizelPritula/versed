use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use ariadne::{Color, Label, Report, ReportKind};

use crate::{
    ast::TypeSet,
    codegen::file_patching::{add_extention, apply_add_edits, apply_remove_edits, concat_files},
    error::{Error, ResultExt},
    handle_reports, load_file_with_source,
    migrations::annotate::{annotate, strip_annotations},
    preprocessing::BasicMetadata,
    reports::Reports,
};

pub use pairing::{TypePair, pair_types};

mod annotate;
mod pairing;

const OLD_EXTENSION: &str = ".old";

fn old_schema_path(new_path: &Path) -> PathBuf {
    add_extention(new_path, OLD_EXTENSION)
}

pub fn begin(path: &Path) -> Result<(), Error> {
    let (types, src) = load_file_with_source(path)?;
    let edits = annotate(&types);

    let old_path = old_schema_path(path);

    let mut file = BufWriter::new(File::create_new(&old_path).with_path(&old_path)?);
    apply_add_edits(&mut file, &src, edits).with_path(&old_path)?;
    file.flush().with_path(&old_path)?;

    fs::copy(&old_path, path).with_path(path)?;

    Ok(())
}

pub fn finish(new_path: &Path, migration_path: &Path) -> Result<(), Error> {
    let old_path = old_schema_path(new_path);

    let (new_types, new_src) = load_file_with_source(new_path)?;
    let (old_types, old_src) = load_file_with_source(&old_path)?;

    let filename = new_path.to_string_lossy();
    let reports = check_versions(&new_types, &old_types, &filename);
    handle_reports(&reports, &filename, &new_src)?;

    concat_files(&old_src, &new_src, migration_path).with_path(migration_path)?;

    let edits = strip_annotations(&new_types);

    let mut file = BufWriter::new(File::create(new_path).with_path(new_path)?);
    apply_remove_edits(&mut file, &new_src, edits).with_path(new_path)?;
    file.flush().with_path(new_path)?;

    fs::remove_file(&old_path).with_path(&old_path)?;

    Ok(())
}

fn check_versions<'filename>(
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
