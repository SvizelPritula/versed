use std::{
    collections::{HashMap, hash_map::Entry},
    ops::Range,
};

use ariadne::{Color, Label, Report, ReportKind};

use crate::{
    Reports,
    ast::{Enum, Field, Identifier, List, NamedType, Primitive, Struct, Type, TypeSet, Variant},
    metadata::Metadata,
    syntax::{Span, SpanMetadata},
};

#[derive(Debug)]
struct NameInfo {
    index: usize,
    span: Span,
}

pub fn resolve_and_check(
    TypeSet {
        version,
        types,
        metadata: (),
    }: TypeSet<SpanMetadata>,
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

    (
        TypeSet {
            version,
            types,
            metadata: (),
        },
        reports,
    )
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
            check_unique(
                fields
                    .iter()
                    .map(|Field { name, metadata, .. }| (name.as_str(), metadata.span)),
                "field",
                filename,
                reports,
            );

            let fields = fields
                .into_iter()
                .map(
                    |Field {
                         name,
                         r#type,
                         metadata: _,
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
            check_unique(
                variants
                    .iter()
                    .map(|Variant { name, metadata, .. }| (name.as_str(), metadata.span)),
                "variant",
                filename,
                reports,
            );

            let variants = variants
                .into_iter()
                .map(
                    |Variant {
                         name,
                         r#type,
                         metadata: _,
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
        Type::List(List {
            r#type,
            metadata: (),
        }) => Type::List(List {
            r#type: Box::new(resolve_type(*r#type, names, filename, reports)),
            metadata: (),
        }),
        Type::Primitive(Primitive {
            r#type,
            metadata: (),
        }) => Type::Primitive(Primitive {
            r#type,
            metadata: (),
        }),
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

fn check_unique<'a, 'filename>(
    iter: impl Iterator<Item = (&'a str, Span)>,
    type_name: &'a str,
    filename: &'filename str,
    reports: &mut Reports<'filename>,
) {
    let mut names = HashMap::new();

    for (name, span) in iter {
        match names.entry(name) {
            Entry::Occupied(entry) => reports.push(make_double_label_report(
                format!("the {type_name} '{name}' was declared multiple times"),
                format!("the {type_name} '{name}' was declared again here"),
                span,
                format!("the {type_name} '{name}' was first declared here"),
                *entry.get(),
                filename,
            )),
            Entry::Vacant(entry) => {
                entry.insert(span);
            }
        };
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
    type List = ();
    type Primitive = ();
    type Identifier = Resolution;

    type TypeSet = ();
    type Named = ();
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
