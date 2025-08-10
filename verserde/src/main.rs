use std::io::{Read, Result, stdin};

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::Parser;

use crate::syntax::lexer::lexer;

pub mod ast;
pub mod c_sharp;
pub mod codegen;
pub mod r#macro;
pub mod metadata;
pub mod syntax;

fn main() -> Result<()> {
    let mut src = String::new();
    stdin().lock().read_to_string(&mut src)?;

    let (tokens, errs) = lexer().parse(&src).into_output_errors();

    for error in errs {
        Report::build(ReportKind::Error, error.span().into_range())
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_message(error.to_string())
            .with_label(
                Label::new(error.span().into_range())
                    .with_message(error.reason().to_string())
                    .with_color(Color::Red),
            )
            .finish()
            .print(Source::from(&src))?
    }

    if let Some(tokens) = tokens {
        for (token, _span) in tokens {
            println!("{token:?}");
        }
    }

    Ok(())
}
