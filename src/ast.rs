use crate::metadata::Metadata;

#[derive(Debug, Clone)]
pub struct Type<M: Metadata> {
    pub r#type: TypeType<M>,
    pub number: Option<u64>,
    pub metadata: M::Type,
}

#[derive(Debug, Clone)]
pub enum TypeType<M: Metadata> {
    Struct(Struct<M>),
    Enum(Enum<M>),
    List(List<M>),
    Primitive(Primitive<M>),
    Identifier(Identifier<M>),
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

#[derive(Debug, Clone)]
pub struct List<M: Metadata> {
    pub r#type: Box<Type<M>>,
    pub metadata: M::List,
}

#[derive(Debug, Clone)]
pub struct Primitive<M: Metadata> {
    pub r#type: PrimitiveType,
    pub metadata: M::Primitive,
}

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveType {
    String,
    Number,
    Unit,
}

#[derive(Debug, Clone)]
pub struct Identifier<M: Metadata> {
    pub ident: String,
    pub metadata: M::Identifier,
}

#[derive(Debug, Clone)]
pub struct TypeSet<M: Metadata> {
    pub version: String,
    pub types: Vec<NamedType<M>>,
    pub metadata: M::TypeSet,
}

#[derive(Debug, Clone)]
pub struct NamedType<M: Metadata> {
    pub name: String,
    pub r#type: Type<M>,
    pub metadata: M::Named,
}
