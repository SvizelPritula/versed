use std::io::{Result, stdout};

use c_sharp::{emit, name};
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

    let types = name(types);

    emit(&types, &mut SourceWriter::new(stdout().lock()))
}
