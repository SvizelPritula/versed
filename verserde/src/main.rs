use std::io::{Read, Result, stdin};

use ariadne::sources;
use chumsky::{Parser, input::Input};

use crate::syntax::{lexer::lexer, make_report, parser::parser};

pub mod ast;
pub mod c_sharp;
pub mod codegen;
pub mod r#macro;
pub mod metadata;
pub mod syntax;

fn main() -> Result<()> {
    let mut src = String::new();
    stdin().lock().read_to_string(&mut src)?;
    let filename = "input";

    let (tokens, errors) = lexer().parse(&src).into_output_errors();
    for error in errors {
        make_report(error, filename).print(sources([(filename, &src)]))?
    }

    if let Some(tokens) = tokens {
        let tokens = tokens
            .as_slice()
            .map((src.len()..src.len()).into(), |(t, s)| (t, s));

        let (ast, errors) = parser().parse(tokens).into_output_errors();
        for error in errors {
            make_report(error, filename).print(sources([(filename, &src)]))?
        }

        if let Some((version, types)) = ast {
            println!("Version: {version}");
            println!("Types: {types:#?}");
        }
    }

    Ok(())
}
