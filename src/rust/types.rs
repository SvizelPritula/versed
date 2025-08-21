use std::io::{Result, Write};

use crate::{
    ast::{Enum, NamedType, PrimitiveType, Struct, Type, TypeSet},
    codegen::source_writer::SourceWriter,
    rust::RustMetadata,
};

pub fn emit_types(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
) -> Result<()> {
    for r#type in &types.types {
        if needs_type_alias(&r#type.r#type) {
            emit_type_alias(writer, types, r#type)?;
        }
    }
    writer.blank_line();

    for r#type in &types.types {
        emit_type_recursive(writer, types, &r#type.r#type)?;
    }

    Ok(())
}

fn emit_type_recursive(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
    r#type: &Type<RustMetadata>,
) -> Result<()> {
    match r#type {
        Type::Struct(r#struct) => {
            emit_struct(writer, types, r#struct)?;

            for field in &r#struct.fields {
                emit_type_recursive(writer, types, &field.r#type)?;
            }
        }
        Type::Enum(r#enum) => {
            emit_enum(writer, types, r#enum)?;

            for variant in &r#enum.variants {
                emit_type_recursive(writer, types, &variant.r#type)?;
            }
        }
        Type::List(list) => emit_type_recursive(writer, types, &list.r#type)?,
        Type::Primitive(_) => {}
        Type::Identifier(_) => {}
    }

    Ok(())
}

fn emit_struct(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
    r#struct: &Struct<RustMetadata>,
) -> Result<()> {
    writer.write("struct ")?;
    writer.write(&r#struct.metadata.name)?;
    writer.write_nl(" {")?;
    writer.indent();

    for field in &r#struct.fields {
        writer.write(&field.metadata.name)?;
        writer.write(": ")?;
        write_type_name(writer, types, &field.r#type, field.metadata.r#box)?;
        writer.write_nl(",")?;
    }

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

fn emit_enum(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
    r#enum: &Enum<RustMetadata>,
) -> Result<()> {
    writer.write("enum ")?;
    writer.write(&r#enum.metadata.name)?;
    writer.write_nl(" {")?;
    writer.indent();

    for variant in &r#enum.variants {
        writer.write(&variant.metadata.name)?;
        writer.write("(")?;
        write_type_name(writer, types, &variant.r#type, variant.metadata.r#box)?;
        writer.write_nl("),")?;
    }

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

fn emit_type_alias(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
    r#type: &NamedType<RustMetadata>,
) -> Result<()> {
    writer.write("type ")?;
    writer.write(type_name(&r#type.r#type))?;
    writer.write(" = ")?;
    write_type_name(writer, types, &r#type.r#type, r#type.metadata.r#box)?;
    writer.write_nl(";")?;

    Ok(())
}

fn write_type_name(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
    r#type: &Type<RustMetadata>,
    r#box: bool,
) -> Result<()> {
    if r#box {
        writer.write("::std::boxed::Box<")?;
    }

    match r#type {
        Type::Struct(r#struct) => writer.write(&r#struct.metadata.name)?,
        Type::Enum(r#enum) => writer.write(&r#enum.metadata.name)?,
        Type::List(list) => {
            writer.write("::std::vec::Vec<")?;
            write_type_name(writer, types, &list.r#type, false)?;
            writer.write(">")?;
        }
        Type::Primitive(primitive) => {
            writer.write(match primitive.r#type {
                PrimitiveType::String => "::std::string::String",
                PrimitiveType::Number => "::std::primitive::i64",
                PrimitiveType::Unit => "()",
            })?;
        }
        Type::Identifier(identifier) => writer.write(type_name(
            &types.types[identifier.metadata.resolution.index].r#type,
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
