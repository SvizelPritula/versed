use std::io::{Result, stdout};

use c_sharp::idents::CSharpIdentRules;
use idents::{CamelCase, PascalCase, convert_case};
use source_writer::SourceWriter;

pub mod ast;
pub mod c_sharp;
pub mod idents;
pub mod r#macro;
pub mod metadata;
pub mod source_writer;

fn main() -> Result<()> {
    let types = r#types! {
        Name = string;

        User = (struct {
            name: Name,
            age: (enum { age: number, unknown: unit }),
            contact: Contact
        });

        Contact = (versioned enum {
            phone: number,
            email: string,
            address: (struct {
                street: string,
                city: string,
                country: string
            })
        })
    };

    println!("{types:#?}");

    let mut writer = SourceWriter::new(stdout().lock());

    writer.write_fmt_nl(format_args!("public class {} {{", "User"))?;
    writer.indent();
    writer.write_fmt_nl(format_args!(
        "public required string {} {{ get; set; }}",
        "Name"
    ))?;
    writer.write_fmt_nl(format_args!(
        "public required ContactType {} {{ get; set; }}",
        "Contact"
    ))?;
    writer.nl()?;

    writer.write_fmt_nl(format_args!("public class {} {{", "ContactType"))?;
    writer.indent();
    writer.write_fmt_nl(format_args!(
        "public required string {} {{ get; set; }}",
        "Email"
    ))?;
    writer.dedent();
    writer.write_nl("}")?;

    writer.dedent();
    writer.write_nl("}")?;

    for ident in [
        "my_field", "my-field", "my field", "myField", "MyField", "struct", "10",
    ] {
        println!(
            "{} {}",
            convert_case(ident, CamelCase, CSharpIdentRules),
            convert_case(ident, PascalCase, CSharpIdentRules)
        );
    }

    Ok(())
}
