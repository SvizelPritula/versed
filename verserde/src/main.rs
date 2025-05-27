pub mod ast;
pub mod r#macro;

fn main() {
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
}
