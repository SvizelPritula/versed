# Versed

[Versed](https://crates.io/crates/versed) is a tool for generating DTO type definitions
in Rust and TypeScript based on a schema description
in a simple custom language based on algebraic data types.
The types it generates can be serialized to JSON,
using [Serde](https://serde.rs/) in Rust and `JSON.serialize(…)` in TypeScript,
and deserialized again in any supported language.

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

You can run `versed rust types schema.vs src/schema/ --serde`
to generate corresponding Rust type declarations:

```rs
// src/schema/v1.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub age: UserAge,
    pub contacts: Vec<Contact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum UserAge {
    #[serde(rename = "known")]
    Known(i64),
    #[serde(rename = "unknown")]
    Unknown(()),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Contact {
    #[serde(rename = "phone")]
    Phone(i64),
    #[serde(rename = "email")]
    Email(String),
    #[serde(rename = "address")]
    Address(ContactAddress),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactAddress {
    pub street: String,
    pub city: String,
    pub country: String,
}
```

You can also run `versed typescript types schema.vs src/schema/`
to generate TypeScript type declarations:

```ts
// src/schema/v1.ts
export type User = {
    name: string,
    age: (
        {
            type: "known",
            value: number,
        } | {
            type: "unknown",
            value: null,
        }
    ),
    contacts: Contact[],
};

export type Contact = (
    {
        type: "phone",
        value: number,
    } | {
        type: "email",
        value: string,
    } | {
        type: "address",
        value: {
            street: string,
            city: string,
            country: string,
        },
    }
);
```

`versed` automatically converts identifiers based on the naming convention of the target language,
i.e. PascalCase/snake_case for Rust and PascalCase/camelCase/kebab-case for TypeScript.

## Usage

There are two main commands, `versed rust types` and `versed typescript types`.
Both take two arguments, the path to the schema and the path to the output directory,
in that order.
Versed will write the generated types to a new file inside the output directory
named after the version of the schema.
For example, if you run `versed rust types schema.vs src/schema/`
and `schema.vs` starts with `version v1;`,
then the types will be written to `src/schema/v1.rs`.
It will also add an import of this file to `mod.rs` or `index.ts`.
The directory will be created if it doesn't exist.

You can also use the `-f` or `--to-file` flag to write the types directly to a specified file,
which will cause the second argument to be interpreted as the path to that file
instead of a directory.
For example, `versed rust types schema.vs src/current-schema.rs` will simply
write the types to `src/current-schema.rs`.

If you only want to check if a schema file is syntactically and semantically well-formed,
you can use `versed check`.
There is also `versed version`, which will additionally
output the version of the schema.

Lastly, there is `versed completions`, which prints out a script for providing tab-completion
for `versed` for the specified shell.
For example, you can install tab-completions for bash like this:

```sh
versed completions bash > ~/.local/share/bash-completion/completions/versed
```

You can run `versed help` for a more detailed usage description.

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

| Name     | Contents                                                                         | Equivalent Rust type | Equivalent TypeScript type |
| -------- | -------------------------------------------------------------------------------- | -------------------- | -------------------------- |
| `int`    | a 64-bit signed integer                                                          | `i64`                | `number`                   |
| `string` | a sequence of Unicode code points                                                | `String`             | `string`                   |
| `unit`   | the [unit type](https://en.wikipedia.org/wiki/Unit_type) with one possible value | `()`                 | `null`                     |

### Lists

Any type can be surrounded by square brackets to turn it into a list:

```
Vector = [int];
Matrix = [[int]];
```

In Rust, lists are translated to `Vec`s.

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
