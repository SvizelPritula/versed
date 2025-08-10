use chumsky::span::SimpleSpan;

pub mod lexer;
pub mod parser;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);
