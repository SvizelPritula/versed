use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Arguments, Display},
    io::{Result, Write},
};

use crate::{
    ast::{
        Enum, Field, Identifier, List, Migration, Primitive, Struct, Type, TypeSet, TypeType,
        Variant,
    },
    codegen::{idents::IdentRules, source_writer::SourceWriter},
    migrations::TypePair,
    rust::{
        GetBase, RustMigrationMetadata,
        codegen::{self, NamingContext},
        idents::RustIdentRules,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    old: NamingContext<'a, RustMigrationMetadata>,
    new: NamingContext<'a, RustMigrationMetadata>,
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
        let name = &new.metadata.migration_name;
        let name = name
            .strip_prefix(RustIdentRules.reserved_prefix())
            .unwrap_or(name);
        FunctionName(self.direction, name)
    }
}

struct FunctionName<'a>(&'a str, &'a str);

impl Display for FunctionName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FunctionName(prefix, suffix) = self;
        write!(f, "{prefix}_{suffix}")
    }
}

struct WithMetadata<'a, T> {
    r#type: &'a T,
    full: &'a Type<RustMigrationMetadata>,
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

const TODO: &str = "todo!()";

pub fn emit_migrations(
    writer: &mut SourceWriter<impl Write>,
    migration: &Migration<RustMigrationMetadata>,
    pairs: &[TypePair<RustMigrationMetadata>],
) -> Result<()> {
    let swaped_pairs: Vec<TypePair<RustMigrationMetadata>> = pairs
        .iter()
        .map(|TypePair { old, new }| TypePair { old: new, new: old })
        .collect();

    emit_directional_migration(writer, &migration.old, &migration.new, pairs, "upgrade")?;
    writer.blank_line();
    emit_directional_migration(
        writer,
        &migration.new,
        &migration.old,
        &swaped_pairs,
        "downgrade",
    )?;

    Ok(())
}

fn emit_directional_migration(
    writer: &mut SourceWriter<impl Write>,
    old: &TypeSet<RustMigrationMetadata>,
    new: &TypeSet<RustMigrationMetadata>,
    pairs: &[TypePair<RustMigrationMetadata>],
    direction: &'static str,
) -> Result<()> {
    let context = Context {
        old: codegen::NamingContext {
            types: old,
            used_type_names: &HashSet::new(),
        },
        new: codegen::NamingContext {
            types: new,
            used_type_names: &HashSet::new(),
        },
        direction,
    };

    writer.write_fmt_nl(format_args!("pub mod {direction} {{"))?;
    writer.indent();

    writer.write_fmt_nl(format_args!(
        "use super::super::super::{{{}, {}}};",
        old.metadata.base.name, new.metadata.base.name
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

    let expr = pair.old.metadata.migration_name.as_str();
    writer.write_fmt(format_args!("{expr}: "))?;
    write_type_name(writer, context.old, pair.old)?;

    writer.write(") -> ")?;
    write_type_name(writer, context.new, pair.new)?;
    writer.write_nl(" {")?;
    writer.indent();

    emit_body(writer, context, pair, format_args!("{expr}"))?;

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

fn emit_body(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    pair: TypePair<RustMigrationMetadata>,
    expr: fmt::Arguments,
) -> Result<()> {
    if pair.new.metadata.base.newtype {
        write_type_name(writer, context.new, pair.new)?;
        writer.write_nl("(")?;
        writer.indent();
    }

    if pair.new.metadata.base.r#box {
        writer.write_nl("Box::new(")?;
        writer.indent();
    }

    let expr = if pair.old.metadata.base.newtype {
        format_args!("{expr}.0")
    } else {
        expr
    };

    let expr = if pair.old.metadata.base.r#box {
        format_args!("(*{expr})")
    } else {
        expr
    };

    match (&pair.old.r#type, &pair.new.r#type) {
        (TypeType::Struct(old), TypeType::Struct(new)) => emit_struct(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
            expr,
        )?,
        (TypeType::Enum(old), TypeType::Enum(new)) => emit_enum(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
            expr,
        )?,
        (TypeType::List(old), TypeType::List(new)) => emit_list(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
            expr,
        )?,
        (TypeType::Primitive(old), TypeType::Primitive(new)) => emit_primitive(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
            expr,
        )?,
        (TypeType::Identifier(old), TypeType::Identifier(new)) => emit_identifier(
            writer,
            context,
            GenericPair::new(old, new, pair.old, pair.new),
            expr,
        )?,
        (_old, _new) => write_todo(writer)?,
    }

    if pair.new.metadata.base.r#box {
        writer.dedent();
        writer.write_nl(")")?;
    }

    if pair.new.metadata.base.newtype {
        writer.dedent();
        writer.write_nl(")")?;
    }

    Ok(())
}

