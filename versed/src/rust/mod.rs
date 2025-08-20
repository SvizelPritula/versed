use std::{fs::File, io::{Result, Write}, path::Path};

use crate::{
    ast::TypeSet,
    codegen::{
        idents::{PascalCase, SnakeCase},
        naming::{NameMetadata, name},
        source_writer::SourceWriter,
    },
    composite, mapper,
    name_resolution::ResolutionMetadata,
    rust::{
        idents::{RustIdentRules, RustModIdentRules},
        types::emit_types,
    },
};

mod idents;
mod types;

pub fn generate_types(types: TypeSet<ResolutionMetadata>, output: &Path) -> Result<()> {
    let types = name(
        types,
        PascalCase,
        SnakeCase,
        PascalCase,
        SnakeCase,
        RustIdentRules,
        RustModIdentRules,
        AddName,
    );

    let mut writer = SourceWriter::new(File::create(output)?);

    emit_types(&mut writer, &types)?;

    writer.into_inner().flush()?;
    Ok(())
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
