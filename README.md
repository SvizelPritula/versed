# Versed

Versed is a tool for generating DTO type definitions in Rust (and soon TypeScript) based on
a schema description in a simple custom language based on algebraic data types.

## Example

Given a schema like this:

```
// schema.vs
version v1;

User = struct {
    name: string,
    age: enum { known: int, unknown },
    contacts: [Contact],
};

Contact = enum {
    phone: int,
    email: string,
    address: struct {
        street: string,
        city: string,
        country: string,
    },
};
```

You can run `versed rust types schema.vs src/schema/`
to generate corresponding Rust type declarations:

```rs
// src/schema/v1.rs
#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub age: UserAge,
    pub contacts: Vec<Contact>,
}

#[derive(Debug, Clone)]
pub enum UserAge {
    Known(i64),
    Unknown(()),
}

#[derive(Debug, Clone)]
pub enum Contact {
    Phone(i64),
    Email(String),
    Address(ContactAddress),
}

#[derive(Debug, Clone)]
pub struct ContactAddress {
    pub street: String,
    pub city: String,
    pub country: String,
}
```

## Schema files

Schema files consist of a version header and any number of named types.

### Version header

Schema files must start with their version, like so:

```
version v1;
```

The version name is used to name the generated Rust module.
This way you can have multiple versions of the same schema in your project.
In the future, you will be able to migrate values from one version to another
using semi-automatically generated migration functions.

### Named types

A schema may contain any number of (uniquely) named types.
Each consists of a name, an equal sign, the type itself and a semicolon:

```
Keyword = string;
```

There are five types of types to choose from.

### Primitive types

There are currently three primitive types:

| Name     | Contents                                                                         | Equivalent Rust type |
| -------- | -------------------------------------------------------------------------------- | -------------------- |
| `int`    | a 64-bit signed integer                                                          | `i64`                |
| `string` | a sequence of Unicode code points                                                | `String`             |
| `unit`   | the [unit type](https://en.wikipedia.org/wiki/Unit_type) with one possible value | `()`                 |

### Lists

Any type can be surrounded by square brackets to turn in into a list:

```
Vector = [int];
Matrix = [[int]];
```

Lists are translated to Rust `Vec`s.

### Structs

Structs are composite types with zero or more fields, like `struct`s in Rust:

```
User = struct {
    id: int,
    name: string,
};
```

Structs can be nested:

```
User = struct {
    id: int,
    name: struct {
        first: string,
        last: string,
    },
};
```

### Enums

Enums represent [tagged unions](https://en.wikipedia.org/wiki/Tagged_union), like `enum`s in Rust.
An enum has several variants, each a possible value that the enum can take:

```
Element = enum {
    heading: string,
    paragraph: string,
    image: struct {
        url: string,
        width: int,
    },
    horizontal_line: unit,
};
```

As you can see, enums and structs can be freely nested.
The type of a variant can be omitted, in which case it defaults to `unit`:

```
Color = enum {
    red,
    green,
    blue,
};
```

### Identifiers

You can also refer directly to named types using their name:

```
User = struct {
    name: Name,
    email: Email,
    friends: [Friend],
};

Name = struct {
    first: string,
    last: string,
};
Email = string;
Friend = struct { name: Name };
```

Types can also be recursive:

```
Category = struct {
    name: string,
    subcategories: [Category],
};
```

`versed` will sometimes have to insert `Box`es
into the generated Rust type declarations to make that work.

## Installation

You can install the `versed` command from [crates.io](https://crates.io/crates/versed)
using [Cargo](https://github.com/rust-lang/cargo):

```sh
cargo install versed
```

## License

Copyright 2025 Benjamin Swart

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this software except in compliance with the License.
You may obtain a copy of the License at:

[http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an **"AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND**, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

Unless You explicitly state otherwise, any Contribution intentionally
submitted for inclusion in the Work by You to the Licensor shall be
under the terms and conditions of this License, without any additional
terms or conditions.
