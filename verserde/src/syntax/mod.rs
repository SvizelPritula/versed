use std::{fmt::Display, ops::Range};

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::{error::Rich, span::SimpleSpan};

pub mod lexer;
pub mod parser;
pub mod tokens;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

pub fn make_report<'a, 'b, T: Display>(
    error: Rich<'a, T>,
    filename: &'b str,
) -> Report<'a, (&'b str, Range<usize>)> {
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
