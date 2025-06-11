use std::io::{BufWriter, Result, stdout};

use c_sharp::{emit, name};
use source_writer::SourceWriter;

pub mod ast;
pub mod c_sharp;
pub mod idents;
pub mod r#macro;
pub mod metadata;
pub mod source_writer;

fn main() -> Result<()> {
    let types = types! {
        Names = [Name];
        Name = string;

        User = (struct {
            name: Name,
            tags: [string],
            age: (enum { age: number, unknown: unit }),
            contact: Contact
        });

        Contact = (versioned [enum {
            phone: number,
            email: string,
            address: (struct {
                street: string,
                city: string,
                country: string
            })
        }])
    };

    let types = name(types);

    let writer = BufWriter::new(stdout().lock());
    emit(&types, &mut SourceWriter::new(writer))
}
