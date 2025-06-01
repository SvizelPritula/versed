use std::io::{Error, Result, Write};

use crate::{
    ast::{Enum, NamedType, Primitive, Struct, Type, TypeSet},
    source_writer::SourceWriter,
};

use super::CSharpMetadata;

pub fn emit(types: &TypeSet<CSharpMetadata>, writer: &mut SourceWriter<impl Write>) -> Result<()> {
    for r#type in &types.types {
        emit_alias_if_needed(r#type, types, writer)?;
    }

    for r#type in &types.types {
        emit_declaration_if_needed(&r#type.r#type, types, writer)?;
    }

    Ok(())
}

fn emit_declaration_if_needed(
    r#type: &Type<CSharpMetadata>,
    types: &TypeSet<CSharpMetadata>,
    writer: &mut SourceWriter<impl Write>,
) -> Result<()> {
    match r#type {
        Type::Struct(r#struct) => emit_struct(r#struct, types, writer),
        Type::Enum(r#enum) => emit_enum(r#enum, types, writer),
        Type::Versioned(versioned) => emit_declaration_if_needed(&versioned.r#type, types, writer),
        Type::List(inner) => emit_declaration_if_needed(inner, types, writer),
        Type::Primitive(_) => Ok(()),
        Type::Identifier(_) => Ok(()),
    }
}

fn emit_struct(
    r#struct: &Struct<CSharpMetadata>,
    types: &TypeSet<CSharpMetadata>,
    writer: &mut SourceWriter<impl Write>,
) -> Result<()> {
    writer.blank_line();
    writer.write_fmt_nl(format_args!("public class {} {{", r#struct.metadata.ident))?;
    writer.indent();

    for field in &r#struct.fields {
        writer.write("public required ")?;
        write_type_name(&field.r#type, types, writer)?;
        writer.write(" ")?;
        writer.write(&field.metadata.ident)?;
        writer.write_nl(" { get; set; }")?;
    }

    for field in &r#struct.fields {
        emit_declaration_if_needed(&field.r#type, types, writer)?;
    }

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

fn emit_enum(
    r#enum: &Enum<CSharpMetadata>,
    types: &TypeSet<CSharpMetadata>,
    writer: &mut SourceWriter<impl Write>,
) -> Result<()> {
    writer.blank_line();
    writer.write_fmt_nl(format_args!(
        "public abstract class {} {{",
        r#enum.metadata.ident
    ))?;
    writer.indent();

    for variant in &r#enum.variants {
        writer.blank_line();
        writer.write_fmt_nl(format_args!(
            "public class {} : {} {{",
            variant.metadata.ident, r#enum.metadata.ident
        ))?;
        writer.indent();

        writer.write("public required ")?;
        write_type_name(&variant.r#type, types, writer)?;
        writer.write_nl(" Value { get; set; }")?;

        emit_declaration_if_needed(&variant.r#type, types, writer)?;

        writer.dedent();
        writer.write_nl("}")?;
        writer.blank_line();
    }

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

fn write_type_name(
    r#type: &Type<CSharpMetadata>,
    types: &TypeSet<CSharpMetadata>,
    writer: &mut SourceWriter<impl Write>,
) -> Result<()> {
    write_type_name_advanced(r#type, types, writer, false)
}

fn write_type_name_advanced(
    r#type: &Type<CSharpMetadata>,
    types: &TypeSet<CSharpMetadata>,
    writer: &mut SourceWriter<impl Write>,
    resolve_aliases: bool,
) -> Result<()> {
    match r#type {
        Type::Struct(r#struct) => writer.write(&r#struct.metadata.ident),
        Type::Enum(r#enum) => writer.write(&r#enum.metadata.ident),
        Type::Versioned(versioned) => {
            write_type_name_advanced(&versioned.r#type, types, writer, resolve_aliases)
        }
        Type::List(inner) => {
            writer.write("System.Collections.Generic.List<")?;
            write_type_name_advanced(inner, types, writer, resolve_aliases)?;
            writer.write(">")
        }
        Type::Primitive(Primitive::Number) => writer.write("int"),
        Type::Primitive(Primitive::String) => writer.write("string"),
        Type::Primitive(Primitive::Unit) => writer.write("System.ValueTuple"),
        Type::Identifier(name) => match types.types.iter().find(|t| t.name == *name) {
            Some(named) => {
                if resolve_aliases {
                    write_type_name_advanced(&named.r#type, types, writer, resolve_aliases)
                } else {
                    writer.write(&named.metadata.ident)
                }
            }
            None => Err(Error::other("unknown type name")), // TODO: Check this earlier
        },
    }
}

fn emit_alias_if_needed(
    r#type: &NamedType<CSharpMetadata>,
    types: &TypeSet<CSharpMetadata>,
    writer: &mut SourceWriter<impl Write>,
) -> Result<()> {
    if !has_intrinsic_name(&r#type.r#type) {
        writer.write("using ")?;
        writer.write(&r#type.metadata.ident)?;
        writer.write(" = ")?;
        write_type_name_advanced(&r#type.r#type, types, writer, true)?;
        writer.write_nl(";")?;
    }
    Ok(())
}

fn has_intrinsic_name(r#type: &Type<CSharpMetadata>) -> bool {
    match r#type {
        Type::Struct(_) => true,
        Type::Enum(_) => true,
        Type::Versioned(versioned) => has_intrinsic_name(&versioned.r#type),
        Type::List(_) => false,
        Type::Primitive(_) => false,
        Type::Identifier(_) => false,
    }
}
