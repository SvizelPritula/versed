use std::io::stdout;

use crate::{
    ast::TypeSet,
    codegen::{
        idents::{PascalCase, SnakeCase},
        naming::{NameMetadata, name},
        source_writer::SourceWriter,
    },
    composite, mapper,
    name_resolution::ResolutionMetadata,
    rust::{idents::RustIdentRules, types::emit_types},
};

mod idents;
mod types;

pub fn generate_types(types: TypeSet<ResolutionMetadata>) {
    let types = name(
        types,
        PascalCase,
        SnakeCase,
        PascalCase,
        RustIdentRules,
        AddName,
    );

    let output = stdout().lock();
    let mut writer = SourceWriter::new(output);

    emit_types(&mut writer, &types).unwrap();
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
