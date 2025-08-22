use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::{BufWriter, Read, Result, Seek, SeekFrom, Write},
    path::Path,
};

use crate::{
    ast::TypeSet,
    codegen::{
        naming_pass::{NameMetadata, name},
        source_writer::SourceWriter,
    },
    composite, mapper,
    name_resolution::ResolutionMetadata,
    rust::{
        idents::RustNamingRules,
        recursive::{BoxMetadata, NewtypeMetadata, mark_boxes, mark_newtypes},
        types::emit_types,
    },
};

mod idents;
mod recursive;
mod types;

fn convert_types(types: TypeSet<ResolutionMetadata>) -> TypeSet<RustMetadata> {
    let mut types = name(types, RustNamingRules, AddName);
    mark_boxes(&mut types);
    mark_newtypes(&mut types);
    types
}

pub fn generate_types(
    types: TypeSet<ResolutionMetadata>,
    output: &Path,
    to_file: bool,
) -> Result<()> {
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
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(false)
        .open(path)?;

    let pos = file.seek(SeekFrom::End(0))?;

    let must_add_lf = if pos > 0 {
        file.seek_relative(-1)?;

        let mut byte_buf = [0];
        file.read_exact(&mut byte_buf)?;
        let [byte] = byte_buf;

        byte != b'\n'
    } else {
        false
    };

    let mut file = BufWriter::new(file);

    if must_add_lf {
        file.write_all(b"\n")?;
    }

    writeln!(file, "mod {mod_name};")?;
    file.flush()?;

    Ok(())
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
    fn AddName(resolution: ResolutionMetadata, name: NameMetadata) -> RustMetadata {
        RustInfo {
            name,
            resolution,
            // Either false or ():
            r#box: Default::default(),
            newtype: Default::default(),
        }
    }
}
