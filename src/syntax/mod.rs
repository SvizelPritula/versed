use std::{fmt::Display, ops::Range};

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::{Parser, error::Rich, extra, input::Input as _, span::SimpleSpan};

use crate::{
    ast::{Migration, TypeSet},
    metadata::Metadata,
    reports::Reports,
    syntax::{
        lexer::lexer,
        parser::{Error, Input, migration_file_parser, schema_file_parser},
    },
};

pub mod lexer;
pub mod parser;
pub mod tokens;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

fn parse<'filename, P, O>(
    parser: P,
    src: &str,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) -> Option<O>
where
    P: ParserFactory<O>,
{
    let (tokens, errors) = lexer().parse(src).into_output_errors();
    reports.extend_fatal(
        errors
            .into_iter()
            .map(|error| make_report(&error, filename)),
    );

    if let Some(tokens) = tokens {
        let tokens = tokens
            .as_slice()
            .map((src.len()..src.len()).into(), |(t, s)| (t, s));

        let (ast, errors) = parser.make().parse(tokens).into_output_errors();
        reports.extend_fatal(
            errors
                .into_iter()
                .map(|error| make_report(&error, filename)),
        );

        ast
    } else {
        None
    }
}

pub fn parse_schema<'filename>(
    src: &str,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) -> Option<TypeSet<SpanMetadata>> {
    struct Factory;
    impl ParserFactory<TypeSet<SpanMetadata>> for Factory {
        fn make<'tokens, I: Input<'tokens>>(
            self,
        ) -> impl Parser<'tokens, I, TypeSet<SpanMetadata>, extra::Err<Error<'tokens>>> {
            schema_file_parser()
        }
    }

    parse(Factory, src, reports, filename)
}

pub fn parse_migration<'filename>(
    src: &str,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) -> Option<Migration<SpanMetadata>> {
    struct Factory;
    impl ParserFactory<Migration<SpanMetadata>> for Factory {
        fn make<'tokens, I: Input<'tokens>>(
            self,
        ) -> impl Parser<'tokens, I, Migration<SpanMetadata>, extra::Err<Error<'tokens>>> {
            migration_file_parser()
        }
    }

    parse(Factory, src, reports, filename)
}

// There is no way to specify that the parser argument of parse has to implement
// Parser<'t, I, ...> for every I: Input<'t>, necessitating this trait.
trait ParserFactory<O> {
    fn make<'tokens, I: Input<'tokens>>(
        self,
    ) -> impl Parser<'tokens, I, O, extra::Err<Error<'tokens>>>;
}

fn make_report<'tokens, T: Display>(
    error: &Rich<T>,
    filename: &'tokens str,
) -> Report<'static, (&'tokens str, Range<usize>)> {
    Report::build(ReportKind::Error, (filename, error.span().into_range()))
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
        .with_message(error.to_string())
        .with_label(
            Label::new((filename, error.span().into_range()))
                .with_message(error.to_string())
                .with_color(Color::Red),
        )
        .finish()
}

#[derive(Debug, Clone, Copy)]
pub struct TypeSpanInfo {
    pub r#type: Span,
    pub number: Option<Span>,
}

impl TypeSpanInfo {
    pub fn number_or_type(&self) -> Span {
        self.number.unwrap_or(self.r#type)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemberSpanInfo {
    pub name: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct TypeSetSpanInfo {
    pub version: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct SpanMetadata;
impl Metadata for SpanMetadata {
    type Type = TypeSpanInfo;
    type TypeSet = TypeSetSpanInfo;
    type Named = MemberSpanInfo;

    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type Field = MemberSpanInfo;
    type Variant = MemberSpanInfo;
}

#[derive(Debug, Clone)]
struct FromIterFlatten<Collection>(Collection);

impl<Collection, Item, Inner> FromIterator<Inner> for FromIterFlatten<Collection>
where
    Collection: FromIterator<Item>,
    Inner: IntoIterator<Item = Item>,
{
    fn from_iter<U: IntoIterator<Item = Inner>>(iter: U) -> Self {
        FromIterFlatten(iter.into_iter().flatten().collect())
    }
}
