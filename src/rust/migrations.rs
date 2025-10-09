use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
    io::{Result, Write},
};

use crate::{
    ast::{Enum, Field, Identifier, List, Migration, Primitive, Struct, Type, TypeType, Variant},
    codegen::source_writer::SourceWriter,
    metadata::Metadata,
    migrations::TypePair,
    rust::{GetBase, RustMigrationMetadata, RustOptions, codegen},
};

#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    old: codegen::Context<'a, RustMigrationMetadata>,
    new: codegen::Context<'a, RustMigrationMetadata>,
    direction: &'static str,
}

impl<'a> Context<'a> {
    fn function_between(
        &'a self,
        old: &'a Type<RustMigrationMetadata>,
        new: &'a Type<RustMigrationMetadata>,
    ) -> Option<impl Display> {
        if old.number.zip(new.number).is_some_and(|(o, n)| o == n) {
            Some(self.function_to(new))
        } else {
            None
        }
    }

    fn function_to(&'a self, new: &'a Type<RustMigrationMetadata>) -> impl Display {
        FunctionName(self.direction, &new.metadata.migration_name)
    }
}

const TODO: &str = "todo!()";

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
            used_type_names: &HashSet::new(),
        },
        new: codegen::Context {
            types: &migration.new,
            options: &options,
            used_type_names: &HashSet::new(),
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
    writer.write_fmt(format_args!("pub fn {}(", context.function_to(pair.new)))?;

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
    match (&pair.old.r#type, &pair.new.r#type) {
        (TypeType::Struct(old), TypeType::Struct(new)) => emit_struct(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
        )?,
        (TypeType::Enum(old), TypeType::Enum(new)) => emit_enum(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
        )?,
        (TypeType::List(old), TypeType::List(new)) => emit_list(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
        )?,
        (TypeType::Primitive(old), TypeType::Primitive(new)) => emit_primitive(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
        )?,
        (TypeType::Identifier(old), TypeType::Identifier(new)) => emit_identifier(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
        )?,
        (_old, _new) => emit_todo(writer)?,
    }

    Ok(())
}

struct WithMetadata<'a, T> {
    r#type: &'a T,
    full: &'a Type<RustMigrationMetadata>,
}

impl<T> WithMetadata<'_, T> {
    pub fn metadata(&self) -> &<RustMigrationMetadata as Metadata>::Type {
        &self.full.metadata
    }
}

struct GenericPair<'a, T> {
    old: WithMetadata<'a, T>,
    new: WithMetadata<'a, T>,
}

impl<'a, T> GenericPair<'a, T> {
    fn new(
        old: &'a T,
        new: &'a T,
        old_full: &'a Type<RustMigrationMetadata>,
        new_full: &'a Type<RustMigrationMetadata>,
    ) -> Self {
        Self {
            old: WithMetadata {
                r#type: old,
                full: old_full,
            },
            new: WithMetadata {
                r#type: new,
                full: new_full,
            },
        }
    }
}

fn emit_struct(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    GenericPair { old, new }: GenericPair<Struct<RustMigrationMetadata>>,
) -> Result<()> {
    let arg = &old.metadata().migration_name;

    write_type_name(writer, context.new, new.full, false)?;
    writer.write_nl(" {")?;
    writer.indent();

    let by_type_number: HashMap<u64, &Field<RustMigrationMetadata>> = old
        .r#type
        .fields
        .iter()
        .flat_map(|field| field.r#type.number.map(|number| (number, field)))
        .collect();

    for field in &new.r#type.fields {
        writer.write_fmt(format_args!("{}: ", field.metadata.base.name))?;

        if let Some(&old_field) = field.r#type.number.and_then(|n| by_type_number.get(&n)) {
            let func = context.function_to(&field.r#type);
            let field_name = &old_field.metadata.base.name;
            writer.write_fmt(format_args!("{func}({arg}.{field_name})"))?;
        } else {
            writer.write(TODO)?;
        }

        writer.write_nl(",")?;
    }

    writer.dedent();
    writer.write_nl("}")?;

    Ok(())
}

fn emit_enum(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    GenericPair { old, new }: GenericPair<Enum<RustMigrationMetadata>>,
) -> Result<()> {
    let arg = &old.metadata().migration_name;

    writer.write_fmt_nl(format_args!("match {arg} {{"))?;
    writer.indent();

    let by_type_number: HashMap<u64, &Variant<RustMigrationMetadata>> = new
        .r#type
        .variants
        .iter()
        .flat_map(|variant| variant.r#type.number.map(|number| (number, variant)))
        .collect();

    for variant in &old.r#type.variants {
        let binding = &variant.metadata.migration_name;

        write_type_name(writer, context.old, old.full, false)?;
        writer.write_fmt(format_args!(
            "::{}({binding}) => ",
            variant.metadata.base.name
        ))?;

        if let Some(&new_variant) = variant.r#type.number.and_then(|n| by_type_number.get(&n)) {
            let func = context.function_to(&new_variant.r#type);
            let variant_name = &new_variant.metadata.base.name;

            write_type_name(writer, context.new, new.full, false)?;
            writer.write_fmt(format_args!("::{variant_name}({func}({binding}))"))?;
        } else {
            writer.write(TODO)?;
        }

        writer.write_nl(",")?;
    }

    writer.dedent();
    writer.write_nl("}")?;

    Ok(())
}

fn emit_list(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    GenericPair { old, new }: GenericPair<List<RustMigrationMetadata>>,
) -> Result<()> {
    let arg = &old.metadata().migration_name;

    if let Some(func) = context.function_between(&old.r#type.r#type, &new.r#type.r#type) {
        writer.write_fmt_nl(format_args!("{arg}.into_iter().map({func}).collect()"))
    } else {
        emit_todo(writer)
    }
}

fn emit_primitive(
    writer: &mut SourceWriter<impl Write>,
    _context: Context,
    GenericPair { old, new }: GenericPair<Primitive<RustMigrationMetadata>>,
) -> Result<()> {
    if old.r#type.r#type == new.r#type.r#type {
        writer.write_nl(&old.metadata().migration_name)
    } else {
        emit_todo(writer)
    }
}

fn emit_identifier(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    GenericPair { old, new }: GenericPair<Identifier<RustMigrationMetadata>>,
) -> Result<()> {
    let old_ref = &context.old.types.types[old.r#type.metadata.base.resolution];
    let new_ref = &context.new.types.types[new.r#type.metadata.base.resolution];
    let arg = &old.metadata().migration_name;

    if let Some(func) = context.function_between(&old_ref.r#type, &new_ref.r#type) {
        writer.write_fmt_nl(format_args!("{func}({arg})"))
    } else {
        emit_todo(writer)
    }
}

fn emit_todo(writer: &mut SourceWriter<impl Write>) -> Result<()> {
    writer.write_nl(TODO)
}

struct FunctionName<'a>(&'a str, &'a str);

impl Display for FunctionName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FunctionName(prefix, suffix) = self;
        write!(f, "{prefix}_{suffix}")
    }
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
