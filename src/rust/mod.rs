use std::{
    borrow::Cow,
    fs::{File, create_dir_all, exists},
    io::{BufWriter, Result, Write},
    path::Path,
};

use crate::{
    ast::{Migration, TypeSet},
    codegen::{
        file_patching::add_line_to_file,
        naming_pass::{NameMetadata, name},
        source_writer::SourceWriter,
    },
    composite, getter, mapper,
    migrations::pair_types,
    preprocessing::{BasicMetadata, ResolutionMetadata},
    rust::{
        idents::{RustMigrationSuffixNamingRules, RustNamingRules},
        migrations::emit_migration,
        recursive::{BoxMetadata, NewtypeMetadata, mark_boxes, mark_newtypes},
        types::emit_types,
    },
    typescript::TypeScriptNamingRules,
};

mod codegen;
mod idents;
mod migrations;
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

impl Default for RustOptions {
    fn default() -> Self {
        Self::new(false, vec![])
    }
}

fn convert_types(types: TypeSet<BasicMetadata>) -> TypeSet<RustMetadata> {
    let types = name(types, RustNamingRules, AddRustName);
    let mut types = name(types, TypeScriptNamingRules, AddTypeScriptName);

    mark_boxes(&mut types);
    mark_newtypes(&mut types);

    types
}

fn convert_types_for_migration(types: TypeSet<BasicMetadata>) -> TypeSet<RustMigrationMetadata> {
    let types = convert_types(types);
    name(types, RustMigrationSuffixNamingRules, AddMigrationName)
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

pub fn generate_migration(migration: Migration<BasicMetadata>, output: &Path) -> Result<()> {
    const MIGRATION_MOD: &str = "migrations";

    let migration = migration.map(convert_types_for_migration);
    let pairs = pair_types(&migration);

    let migrations_dir = output.join(MIGRATION_MOD);
    let is_directory_new = !exists(&migrations_dir)?;
    create_dir_all(&migrations_dir)?;

    let mod_name = &migration.new.metadata.base.name;
    let type_path = migrations_dir.join(format!("{mod_name}.rs"));
    let file = File::create_new(type_path)?;

    let mut writer = SourceWriter::new(BufWriter::new(file));
    emit_migration(&mut writer, &migration, &pairs, "upgrade")?;
    writer.into_inner().flush()?;

    let mod_path = migrations_dir.join("mod.rs");
    add_mod_to_file(mod_name, &mod_path)?;

    if is_directory_new {
        add_mod_to_file(MIGRATION_MOD, &output.join("mod.rs"))?;
    }

    Ok(())
}

fn write_to_directory(
    types: &TypeSet<RustMetadata>,
    options: &RustOptions,
    path: &Path,
) -> Result<()> {
    create_dir_all(path)?;
    let mod_name = &types.metadata.name;

    let type_path = path.join(format!("{mod_name}.rs"));
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
    add_line_to_file(path, format_args!("pub mod {mod_name};"))
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

composite! {
    struct (RustMigrationInfo, RustMigrationMetadata) {
        base: RustMetadata | B,
        migration_name: NameMetadata | M
    }
}

mapper! {
    fn AddMigrationName(base: RustMetadata, migration_name: NameMetadata) -> RustMigrationMetadata {
        RustMigrationInfo {
            base,
            migration_name,
        }
    }
}

getter! {
    fn GetBase(metadata: RustMigrationMetadata) -> RustMetadata {
        &metadata.base
    }
}
