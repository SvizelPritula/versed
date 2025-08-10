use std::io::{Read, Result, stdin};

use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::{Parser, error::Rich};

use crate::syntax::lexer::lexer;

pub mod ast;
pub mod c_sharp;
pub mod codegen;
pub mod r#macro;
pub mod metadata;
pub mod syntax;

fn make_report<'a, 'b>(
    error: Rich<'a, char>,
    filename: &'b str,
) -> Report<'a, (&'b str, std::ops::Range<usize>)> {
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

fn main() -> Result<()> {
    let mut src = String::new();
    stdin().lock().read_to_string(&mut src)?;

    let (tokens, errs) = lexer().parse(&src).into_output_errors();

    let filename = "input";
    for error in errs {
        make_report(error, filename).print(sources([(filename, &src)]))?
    }

    if let Some(tokens) = tokens {
        for (token, _span) in tokens {
            println!("{token:?}");
        }
    }

    Ok(())
}
