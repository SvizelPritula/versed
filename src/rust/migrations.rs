use std::io::{Result, Write};

use crate::{
    ast::{Migration, Primitive, Type, TypeType},
    codegen::source_writer::SourceWriter,
    metadata::Metadata,
    migrations::TypePair,
    rust::{
        GetBase, RustMigrationMetadata, RustOptions,
        codegen::{self, all_rust_type_names},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    old: codegen::Context<'a, RustMigrationMetadata>,
    new: codegen::Context<'a, RustMigrationMetadata>,
    direction: &'static str,
}

pub fn emit_migration(
    writer: &mut SourceWriter<impl Write>,
    migration: &Migration<RustMigrationMetadata>,
    pairs: &[TypePair<RustMigrationMetadata>],
    direction: &'static str,
) -> Result<()> {
    let options = RustOptions::default();

    let context = Context {
        old: codegen::Context {
            types: &migration.old,
            options: &options,
            used_type_names: &all_rust_type_names(&migration.old, GetBase),
        },
        new: codegen::Context {
            types: &migration.new,
            options: &options,
            used_type_names: &all_rust_type_names(&migration.new, GetBase),
        },
        direction,
    };

    writer.write_fmt_nl(format_args!("pub mod {direction} {{"))?;
    writer.indent();

    writer.write_fmt_nl(format_args!(
        "use super::super::super::{{{}, {}}};",
        migration.old.metadata.base.name, migration.new.metadata.base.name
    ))?;
    writer.blank_line();

    for &pair in pairs {
        emit_function(writer, context, pair)?;
    }

    writer.dedent();
    writer.write_nl("}")?;

    Ok(())
}

fn emit_function(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    pair: TypePair<RustMigrationMetadata>,
) -> Result<()> {
    writer.write_fmt(format_args!(
        "pub fn {}_{}(",
        context.direction, pair.new.metadata.migration_name
    ))?;

    writer.write_fmt(format_args!("{}: ", pair.old.metadata.migration_name))?;
    write_type_name(writer, context.old, pair.old, false)?;

    writer.write(") -> ")?;
    write_type_name(writer, context.new, pair.new, false)?;
    writer.write_nl(" {")?;
    writer.indent();

    emit_body(writer, context, pair)?;

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

fn emit_body(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    pair: TypePair<RustMigrationMetadata>,
) -> Result<()> {
    let old_metadata = &pair.old.metadata;
    let new_metadata = &pair.new.metadata;

    match (&pair.old.r#type, &pair.new.r#type) {
        (TypeType::Primitive(old), TypeType::Primitive(new)) => emit_primitive(
            writer,
            context,
            GenericPair::new(old, new, old_metadata, new_metadata),
        )?,
        (_old, _new) => emit_todo(writer)?,
    }

    Ok(())
}

struct WithMetadata<'a, T> {
    r#type: &'a T,
    metadata: &'a <RustMigrationMetadata as Metadata>::Type,
}

struct GenericPair<'a, T> {
    old: WithMetadata<'a, T>,
    new: WithMetadata<'a, T>,
}

impl<'a, T> GenericPair<'a, T> {
    fn new(
        old: &'a T,
        new: &'a T,
        old_metadata: &'a <RustMigrationMetadata as Metadata>::Type,
        new_metadata: &'a <RustMigrationMetadata as Metadata>::Type,
    ) -> Self {
        Self {
            old: WithMetadata {
                r#type: old,
                metadata: old_metadata,
            },
            new: WithMetadata {
                r#type: new,
                metadata: new_metadata,
            },
        }
    }
}

fn emit_primitive(
    writer: &mut SourceWriter<impl Write>,
    _context: Context,
    GenericPair { old, new }: GenericPair<Primitive<RustMigrationMetadata>>,
) -> Result<()> {
    eprintln!("{:?} {:?}", old.r#type.r#type, new.r#type.r#type);
    if old.r#type.r#type == new.r#type.r#type {
        writer.write_nl(&old.metadata.migration_name)
    } else {
        emit_todo(writer)
    }
}

fn emit_todo(writer: &mut SourceWriter<impl Write>) -> Result<()> {
    writer.write_nl("todo!()")
}

fn write_type_name(
    writer: &mut SourceWriter<impl Write>,
    context: codegen::Context<RustMigrationMetadata>,
    r#type: &Type<RustMigrationMetadata>,
    r#box: bool,
) -> Result<()> {
    codegen::write_type_name(
        writer,
        context,
        r#type,
        r#box,
        format_args!("{}::", context.types.metadata.base.name),
        GetBase,
    )
}
