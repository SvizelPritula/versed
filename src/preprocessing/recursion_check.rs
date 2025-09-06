use std::{collections::HashMap, ops::Range};

use ariadne::{Color, Label, Report, ReportKind};

use crate::{
    ast::{Type, TypeSet},
    preprocessing::{name_resolution::INVALID_INDEX, BasicMetadata},
    reports::Reports,
    syntax::Span,
};

struct RecursionContext<'types> {
    types: &'types TypeSet<BasicMetadata>,
    cache: HashMap<usize, bool>,
    source: usize,
}

pub fn check_recursion<'filename>(
    types: &TypeSet<BasicMetadata>,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) {
    for (source, r#type) in types.types.iter().enumerate() {
        let mut context = RecursionContext {
            types,
            cache: HashMap::new(),
            source,
        };

        if check_type(&r#type.r#type, &mut context) {
            reports.add_nonfatal(make_report(
                format!(
                    "the type '{name}' will unavoidably have infinite depth",
                    name = r#type.name
                ),
                r#type.metadata.span.span,
                filename,
            ));
        }
    }
}

fn check_named(index: usize, context: &mut RecursionContext) -> bool {
    if let Some(result) = context.cache.get(&index) {
        return *result;
    }

    if index == context.source {
        return true;
    }

    if index == INVALID_INDEX {
        return false;
    }

    let result = check_type(&context.types.types[index].r#type, context);
    context.cache.insert(index, result);
    result
}

fn check_type(r#type: &Type<BasicMetadata>, context: &mut RecursionContext) -> bool {
    match r#type {
        Type::Struct(r#struct) => r#struct
            .fields
            .iter()
            .any(|field| check_type(&field.r#type, context)),
        Type::Enum(r#enum) => r#enum
            .variants
            .iter()
            .all(|variant| check_type(&variant.r#type, context)),
        Type::List(_list) => false,
        Type::Primitive(_primitive) => false,
        Type::Identifier(identifier) => check_named(identifier.metadata.resolution, context),
    }
}

fn make_report(
    message: String,
    span: Span,
    filename: &str,
) -> Report<'static, (&str, Range<usize>)> {
    Report::build(ReportKind::Warning, (filename, span.into_range()))
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
        .with_message(message.clone())
        .with_label(
            Label::new((filename, span.into_range()))
                .with_message(message)
                .with_color(Color::Red),
        )
        .finish()
}
