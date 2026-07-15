//! Checks for and warns on unbounded recursion.
//!
//! Unbounded recursion means that a type will inevitably contain itself.
//! This means there is a dependency cycle that does not involve a list or an enum,
//! not counting enums where all variants contain the original type.
//! It will also not trigger for uninhabited types.

use std::{collections::HashMap, ops::Range};

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind};

use crate::{
    ast::{Type, TypeSet, TypeType},
    preprocessing::{BasicMetadata, name_resolution::INVALID_INDEX},
    reports::Reports,
    syntax::Span,
};

/// The context for the recursion check pass, valid for one iteration.
struct RecursionContext<'types> {
    types: &'types TypeSet<BasicMetadata>,
    cache: HashMap<usize, CheckResult>,
}

/// The result of checking a type .
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CheckResult {
    // In order or increasing priority:
    None,
    ContainsNever,
    InfiniteDepth,
}

/// Runs the recursion check pass.
pub fn check_recursion<'filename>(
    types: &TypeSet<BasicMetadata>,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) {
    for r#type in &types.types {
        let mut context = RecursionContext {
            types,
            cache: HashMap::new(),
        };

        if check_type(&r#type.r#type, &mut context) == CheckResult::InfiniteDepth {
            reports.add_nonfatal(make_report(
                format!(
                    "the type '{name}' will unavoidably have infinite depth",
                    name = r#type.name
                ),
                r#type.metadata.span.name,
                filename,
            ));
        }
    }
}

/// Runs the check for one top-level type.
fn check_named(index: usize, context: &mut RecursionContext) -> CheckResult {
    if let Some(result) = context.cache.get(&index) {
        return *result;
    }

    if index == INVALID_INDEX {
        return CheckResult::None;
    }

    context.cache.insert(index, CheckResult::InfiniteDepth);
    let result = check_type(&context.types.types[index].r#type, context);
    context.cache.insert(index, result);
    result
}

/// Visits and checks a type recursively.
fn check_type(r#type: &Type<BasicMetadata>, context: &mut RecursionContext) -> CheckResult {
    match &r#type.r#type {
        TypeType::Struct(r#struct) => r#struct
            .fields
            .iter()
            .map(|field| check_type(&field.r#type, context))
            .max()
            .unwrap_or(CheckResult::None),
        TypeType::Enum(r#enum) => r#enum
            .variants
            .iter()
            .map(|variant| check_type(&variant.r#type, context))
            .min()
            .unwrap_or(CheckResult::ContainsNever),
        TypeType::List(_list) => CheckResult::None,
        TypeType::Primitive(_primitive) => CheckResult::None,
        TypeType::Identifier(identifier) => check_named(identifier.metadata.resolution, context),
    }
}

/// Creates a report.
fn make_report(
    message: String,
    span: Span,
    filename: &str,
) -> Report<'static, (&str, Range<usize>)> {
    Report::build(ReportKind::Warning, (filename, span.into_range()))
        .with_config(Config::new().with_index_type(IndexType::Byte))
        .with_message(message.clone())
        .with_label(
            Label::new((filename, span.into_range()))
                .with_message(message)
                .with_color(Color::Red),
        )
        .finish()
}
