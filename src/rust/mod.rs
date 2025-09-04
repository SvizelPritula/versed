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
    rust::{
        idents::RustNamingRules,
        recursive::{BoxMetadata, NewtypeMetadata, mark_boxes, mark_newtypes},
        types::emit_types,
    },
};

mod idents;
mod recursive;
mod types;

fn convert_types(types: TypeSet<BasicMetadata>) -> TypeSet<RustMetadata> {
    let mut types = name(types, RustNamingRules, AddName);
    mark_boxes(&mut types);
    mark_newtypes(&mut types);
    types
}

pub fn generate_types(types: TypeSet<BasicMetadata>, output: &Path, to_file: bool) -> Result<()> {
    let types = convert_types(types);

    if to_file {
        write_to_file(&types, output, false)
    } else {
        write_to_directory(&types, output)
    }
}

fn write_to_directory(types: &TypeSet<RustMetadata>, path: &Path) -> Result<()> {
    create_dir_all(path)?;
    let mod_name = &types.metadata.name;

    let type_path = path.join(format!("{}.rs", mod_name));
    write_to_file(types, &type_path, true)?;

    let mod_path = path.join("mod.rs");
    add_mod_to_file(mod_name, &mod_path)?;

    Ok(())
}

fn write_to_file(types: &TypeSet<RustMetadata>, path: &Path, must_be_new: bool) -> Result<()> {
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

fn add_mod_to_file(mod_name: &str, path: &Path) -> Result<()> {
    add_line_to_file(path, format_args!("mod {mod_name};"))
}

composite! {
    struct (RustInfo, RustMetadata) {
        name: NameMetadata | N,
        resolution: ResolutionMetadata | R,
        r#box: BoxMetadata | B,
        newtype: NewtypeMetadata | NT
    }
}

mapper! {
    fn AddName(basic: BasicMetadata, name: NameMetadata) -> RustMetadata {
        RustInfo {
            name,
            resolution: basic.resolution,
            // Either false or ():
            r#box: Default::default(),
            newtype: Default::default(),
        }
    }
}
