use std::io::{Result, Write};

use crate::{
    ast::{Migration, Type},
    codegen::source_writer::SourceWriter,
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
        emit_migration_for_pair(writer, context, pair)?;
    }

    writer.dedent();
    writer.write_nl("}")?;

    Ok(())
}

fn emit_migration_for_pair(
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

    writer.write_nl("todo!()")?;

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
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
