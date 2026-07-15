//! Versed's preprocessing pass.
//!
//! It resolves names and checks for
//! duplicate migration markers (which are errors)
//! and unbounded recursion (which is a warning).
//! It also checks whether both versions in a schema file have the same name (which is an error).
//!
use ariadne::{Color, Label, Report, ReportKind};
use name_resolution::resolve_names;

mod annotation_check;
mod name_resolution;
mod recursion_check;

pub use name_resolution::ResolutionMetadata;

use crate::{
    ast::{Migration, TypeSet},
    composite,
    preprocessing::{annotation_check::check_annotations, recursion_check::check_recursion},
    reports::Reports,
    syntax::SpanMetadata,
};

/// Runs the preprocessing pass on a schema file, resolving names and running some checks.
pub fn preprocess<'filename>(
    types: TypeSet<SpanMetadata>,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) -> TypeSet<BasicMetadata> {
    let types = resolve_names(types, reports, filename);
    check_annotations(&types, reports, filename);
    check_recursion(&types, reports, filename);

    types
}

/// Runs the preprocessing pass on a migration file, resolving names and running some checks.
pub fn preprocess_migration<'filename>(
    migration: Migration<SpanMetadata>,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) -> Migration<BasicMetadata> {
    let migration = migration.map(|types| preprocess(types, reports, filename));
    check_migration_versions(&migration.new, &migration.old, reports, filename);

    migration
}

/// Checks whether both versions in a schema file have the same name, producing a report if they do.
pub fn check_migration_versions<'filename>(
    new: &TypeSet<BasicMetadata>,
    old: &TypeSet<BasicMetadata>,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) {
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
}

composite! {
    pub struct (BasicInfo, BasicMetadata) {
        resolution: ResolutionMetadata | R,
        span: SpanMetadata | S
    }
}
