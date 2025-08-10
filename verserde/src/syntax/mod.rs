use std::{fmt::Display, ops::Range};

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::{Parser, error::Rich, input::Input, span::SimpleSpan};

use crate::{
    ast::TypeSet,
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
    Option<TypeSet<()>>,
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

pub fn make_report<'src, 'tokens, T: Display>(
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
