use std::{
    collections::{HashMap, hash_map::Entry},
    ops::Range,
};

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind};

use crate::{
    ast::{Type, TypeSet, TypeType},
    preprocessing::BasicMetadata,
    reports::Reports,
    syntax::Span,
};

struct Context<'types, 'filename, 'reports> {
    used: HashMap<u64, &'types Type<BasicMetadata>>,
    reports: &'reports mut Reports<'filename>,
    filename: &'filename str,
}

pub fn check_annotations<'filename>(
    types: &TypeSet<BasicMetadata>,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) {
    let mut context = Context {
        used: HashMap::new(),
        reports,
        filename,
    };

    for r#type in &types.types {
        check_type(&r#type.r#type, &mut context);
    }
}

fn check_type<'types, 'f, 'r>(
    r#type: &'types Type<BasicMetadata>,
    context: &mut Context<'types, 'f, 'r>,
) {
    if let Some(number) = r#type.number {
        match context.used.entry(number) {
            Entry::Occupied(entry) => context.reports.add_fatal(make_report(
                format!("the type number #{number} was used multiple times"),
                format!("the type number #{number} was used again here"),
                r#type.metadata.span.number_or_type(),
                format!("the type number #{number} was first used here"),
                entry.get().metadata.span.number_or_type(),
                context.filename,
            )),
            Entry::Vacant(entry) => {
                entry.insert(r#type);
            }
        }
    }

    match &r#type.r#type {
        TypeType::Struct(r#struct) => {
            for field in &r#struct.fields {
                check_type(&field.r#type, context);
            }
        }
        TypeType::Enum(r#enum) => {
            for variant in &r#enum.variants {
                check_type(&variant.r#type, context);
            }
        }
        TypeType::List(list) => check_type(&list.r#type, context),
        TypeType::Primitive(_primitive) => {}
        TypeType::Identifier(_identifier) => {}
    }
}

fn make_report(
    error: String,
    primary_label: String,
    primary_span: Span,
    secondary_label: String,
    secondary_span: Span,
    filename: &str,
) -> Report<'static, (&str, Range<usize>)> {
    Report::build(ReportKind::Error, (filename, primary_span.into_range()))
        .with_config(Config::new().with_index_type(IndexType::Byte))
        .with_message(error)
        .with_label(
            Label::new((filename, primary_span.into_range()))
                .with_message(primary_label)
                .with_color(Color::Red),
        )
        .with_label(
            Label::new((filename, secondary_span.into_range()))
                .with_message(secondary_label)
                .with_color(Color::Yellow),
        )
        .finish()
}
