use std::{collections::HashMap, ops::Range};

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind};

use crate::{
    ast::{Type, TypeSet, TypeType},
    preprocessing::{BasicMetadata, name_resolution::INVALID_INDEX},
    reports::Reports,
    syntax::Span,
};

struct RecursionContext<'types> {
    types: &'types TypeSet<BasicMetadata>,
    cache: HashMap<usize, CheckResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CheckResult {
    None,
    ContainsNever,
    InfiniteDepth,
}

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
