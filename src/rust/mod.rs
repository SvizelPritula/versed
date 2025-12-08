use std::{
    borrow::Cow,
    fs::{File, create_dir_all, exists},
    io::{BufWriter, Write},
    path::Path,
};

use crate::{
    ast::{Migration, TypeSet},
    codegen::{
        file_patching::add_line_to_file,
        naming_pass::{NameMetadata, name},
        source_writer::SourceWriter,
    },
    composite,
    error::{Error, ResultExt},
    getter, mapper,
    migrations::{TypePair, pair_types},
    preprocessing::{BasicMetadata, ResolutionMetadata},
    rust::{
        idents::{RustMigrationSuffixNamingRules, RustNamingRules},
        migrations::emit_migrations,
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
) -> Result<(), Error> {
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
) -> Result<(), Error> {
    create_dir_all(path).with_path(path)?;
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
) -> Result<(), Error> {
    let file = if must_be_new {
        File::create_new(path).with_path(path)?
    } else {
        File::create(path).with_path(path)?
    };

    let mut writer = SourceWriter::new(BufWriter::new(file));
    emit_types(&mut writer, types, options).with_path(path)?;
    writer.into_inner().flush().with_path(path)?;

    Ok(())
}

pub fn generate_migration(
    migration: Migration<BasicMetadata>,
    output: &Path,
    to_file: bool,
) -> Result<(), Error> {
    let migration = migration.map(convert_types_for_migration);
    let pairs = pair_types(&migration);

    if to_file {
        write_migration_to_file(&migration, &pairs, output, false)
    } else {
        write_migration_to_directory(&migration, &pairs, output)
    }
}

fn write_migration_to_directory(
    migration: &Migration<RustMigrationMetadata>,
    pairs: &[TypePair<RustMigrationMetadata>],
    path: &Path,
) -> Result<(), Error> {
    const MIGRATION_MOD: &str = "migrations";

    let migrations_dir = path.join(MIGRATION_MOD);
    let is_directory_new = !exists(&migrations_dir).with_path(&migrations_dir)?;
    create_dir_all(&migrations_dir).with_path(&migrations_dir)?;

    let mod_name = &migration.new.metadata.base.name;
    let type_path = migrations_dir.join(format!("{mod_name}.rs"));

    write_migration_to_file(migration, pairs, &type_path, true)?;

    let mod_path = migrations_dir.join("mod.rs");
    add_mod_to_file(mod_name, &mod_path)?;

    if is_directory_new {
        add_mod_to_file(MIGRATION_MOD, &path.join("mod.rs"))?;
    }

    Ok(())
}

fn write_migration_to_file(
    migration: &Migration<RustMigrationMetadata>,
    pairs: &[TypePair<RustMigrationMetadata>],
    path: &Path,
    must_be_new: bool,
) -> Result<(), Error> {
    let file = if must_be_new {
        File::create_new(path).with_path(path)?
    } else {
        File::create(path).with_path(path)?
    };

    let mut writer = SourceWriter::new(BufWriter::new(file));
    emit_migrations(&mut writer, migration, pairs).with_path(path)?;
    writer.into_inner().flush().with_path(path)?;

    Ok(())
}

fn add_mod_to_file(mod_name: &str, path: &Path) -> Result<(), Error> {
    add_line_to_file(path, format_args!("pub mod {mod_name};")).with_path(path)
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
