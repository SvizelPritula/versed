use std::{
    collections::{HashMap, hash_map::Entry},
    ops::Range,
};

use ariadne::{Color, Label, Report, ReportKind};

use crate::{
    Reports,
    ast::{Enum, Field, Identifier, NamedType, Struct, Type, TypeSet, Variant},
    metadata::Metadata,
    syntax::{Span, SpanMetadata},
};

#[derive(Debug)]
struct NameInfo {
    index: usize,
    span: Span,
}

pub fn resolve_and_check(
    TypeSet { version, types }: TypeSet<SpanMetadata>,
    filename: &'_ str,
) -> (TypeSet<ResolutionMetadata>, Reports<'_>) {
    let mut names: HashMap<String, NameInfo> = HashMap::new();
    let mut reports = Vec::new();

    for (index, r#type) in types.iter().enumerate() {
        let name = &r#type.name;

        match names.entry(name.clone()) {
            Entry::Occupied(entry) => reports.push(make_double_label_report(
                format!("the name '{name}' was declared multiple times"),
                format!("the name '{name}' was used again here"),
                r#type.metadata.span,
                format!("the name '{name}' was first used here"),
                entry.get().span,
                filename,
            )),
            Entry::Vacant(entry) => {
                entry.insert(NameInfo {
                    index,
                    span: r#type.metadata.span,
                });
            }
        };
    }

    let types = types
        .into_iter()
        .map(
            |NamedType {
                 name,
                 r#type,
                 metadata: _,
             }| NamedType {
                name,
                r#type: resolve_type(r#type, &names, filename, &mut reports),
                metadata: (),
            },
        )
        .collect();

    (TypeSet { version, types }, reports)
}

fn resolve_type<'filename>(
    r#type: Type<SpanMetadata>,
    names: &HashMap<String, NameInfo>,
    filename: &'filename str,
    reports: &mut Reports<'filename>,
) -> Type<ResolutionMetadata> {
    match r#type {
        Type::Struct(Struct {
            fields,
            metadata: (),
        }) => {
            let fields = fields
                .into_iter()
                .map(
                    |Field {
                         name,
                         r#type,
                         metadata: (),
                     }| Field {
                        name,
                        r#type: resolve_type(r#type, names, filename, reports),
                        metadata: (),
                    },
                )
                .collect();

            Type::Struct(Struct {
                fields,
                metadata: (),
            })
        }
        Type::Enum(Enum {
            variants,
            metadata: (),
        }) => {
            let variants = variants
                .into_iter()
                .map(
                    |Variant {
                         name,
                         r#type,
                         metadata: (),
                     }| Variant {
                        name,
                        r#type: resolve_type(r#type, names, filename, reports),
                        metadata: (),
                    },
                )
                .collect();

            Type::Enum(Enum {
                variants,
                metadata: (),
            })
        }
        Type::List(inner) => Type::List(Box::new(resolve_type(*inner, names, filename, reports))),
        Type::Primitive(primitive) => Type::Primitive(primitive),
        Type::Identifier(Identifier { ident, metadata }) => {
            let index = if let Some(&NameInfo { index, .. }) = names.get(&ident) {
                index
            } else {
                reports.push(make_simple_report(
                    format!("unknown type '{ident}'"),
                    metadata.span,
                    filename,
                ));

                usize::MAX // Placeholder, will panic if used, which it should never be
            };

            Type::Identifier(Identifier {
                ident,
                metadata: Resolution { index },
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub index: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolutionMetadata;
impl Metadata for ResolutionMetadata {
    type Struct = ();
    type Enum = ();
    type Identifier = Resolution;
    type Name = ();
    type Field = ();
    type Variant = ();
}

fn make_simple_report(
    error: String,
    span: Span,
    filename: &str,
) -> Report<'static, (&str, Range<usize>)> {
    Report::build(ReportKind::Error, (filename, span.into_range()))
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
        .with_message(error.clone())
        .with_label(
            Label::new((filename, span.into_range()))
                .with_message(error)
                .with_color(Color::Red),
        )
        .finish()
}

fn make_double_label_report(
    error: String,
    primary_label: String,
    primary_span: Span,
    secondary_label: String,
    secondary_span: Span,
    filename: &str,
) -> Report<'static, (&str, Range<usize>)> {
    Report::build(ReportKind::Error, (filename, primary_span.into_range()))
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
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
