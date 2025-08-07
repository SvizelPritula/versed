pub mod v1 {
    pub struct User {
        pub name: String,
        pub contact: Contact,
    }

    pub enum Contact {
        Phone(i64),
        Email(String),
    }
}

pub mod v2 {
    pub struct User {
        pub name: String,
        pub age: UserAge,
        pub contact: Vec<Contact>,
    }

    pub enum UserAge {
        Age(i64),
        Unknown,
    }

    pub enum Contact {
        Phone(i64),
        Email(String),
        Address(ContactAddress),
    }

    pub struct ContactAddress {
        pub street: String,
        pub city: String,
        pub country: String,
    }
}

pub mod migrations {
    pub mod v2 {
        use crate::{
            v1,
            v2::{self, User},
        };

        fn upgrade_user(user: v1::User) -> v2::User {
            let v1::User { name, contact } = user;

            User {
                name,
                age: v2::UserAge::Unknown,
                contact: vec![upgrade_contact(contact)],
            }
        }

        fn upgrade_contact(contact: v1::Contact) -> v2::Contact {
            match contact {
                v1::Contact::Phone(phone) => v2::Contact::Phone(phone),
                v1::Contact::Email(email) => v2::Contact::Email(email),
            }
        }
    }
}
