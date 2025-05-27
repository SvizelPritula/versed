#[derive(Debug, Clone)]
pub struct Type<Metadata> {
    metadata: Metadata,
    r#type: TypeType<Metadata>,
}

#[derive(Debug, Clone)]
pub enum TypeType<Metadata> {
    Struct(Struct<Metadata>),
    Enum(Enum<Metadata>),
    Versioned(Versioned<Metadata>),
    Primitive(Primitive),
    Identifier(String),
}

#[derive(Debug, Clone)]
pub struct Struct<Metadata> {
    pub fields: Vec<Field<Metadata>>,
}

#[derive(Debug, Clone)]
pub struct Field<Metadata> {
    pub name: String,
    pub r#type: Type<Metadata>,
}

#[derive(Debug, Clone)]
pub struct Enum<Metadata> {
    pub variants: Vec<Variant<Metadata>>,
}

#[derive(Debug, Clone)]
pub struct Variant<Metadata> {
    pub name: String,
    pub r#type: Type<Metadata>,
}

#[derive(Debug, Clone)]
pub struct Versioned<Metadata> {
    pub r#type: Box<Type<Metadata>>,
    // TODO: Add versioning metadata
}

#[derive(Debug, Clone, Copy)]
pub enum Primitive {
    String,
    Number,
}