fn emit_struct(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    GenericPair { old, new }: GenericPair<Struct<RustMigrationMetadata>>,
    expr: fmt::Arguments,
) -> Result<()> {
    write_type_name(writer, context.new, new.full)?;
    writer.write_nl(" {")?;
    writer.indent();

    let by_type_number: HashMap<u64, &Field<RustMigrationMetadata>> = old
        .r#type
        .fields
        .iter()
        .filter_map(|field| field.r#type.number.map(|number| (number, field)))
        .collect();

    for field in &new.r#type.fields {
        writer.write_fmt(format_args!("{}: ", field.metadata.base.name))?;

        if let Some(&old_field) = field.r#type.number.and_then(|n| by_type_number.get(&n)) {
            let field_name = &old_field.metadata.base.name;

            write_upgrade(
                writer,
                context,
                format_args!("{expr}.{field_name}"),
                &field.r#type,
            )?;
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
    expr: fmt::Arguments,
) -> Result<()> {
    writer.write_fmt_nl(format_args!("match {expr} {{"))?;
    writer.indent();

    let by_type_number: HashMap<u64, &Variant<RustMigrationMetadata>> = new
        .r#type
        .variants
        .iter()
        .filter_map(|variant| variant.r#type.number.map(|number| (number, variant)))
        .collect();

    for variant in &old.r#type.variants {
        let binding = &variant.metadata.migration_name;

        write_type_name(writer, context.old, old.full)?;
        writer.write_fmt(format_args!(
            "::{}({binding}) => ",
            variant.metadata.base.name
        ))?;

        if let Some(&new_variant) = variant.r#type.number.and_then(|n| by_type_number.get(&n)) {
            let variant_name = &new_variant.metadata.base.name;

            write_type_name(writer, context.new, new.full)?;
            writer.write_fmt(format_args!("::{variant_name}("))?;
            write_upgrade(
                writer,
                context,
                format_args!("{binding}"),
                &new_variant.r#type,
            )?;
            writer.write(")")?;
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
    expr: fmt::Arguments,
) -> Result<()> {
    if let Some(func) = context.function_between(&old.r#type.r#type, &new.r#type.r#type) {
        writer.write_fmt_nl(format_args!("{expr}.into_iter().map({func}).collect()"))
    } else {
        write_todo(writer)
    }
}

fn emit_primitive(
    writer: &mut SourceWriter<impl Write>,
    _context: Context,
    GenericPair { old, new }: GenericPair<Primitive<RustMigrationMetadata>>,
    expr: fmt::Arguments,
) -> Result<()> {
    if old.r#type.r#type == new.r#type.r#type {
        writer.write_fmt_nl(expr)
    } else {
        write_todo(writer)
    }
}

fn emit_identifier(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    GenericPair { old, new }: GenericPair<Identifier<RustMigrationMetadata>>,
    expr: fmt::Arguments,
) -> Result<()> {
    let old_ref = &context.old.types.types[old.r#type.metadata.base.resolution];
    let new_ref = &context.new.types.types[new.r#type.metadata.base.resolution];

    if let Some(func) = context.function_between(&old_ref.r#type, &new_ref.r#type) {
        writer.write_fmt_nl(format_args!("{func}({expr})"))
    } else {
        write_todo(writer)
    }
}

fn write_upgrade(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    binding: Arguments,
    new: &Type<RustMigrationMetadata>,
) -> Result<()> {
    let func = context.function_to(new);

    writer.write_fmt(format_args!("{func}("))?;
    writer.write_fmt(binding)?;
    writer.write(")")?;

    Ok(())
}

fn write_todo(writer: &mut SourceWriter<impl Write>) -> Result<()> {
    writer.write_nl(TODO)
}

fn write_type_name(
    writer: &mut SourceWriter<impl Write>,
    context: codegen::NamingContext<RustMigrationMetadata>,
    r#type: &Type<RustMigrationMetadata>,
) -> Result<()> {
    codegen::write_type_name(
        writer,
        context,
        r#type,
        format_args!("{}::", context.types.metadata.base.name),
        false,
        GetBase,
    )
}
