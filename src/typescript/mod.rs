use std::{
    fs::{File, create_dir_all},
    io::{BufWriter, Result, Write},
    path::Path,
};

use crate::{
    ast::TypeSet,
    codegen::{
        file_patching::add_line_to_file,
        naming_pass::{NameMetadata, name},
        source_writer::SourceWriter,
    },
    composite, mapper,
    preprocessing::{BasicMetadata, ResolutionMetadata},
    typescript::types::emit_types,
};

mod idents;
mod types;

pub use idents::TypeScriptNamingRules;

fn convert_types(types: TypeSet<BasicMetadata>) -> TypeSet<TypeScriptMetadata> {
    name(types, TypeScriptNamingRules, AddName)
}

pub fn generate_types(types: TypeSet<BasicMetadata>, output: &Path, to_file: bool) -> Result<()> {
    let types = convert_types(types);

    if to_file {
        write_to_file(&types, output, false)
    } else {
        write_to_directory(&types, output)
    }
}

fn write_to_directory(types: &TypeSet<TypeScriptMetadata>, path: &Path) -> Result<()> {
    create_dir_all(path)?;
    let mod_name = &types.metadata.name;

    let type_path = path.join(format!("{}.ts", mod_name));
    write_to_file(types, &type_path, true)?;

    let index_path = path.join("index.ts");
    add_import_to_file(mod_name, &index_path)?;

    Ok(())
}

fn write_to_file(
    types: &TypeSet<TypeScriptMetadata>,
    path: &Path,
    must_be_new: bool,
) -> Result<()> {
    let file = if must_be_new {
        File::create_new(path)?
    } else {
        File::create(path)?
    };

    let mut writer = SourceWriter::new(BufWriter::new(file));
    emit_types(&mut writer, types)?;
    writer.into_inner().flush()?;

    Ok(())
}

fn add_import_to_file(module_name: &str, path: &Path) -> Result<()> {
    add_line_to_file(
        path,
        format_args!("import * as {module_name} from \"./{module_name}\";"),
    )
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
