use std::io::{Read, Result, stdin};

use ariadne::sources;

use crate::{ast::TypeSet, syntax::parse};

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

    let (types, reports) = parse(&src, filename);

    for report in reports {
        report.print(sources([(filename, &src)]))?;
    }

    if let Some(TypeSet { types, version }) = types {
        println!("Version: {version}");
        println!("Types: {types:#?}");
    }

    Ok(())
}
