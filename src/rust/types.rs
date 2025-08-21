use std::{
    collections::HashSet,
    io::{Result, Write},
};

use crate::{
    ast::{Enum, NamedType, PrimitiveType, Struct, Type, TypeSet},
    codegen::source_writer::SourceWriter,
    rust::RustMetadata,
};

const DERIVE: &str = "#[derive(Debug, Clone)]";

#[derive(Debug, Clone, Copy)]
struct Context<'a> {
    types: &'a TypeSet<RustMetadata>,
    used_type_names: &'a HashSet<&'a str>,
}

impl<'a> Context<'a> {
    fn rust_type<'b>(&'a self, name: &'b str, fallback: &'b str) -> &'b str {
        if self.used_type_names.contains(name) {
            fallback
        } else {
            name
        }
    }
}

pub fn emit_types(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
) -> Result<()> {
    let mut used_type_names = HashSet::new();

    for r#type in &types.types {
        used_type_names.insert(type_name(&r#type.r#type));
        add_all_rust_type_names(&r#type.r#type, &mut used_type_names);
    }

    let context = Context {
        types,
        used_type_names: &used_type_names,
    };

    for r#type in &types.types {
        if needs_type_alias(&r#type.r#type) {
            emit_type_alias(writer, context, r#type)?;
        }
    }
    writer.blank_line();

    for r#type in &types.types {
        emit_type_recursive(writer, context, &r#type.r#type)?;
    }

    Ok(())
}

fn emit_type_recursive(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    r#type: &Type<RustMetadata>,
) -> Result<()> {
    match r#type {
        Type::Struct(r#struct) => {
            emit_struct(writer, context, r#struct)?;

            for field in &r#struct.fields {
                emit_type_recursive(writer, context, &field.r#type)?;
            }
        }
        Type::Enum(r#enum) => {
            emit_enum(writer, context, r#enum)?;

            for variant in &r#enum.variants {
                emit_type_recursive(writer, context, &variant.r#type)?;
            }
        }
        Type::List(list) => emit_type_recursive(writer, context, &list.r#type)?,
        Type::Primitive(_) => {}
        Type::Identifier(_) => {}
    }

    Ok(())
}

fn emit_struct(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    r#struct: &Struct<RustMetadata>,
) -> Result<()> {
    writer.write_nl(DERIVE)?;
    writer.write("pub struct ")?;
    writer.write(&r#struct.metadata.name)?;
    writer.write_nl(" {")?;
    writer.indent();

    for field in &r#struct.fields {
        writer.write("pub ")?;
        writer.write(&field.metadata.name)?;
        writer.write(": ")?;
        write_type_name(writer, context, &field.r#type, field.metadata.r#box)?;
        writer.write_nl(",")?;
    }

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

fn emit_enum(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    r#enum: &Enum<RustMetadata>,
) -> Result<()> {
    writer.write_nl(DERIVE)?;
    writer.write("pub enum ")?;
    writer.write(&r#enum.metadata.name)?;
    writer.write_nl(" {")?;
    writer.indent();

    for variant in &r#enum.variants {
        writer.write(&variant.metadata.name)?;
        writer.write("(")?;
        write_type_name(writer, context, &variant.r#type, variant.metadata.r#box)?;
        writer.write_nl("),")?;
    }

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

fn emit_type_alias(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    r#type: &NamedType<RustMetadata>,
) -> Result<()> {
    writer.write("pub type ")?;
    writer.write(type_name(&r#type.r#type))?;
    writer.write(" = ")?;
    write_type_name(writer, context, &r#type.r#type, r#type.metadata.r#box)?;
    writer.write_nl(";")?;

    Ok(())
}

fn write_type_name(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    r#type: &Type<RustMetadata>,
    r#box: bool,
) -> Result<()> {
    if r#box {
        writer.write(context.rust_type("Box", "::std::boxed::Box"))?;
        writer.write("<")?;
    }

    match r#type {
        Type::Struct(r#struct) => writer.write(&r#struct.metadata.name)?,
        Type::Enum(r#enum) => writer.write(&r#enum.metadata.name)?,
        Type::List(list) => {
            writer.write(context.rust_type("Vec", "::std::vec::Vec"))?;
            writer.write("<")?;
            write_type_name(writer, context, &list.r#type, false)?;
            writer.write(">")?;
        }
        Type::Primitive(primitive) => {
            writer.write(match primitive.r#type {
                PrimitiveType::String => context.rust_type("String", "::std::string::String"),
                PrimitiveType::Number => context.rust_type("i64", "::std::primitive::i64"),
                PrimitiveType::Unit => "()",
            })?;
        }
        Type::Identifier(identifier) => writer.write(type_name(
            &context.types.types[identifier.metadata.resolution.index].r#type,
        ))?,
    }

    if r#box {
        writer.write(">")?;
    }

    Ok(())
}

fn needs_type_alias(r#type: &Type<RustMetadata>) -> bool {
    !matches!(r#type, Type::Struct(_) | Type::Enum(_))
}

fn type_name(r#type: &Type<RustMetadata>) -> &str {
    match r#type {
        Type::Struct(r#struct) => &r#struct.metadata.name,
        Type::Enum(r#enum) => &r#enum.metadata.name,
        Type::List(list) => &list.metadata.name,
        Type::Primitive(primitive) => &primitive.metadata.name,
        Type::Identifier(identifier) => &identifier.metadata.name,
    }
}

fn add_all_rust_type_names<'a>(r#type: &'a Type<RustMetadata>, set: &mut HashSet<&'a str>) {
    match r#type {
        Type::Struct(r#struct) => {
            set.insert(&r#struct.metadata.name);

            for field in &r#struct.fields {
                add_all_rust_type_names(&field.r#type, set);
            }
        }
        Type::Enum(r#enum) => {
            set.insert(&r#enum.metadata.name);

            for variant in &r#enum.variants {
                add_all_rust_type_names(&variant.r#type, set);
            }
        }
        Type::List(list) => add_all_rust_type_names(&list.r#type, set),
        Type::Primitive(_primitive) => {}
        Type::Identifier(_identifier) => {}
    }
}
