use std::{fmt::Display, ops::Range};

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::{Parser, container::Container, error::Rich, input::Input, span::SimpleSpan};

use crate::{
    ast::TypeSet,
    metadata::Metadata,
    syntax::{lexer::lexer, parser::parser},
};

pub mod lexer;
pub mod parser;
pub mod tokens;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

pub fn parse<'src, 'filename>(
    src: &'src str,
    filename: &'filename str,
) -> (
    Option<TypeSet<SpanMetadata>>,
    Vec<Report<'static, (&'filename str, Range<usize>)>>,
) {
    let mut reports = Vec::new();

    let (tokens, errors) = lexer().parse(&src).into_output_errors();
    reports.extend(errors.into_iter().map(|error| make_report(error, filename)));

    let result = if let Some(tokens) = tokens {
        let tokens = tokens
            .as_slice()
            .map((src.len()..src.len()).into(), |(t, s)| (t, s));

        let (ast, errors) = parser().parse(tokens).into_output_errors();
        reports.extend(errors.into_iter().map(|error| make_report(error, filename)));

        ast
    } else {
        None
    };

    (result, reports)
}

fn make_report<'src, 'tokens, T: Display>(
    error: Rich<'src, T>,
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
pub struct IdentSpan {
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct SpanMetadata;
impl Metadata for SpanMetadata {
    type Struct = ();
    type Enum = ();
    type Identifier = IdentSpan;

    type Name = IdentSpan;
    type Field = ();
    type Variant = ();
}

#[derive(Debug, Clone)]
struct ExtendVec<T>(Vec<T>);

impl<T> Default for ExtendVec<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<I, T> Container<I> for ExtendVec<T>
where
    I: IntoIterator<Item = T>,
{
    fn push(&mut self, item: I) {
        self.0.extend(item);
    }

    fn with_capacity(n: usize) -> Self {
        Self(Vec::with_capacity(n))
    }
}
