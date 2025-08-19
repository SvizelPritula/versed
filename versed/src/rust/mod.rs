use crate::{
    ast::TypeSet,
    codegen::{
        idents::{PascalCase, SnakeCase},
        naming::{NameMetadata, name},
    },
    composite, mapper,
    name_resolution::ResolutionMetadata,
    rust::idents::RustIdentRules,
};

mod idents;

pub fn generate_types(types: TypeSet<ResolutionMetadata>) {
    let types = name(
        types,
        PascalCase,
        SnakeCase,
        PascalCase,
        RustIdentRules,
        AddName,
    );

    println!("{types:#?}");
}

composite! {
    struct (RustInfo, RustMetadata) {
        name: NameMetadata | N,
        resolution: ResolutionMetadata | R
    }
}

mapper! {
    fn AddName(resolution: ResolutionMetadata, name: NameMetadata) -> RustMetadata {
        RustInfo { name, resolution }
    }
}
