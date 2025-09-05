use std::{
    borrow::Cow,
    fs::{File, create_dir_all},
    io::{BufWriter, Result, Write},
    path::Path,
};

use crate::{
    ast::TypeSet,
    codegen::{
        file_patching::add_line_to_file,
        naming_pass::{name, NameMetadata},
        source_writer::SourceWriter,
    },
    composite, mapper,
    preprocessing::{BasicMetadata, ResolutionMetadata},
    rust::{
        idents::RustNamingRules,
        recursive::{mark_boxes, mark_newtypes, BoxMetadata, NewtypeMetadata},
        types::emit_types,
    }, typescript::TypeScriptNamingRules,
};

mod idents;
mod recursive;
mod types;

#[derive(Debug, Clone)]
pub struct RustOptions {
    serde: bool,
    derives: Vec<Cow<'static, str>>,
}

impl RustOptions {
    pub fn new(serde: bool, extra_derives: Vec<String>) -> Self {
        let mut derives = vec![Cow::Borrowed("Debug"), Cow::Borrowed("Clone")];

        if serde {
            derives.extend([Cow::Borrowed("Serialize"), Cow::Borrowed("Deserialize")]);
        }

        derives.extend(extra_derives.into_iter().map(Cow::Owned));

        Self { serde, derives }
    }
}

fn convert_types(types: TypeSet<BasicMetadata>) -> TypeSet<RustMetadata> {
    let types = name(types, RustNamingRules, AddRustName);
    let mut types = name(types, TypeScriptNamingRules, AddTypeScriptName);

    mark_boxes(&mut types);
    mark_newtypes(&mut types);

    types
}

pub fn generate_types(
    types: TypeSet<BasicMetadata>,
    options: &RustOptions,
    output: &Path,
    to_file: bool,
) -> Result<()> {
    let types = convert_types(types);

    if to_file {
        write_to_file(&types, options, output, false)
    } else {
        write_to_directory(&types, options, output)
    }
}

fn write_to_directory(
    types: &TypeSet<RustMetadata>,
    options: &RustOptions,
    path: &Path,
) -> Result<()> {
    create_dir_all(path)?;
    let mod_name = &types.metadata.name;

    let type_path = path.join(format!("{}.rs", mod_name));
    write_to_file(types, options, &type_path, true)?;

    let mod_path = path.join("mod.rs");
    add_mod_to_file(mod_name, &mod_path)?;

    Ok(())
}

fn write_to_file(
    types: &TypeSet<RustMetadata>,
    options: &RustOptions,
    path: &Path,
    must_be_new: bool,
) -> Result<()> {
    let file = if must_be_new {
        File::create_new(path)?
    } else {
        File::create(path)?
    };

    let mut writer = SourceWriter::new(BufWriter::new(file));
    emit_types(&mut writer, types, options)?;
    writer.into_inner().flush()?;

    Ok(())
}

fn add_mod_to_file(mod_name: &str, path: &Path) -> Result<()> {
    add_line_to_file(path, format_args!("mod {mod_name};"))
}

composite! {
    struct (RustNamingPassInfo, RustNamingPassMetadata) {
        name: NameMetadata | N,
        resolution: ResolutionMetadata | R
    }
}

mapper! {
    fn AddRustName(basic: BasicMetadata, name: NameMetadata) -> RustNamingPassMetadata {
        RustNamingPassInfo {
            name,
            resolution: basic.resolution,
        }
    }
}

composite! {
    struct (RustInfo, RustMetadata) {
        name: NameMetadata | N,
        resolution: ResolutionMetadata | R,
        serde_name: NameMetadata | S,
        r#box: BoxMetadata | B,
        newtype: NewtypeMetadata | NT
    }
}

mapper! {
    fn AddTypeScriptName(first: RustNamingPassMetadata, name: NameMetadata) -> RustMetadata {
        RustInfo {
            name: first.name,
            resolution: first.resolution,
            serde_name: name,

            // Either false or ():
            r#box: Default::default(),
            newtype: Default::default(),
        }
    }
}
