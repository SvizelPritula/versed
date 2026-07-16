# The Versed schema language

Schema files consist of a version header and any number of named types.

## Version header

Schema files must start with their version, like so:

```
version v1;
```

The version name is used to name the generated Rust module.
This way you can have multiple versions of the same schema in your project.

## Named types

A schema may contain any number of (uniquely) named types.
Each consists of a name, an equal sign, the type itself and a semicolon:

```
Keyword = string;
```

There are five types of types to choose from.

## Primitive types

There are currently three primitive types:

| Name     | Contents                                                                         | Equivalent Rust type | Equivalent TypeScript type |
| -------- | -------------------------------------------------------------------------------- | -------------------- | -------------------------- |
| `int`    | a 64-bit signed integer                                                          | `i64`                | `number`                   |
| `string` | a sequence of Unicode code points                                                | `String`             | `string`                   |
| `unit`   | the [unit type](https://en.wikipedia.org/wiki/Unit_type) with one possible value | `()`                 | `null`                     |

## Lists

Any type can be surrounded by square brackets to turn it into a list:

```
Vector = [int];
Matrix = [[int]];
```

In Rust, lists are translated to `Vec`s.

## Structs

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

## Enums

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

## Identifiers

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

## Migration files

Migration files are simply the concatenation of two schema files.
They usually contain multiple migration markers.
Migration markers are also valid in schema files, where they are ignored.
