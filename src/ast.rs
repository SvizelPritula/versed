//! Declares all AST nodes.
//!
//! See the docs of the [`crate::metadata`] module for a description of the `M` parameter.

use crate::metadata::Metadata;

/// Represents an anonymous type.
#[derive(Debug, Clone)]
pub struct Type<M: Metadata> {
    pub r#type: TypeType<M>,
    pub number: Option<u64>,
    pub metadata: M::Type,
}

/// Represents the type of an anonymous type, as well as its type-specific attributes.
#[derive(Debug, Clone)]
pub enum TypeType<M: Metadata> {
    Struct(Struct<M>),
    Enum(Enum<M>),
    List(List<M>),
    Primitive(Primitive<M>),
    Identifier(Identifier<M>),
}

/// Represents a `struct { }` node.
#[derive(Debug, Clone)]
pub struct Struct<M: Metadata> {
    pub fields: Vec<Field<M>>,
    pub metadata: M::Struct,
}

/// Represents a field of a struct.
#[derive(Debug, Clone)]
pub struct Field<M: Metadata> {
    pub name: String,
    pub r#type: Type<M>,
    pub metadata: M::Field,
}

/// Represents an `enum { }` node.
#[derive(Debug, Clone)]
pub struct Enum<M: Metadata> {
    pub variants: Vec<Variant<M>>,
    pub metadata: M::Enum,
}

/// Represents a variant of an enum.
#[derive(Debug, Clone)]
pub struct Variant<M: Metadata> {
    pub name: String,
    pub r#type: Type<M>,
    pub metadata: M::Variant,
}

/// Represents a `[ ]` node.
#[derive(Debug, Clone)]
pub struct List<M: Metadata> {
    pub r#type: Box<Type<M>>,
    pub metadata: M::List,
}

/// Represents an `int`/`string`/`unit` node.
#[derive(Debug, Clone)]
pub struct Primitive<M: Metadata> {
    pub r#type: PrimitiveType,
    pub metadata: M::Primitive,
}

/// Represents the type of a primitive type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    String,
    Number,
    Unit,
}

/// Represents an identifier node.
#[derive(Debug, Clone)]
pub struct Identifier<M: Metadata> {
    pub ident: String,
    pub metadata: M::Identifier,
}

/// Represents the root of a schema file, or half of a migration file.
#[derive(Debug, Clone)]
pub struct TypeSet<M: Metadata> {
    pub version: String,
    pub types: Vec<NamedType<M>>,
    pub metadata: M::TypeSet,
}

/// Represents a top-level type with a name assigned to it.
#[derive(Debug, Clone)]
pub struct NamedType<M: Metadata> {
    pub name: String,
    pub r#type: Type<M>,
    pub metadata: M::Named,
}

/// Represents the root of a migration file.
#[derive(Debug, Clone)]
pub struct Migration<M: Metadata> {
    pub old: TypeSet<M>,
    pub new: TypeSet<M>,
}

impl<M: Metadata> Migration<M> {
    /// Maps both versions using a function.
    pub fn map<N: Metadata>(self, mut f: impl FnMut(TypeSet<M>) -> TypeSet<N>) -> Migration<N> {
        Migration {
            old: f(self.old),
            new: f(self.new),
        }
    }
}
