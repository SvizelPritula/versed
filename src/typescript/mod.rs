//! The TypeScript language backend.

use std::{
    fs::{File, create_dir_all},
    io::{BufWriter, Write},
    path::Path,
};

use crate::{
    ast::TypeSet,
    codegen::{
        file_patching::add_line_to_file,
        naming_pass::{NameMetadata, name},
        source_writer::SourceWriter,
    },
    composite,
    error::{Error, ResultExt},
    loading::load_file,
    mapper,
    preprocessing::{BasicMetadata, ResolutionMetadata},
    typescript::types::emit_types,
};

mod idents;
mod types;

pub use idents::TypeScriptNamingRules;

/// Runs TypeScript-specific passes to convert [`BasicMetadata`] into [`TypeScriptMetadata`].
fn convert_types(types: TypeSet<BasicMetadata>) -> TypeSet<TypeScriptMetadata> {
    name(types, TypeScriptNamingRules, AddName)
}

/// Implements `versed typescript types`.
pub fn generate_types(path: &Path, output: &Path, to_file: bool) -> Result<(), Error> {
    let types = load_file(path)?;
    let types = convert_types(types);

    if to_file {
        write_to_file(&types, output, false)
    } else {
        write_to_directory(&types, output)
    }
}

/// Saves type declarations into a specific directory and adds a re-export to `index.ts`.
fn write_to_directory(types: &TypeSet<TypeScriptMetadata>, path: &Path) -> Result<(), Error> {
    create_dir_all(path).with_path(path)?;
    let mod_name = &types.metadata.name;

    let type_path = path.join(format!("{mod_name}.ts"));
    write_to_file(types, &type_path, true)?;

    let index_path = path.join("index.ts");
    add_reexport_to_file(mod_name, &index_path)?;

    Ok(())
}

/// Saves type declarations to a specific file.
fn write_to_file(
    types: &TypeSet<TypeScriptMetadata>,
    path: &Path,
    must_be_new: bool,
) -> Result<(), Error> {
    let file = if must_be_new {
        File::create_new(path).with_path(path)?
    } else {
        File::create(path).with_path(path)?
    };

    let mut writer = SourceWriter::new(BufWriter::new(file));
    emit_types(&mut writer, types).with_path(path)?;
    writer.into_inner().flush().with_path(path)?;

    Ok(())
}

/// Appends a re-export to a file.
fn add_reexport_to_file(module_name: &str, path: &Path) -> Result<(), Error> {
    add_line_to_file(
        path,
        format_args!("export * as {module_name} from \"./{module_name}\";"),
    )
    .with_path(path)
}

composite! {
    struct (TypeScriptInfo, TypeScriptMetadata) {
        name: NameMetadata | N,
        resolution: ResolutionMetadata | R
    }
}

mapper! {
    fn AddName(basic: BasicMetadata, name: NameMetadata) -> TypeScriptMetadata {
        TypeScriptInfo {
            name,
            resolution: basic.resolution,
        }
    }
}
