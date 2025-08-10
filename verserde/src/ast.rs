use crate::metadata::Metadata;

#[derive(Debug, Clone)]
pub enum Type<M: Metadata> {
    Struct(Struct<M>),
    Enum(Enum<M>),
    List(Box<Type<M>>),
    Primitive(Primitive),
    Identifier(String),
}

#[derive(Debug, Clone)]
pub struct Struct<M: Metadata> {
    pub fields: Vec<Field<M>>,
    pub metadata: M::Struct,
}

#[derive(Debug, Clone)]
pub struct Field<M: Metadata> {
    pub name: String,
    pub r#type: Type<M>,
    pub metadata: M::Field,
}

#[derive(Debug, Clone)]
pub struct Enum<M: Metadata> {
    pub variants: Vec<Variant<M>>,
    pub metadata: M::Enum,
}

#[derive(Debug, Clone)]
pub struct Variant<M: Metadata> {
    pub name: String,
    pub r#type: Type<M>,
    pub metadata: M::Variant,
}

#[derive(Debug, Clone, Copy)]
pub enum Primitive {
    String,
    Number,
    Unit,
}

#[derive(Debug, Clone)]
pub struct TypeSet<M: Metadata> {
    pub version: String,
    pub types: Vec<NamedType<M>>,
}

#[derive(Debug, Clone)]
pub struct NamedType<M: Metadata> {
    pub name: String,
    pub r#type: Type<M>,
    pub metadata: M::Name,
}
