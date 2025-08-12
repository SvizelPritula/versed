use crate::metadata::Metadata;

pub use generation::emit;
pub use idents::CSharpIdentRules;
pub use naming::name;

mod generation;
mod idents;
mod naming;

#[derive(Debug, Clone)]
pub struct IdentMetadata {
    pub ident: String,
}

#[derive(Debug, Clone)]
pub struct CSharpMetadata;

impl Metadata for CSharpMetadata {
    type Struct = IdentMetadata;
    type Enum = IdentMetadata;
    type Identifier = ();
    type Name = IdentMetadata;
    type Field = IdentMetadata;
    type Variant = IdentMetadata;
}
