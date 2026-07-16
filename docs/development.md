# The developer's guide

There are two main resources that describe Versed's source code:
The documentation, which contains an annotated list of every module, function, type, etc.,
and this document, which describes Versed's high-level architecture and code layout,
as well as its testing approach.

## The documentation

Versed uses `rustdoc` to build its documentation.
To build the documentation, simply run the following command in the root of the repo:

```sh
cargo doc --document-private-items
```

To view the documentation, simply open `target/doc/versed/index.html`
in your browser afterwards.

## The architecture

The Versed compiler is split into three main parts:
the shared language frontend, the various backends,
and the backend support library.

## The frontend

The frontend of the compiler is used by (almost) all commands
and is responsible for parsing the input file into an abstract syntax tree,
as well as name resolution,
i.e. assigning every reference to a named type
the correct index into the array of named types.

The main entrypoint of the frontend is the `loading` module.
It exports functions for loading both schema files and migration files into memory.
Each function takes the path to the file to read as its only argument
and returns a fully resolved AST.

### The syntax module

The `loading` module first loads the file into memory,
after which it calls into the `syntax` module.
The syntax module contains two main submodules, `lexer` and `parser`.
The lexer is responsible for turning the file into a list of tokens,
while the parses parses the tokens into an AST.
Both are written using the [Chumsky](https://docs.rs/chumsky/latest/chumsky/index.html) crate.
The root of the crate also contains functions that convert Chumsky's errors
into [Ariadne](https://docs.rs/ariadne/latest/ariadne/index.html) reports,
which is how the rest of the compiler represents errors and warnings.
The public functions of the `syntax` module handle the entirety
of turning the contents of a schema/migration files into
both an AST and a list of reports.
The lexer and parser support error recovery,
so they'll (almost) always return an AST, even when faced with errors.
This allows syntactical and semantic errors to be returned in one compiler invocation.

### The AST

The AST is defined using two modules, `ast` and `metadata`,
which work in tandem.
The `ast` module contains the nodes that make up the AST.
The AST is represented using plain Rust structs and enums.
The `metadata` module contains the foundations of Versed's metadata system,
which is similar to the technique described in [Trees that Grow](https://lib.jucs.org/article/22912).
It allows different parts of the compiler to attach various pieces of metadata
to the AST, all while maintaining full type safety.

The most important part of the metadata system is the `Metadata` trait.
The `Metadata` trait has one associated type for each node in the AST.
Every AST node is generic over `M: Metadata`,
and contains a `metadata: M::[node type]` field.
Every phase of the compiler declares a marker type implementing `Metadata`,
with the associated types defining the type of the additional value attached to each node.

For example, the parser returns `TypeSet<SpanMetadata>` (or `Migration<SpanMetadata>`),
where `SpanMetadata` is defined as follows:
```rs
#[derive(Debug, Clone, Copy)]
pub struct SpanMetadata;

impl Metadata for SpanMetadata {
    type Type = TypeSpanInfo;
    type TypeSet = TypeSetSpanInfo;
    type Named = MemberSpanInfo;

    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type Field = MemberSpanInfo;
    type Variant = MemberSpanInfo;
}
```
This means that types will be decorated with the `TypeSpanInfo` struct,
the full schema with `TypeSetSpanInfo`,
and named types, fields and variants with `MemberSpanInfo`.
Other nodes will have the unit type attached.

An important part of the metadata system is the `composite!` macro,
which allows for the composition of multiple `Metadata` implementations into one.
For example:
```rs
composite! {
    pub struct (BasicInfo, BasicMetadata) {
        resolution: ResolutionMetadata | R,
        span: SpanMetadata | S
    }
}
```
This example expands to:
```rs
#[derive(Debug, Clone)]
pub struct BasicInfo<R, S> {
    pub resolution: R,
    pub span: S,
}
#[derive(Debug, Clone, Copy)]
pub struct BasicMetadata;

impl crate::metadata::Metadata for BasicMetadata {
    type Type = BasicInfo<
        <ResolutionMetadata as crate::metadata::Metadata>::Type,
        <SpanMetadata as crate::metadata::Metadata>::Type,
    >;
    type TypeSet = BasicInfo<
        <ResolutionMetadata as crate::metadata::Metadata>::TypeSet,
        <SpanMetadata as crate::metadata::Metadata>::TypeSet,
    >;
    // And so on...
}
```

The more detailed description of the metadata system can be found in Section 10.2 of the thesis.

### The preprocessor

The third step performed by the `loading` module
is a call into the `preprocessing` module.

This module runs several recursive passes over the AST.
The first of them is the name resolution pass,
which assigns an index into the main type array to every identifier.
This turns the `TypeSet<SpanMetadata>` into a `TypeSet<BasicMetadata>`.
The other two passes are purely diagnostic.
The first issues errors if any migration marker is present more than once.
The second of them issues warnings if there is any type with unbounded recursion.

After the preprocessor finishes, the `loading` module prints any errors or warnings
and returns the resultant AST if no errors occurred.

## The backends

The Versed compiler contains multiple backends
— modules which take the AST and turn it into some output.
The backends are grouped into four groups based on the type of output they produce.
Two of them are language specific, while two of them are general.

### The simple backends

The three simplest commands that Versed implements
are `versed check`, `versed migration check` and `versed version`.
The first two don't really have a backend — they just run the frontend and terminate.
The version backend simply prints `TypeSet::version`.
All three of them are implemented in a couple lines within `main.rs`.

### The migration backends

These are the backends that power `versed migration begin` and `versed migration finish`.
Both modify the schema file in place, while also touching some other files.
They are implemented within the `migrations` module.
Both edit the schema file by first assembling a list of edits
using the source code spans provided by the compiler.
These edits are then applied using a separate pass over the schema file.

### The TypeScript backend

The TypeScript language support module (`typescript`)
only exports one backend, which powers `versed typescript types`.
It converts the schema to TypeScript types,
using the support library in a straightforward way.

### The Rust backends

The Rust language support module (`rust`)
exports two backends, one for type declarations, one for migrations.
Both utilize the support library.
The Rust module is a little more complicated than the TypeScript module,
as it might need to insert `Box`es or newtypes into the schema.
It does this using two recursive passes over the schema,
their result is stored as metadata.

One other interesting detail of the Rust module is that it depends on the TypeScript module,
in order to reference its configuration for the naming pass.
It does this since it needs to reproduce the results of TypeScript's naming pass,
so that it can output matching Serde attributes.

## The support library

The support library contains pieces of code used by multiple backends,
or that could be used by multiple backends in the future.
Most of it is located in the `codegen` module.

One of its contents is the naming pass,
which assigns new names to every type, field and variant
based on rules defined by the target language.
It is highly generic.
Firstly, it is generic over the configuration provided by the language,
which consists of an implementation of `CaseType` and `IdentRules` for every kind of named entity.
Each language defines its own `IdentRules`,
while `CaseType`s are shared and defined in the `idents` submodule,
alongside the generic case conversion code.
The naming pass is also generic over the type of metadata attached to the AST
before (`A`) and after (`B`) the naming pass runs,
as well as a collection of functions to turn `A::[node type]` + `name` into `B::[node type]`.
This is needed since different backends might need to
run the pass after already attaching custom metadata to the AST,
and since the Rust module needs to run the pass multiple times.

Another helper defined by the support library is `SourceWriter`,
which wraps a `Writer` to provide useful methods for producing source code.
It can track the indentation level as the file is being written,
and can help with adding blank lines in an aesthetically pleasing way.

Some other parts of the support library include
a helper for appending a line to a file,
the file patching pass used by the migration backends,
and the code to pair up types with matching migration markers.

## The CLI

Most of the code related to the CLI is contained within `main.rs`.
It uses [clap](https://docs.rs/clap/latest/clap/index.html) to parse the arguments
and to output help screens, using its Derive API.
The `main` functions calls `run_command` and handles errors,
`run_command` contains a large match statement which forwards each command to its handler function.

Handler functions are part of their backends.
They are responsible for invoking the frontend themselves,
i.e. calling the correct function(s) from `loading`,
as well as any backend-specific IO.

Every backend implements its own IO (using the provided helpers).
This is needed since backends need to perform IO in radically different ways —
`versed rust types` reads one schema file and writes one and edits one Rust file,
`versed migration begin` edits one and writes one schema file,
`versed migration finish` edits one and deletes one schema file and writes a migration file,
and so on.
There is no uniform interface for calling backends for the same reason.
It could be useful to implement sans-IO versions of all commands in the future,
but the API design would not be exactly trivial and there is no present need.

## The tests

The Versed compiler has several tests.
All of them take the form of integration tests and are located inside the `tests/` directory.
They invoke the compiler using its CLI on various schemas.
Aside from merely verifying that the compiler can parse the schemas,
they also attempt to convert them to TypeScript and Rust type declarations
and compile those using `tsc` and `rustc` or `cargo`.
As such, all of those commands need to be installed to run the full test suite.

There are also tests that compile migration functions,
that make sure the Serde attributes and TypeScript declarations match, etc.
See the `tests/` folder for a full list.
Many test groups `include!` `tests/utils/test_schemas.inc.rs`,
which defines a base set of both simple and tricky schemas.
