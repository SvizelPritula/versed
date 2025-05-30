use crate::metadata::Metadata;

pub use idents::CSharpIdentRules;
pub use naming::name;

mod idents;
mod naming;

#[derive(Debug, Clone)]
pub struct IdentMetadata {
    pub ident: String,
}

#[derive(Debug, Clone)]
pub struct CSharpMetadata;

impl Metadata for CSharpMetadata {
    type Name = IdentMetadata;
    type Struct = IdentMetadata;
    type Field = IdentMetadata;
    type Enum = IdentMetadata;
    type Variant = IdentMetadata;
}
