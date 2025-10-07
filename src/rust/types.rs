use std::io::{Result, Write};

use crate::{
    ast::{Enum, NamedType, Struct, Type, TypeSet, TypeType},
    codegen::source_writer::SourceWriter,
    metadata::GetIdentity,
    rust::{
        RustMetadata, RustOptions,
        codegen::{self, all_rust_type_names},
    },
};

type Context<'a> = codegen::Context<'a, RustMetadata>;

pub fn emit_types(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
    options: &RustOptions,
) -> Result<()> {
    let used_type_names = all_rust_type_names(types, GetIdentity);

    let context = Context {
        types,
        options,
        used_type_names: &used_type_names,
    };

    if context.options.serde {
        writer.write_nl(r#"use serde::{Deserialize, Serialize};"#)?;
        writer.blank_line();
    }

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
    match &r#type.r#type {
        TypeType::Struct(r#struct) => {
            emit_struct(writer, context, r#struct, &r#type.metadata.name)?;

            for field in &r#struct.fields {
                emit_type_recursive(writer, context, &field.r#type)?;
            }
        }
        TypeType::Enum(r#enum) => {
            emit_enum(writer, context, r#enum, &r#type.metadata.name)?;

            for variant in &r#enum.variants {
                emit_type_recursive(writer, context, &variant.r#type)?;
            }
        }
        TypeType::List(list) => emit_type_recursive(writer, context, &list.r#type)?,
        TypeType::Primitive(_) => {}
        TypeType::Identifier(_) => {}
    }

    Ok(())
}

fn emit_struct(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    r#struct: &Struct<RustMetadata>,
    name: &str,
) -> Result<()> {
    write_derive(writer, context)?;
    writer.write("pub struct ")?;
    writer.write(name)?;
    writer.write_nl(" {")?;
    writer.indent();

    for field in &r#struct.fields {
        if context.options.serde && field.metadata.serde_name != field.metadata.name {
            writer.write(r#"#[serde(rename = ""#)?;
            writer.write(&field.metadata.serde_name)?;
            writer.write_nl(r#"")]"#)?;
        }

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
    name: &str,
) -> Result<()> {
    write_derive(writer, context)?;
    if context.options.serde {
        writer.write_nl(r#"#[serde(tag = "type", content = "value")]"#)?;
    }

    writer.write("pub enum ")?;
    writer.write(name)?;
    writer.write_nl(" {")?;
    writer.indent();

    for variant in &r#enum.variants {
        if context.options.serde && variant.metadata.serde_name != variant.metadata.name {
            writer.write(r#"#[serde(rename = ""#)?;
            writer.write(&variant.metadata.serde_name)?;
            writer.write_nl(r#"")]"#)?;
        }

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
    if r#type.metadata.newtype {
        write_derive(writer, context)?;
        if context.options.serde {
            writer.write_nl("#[serde(transparent)]")?;
        }

        writer.write("pub struct ")?;
        writer.write(&r#type.r#type.metadata.name)?;
        writer.write("(pub ")?;
        write_type_name(writer, context, &r#type.r#type, r#type.metadata.r#box)?;
        writer.write_nl(");")?;
    } else {
        writer.write("pub type ")?;
        writer.write(&r#type.r#type.metadata.name)?;
        writer.write(" = ")?;
        write_type_name(writer, context, &r#type.r#type, r#type.metadata.r#box)?;
        writer.write_nl(";")?;
    }

    Ok(())
}

fn write_type_name(
    writer: &mut SourceWriter<impl Write>,
    context: Context,
    r#type: &Type<RustMetadata>,
    r#box: bool,
) -> Result<()> {
    codegen::write_type_name(writer, context, r#type, r#box, format_args!(""), GetIdentity)
}

fn write_derive(writer: &mut SourceWriter<impl Write>, context: Context) -> Result<()> {
    writer.write("#[derive(")?;

    for (index, name) in context.options.derives.iter().enumerate() {
        if index > 0 {
            writer.write(", ")?;
        }

        writer.write(name)?;
    }

    writer.write_nl(")]")?;

    Ok(())
}

fn needs_type_alias(r#type: &Type<RustMetadata>) -> bool {
    !matches!(r#type.r#type, TypeType::Struct(_) | TypeType::Enum(_))
}
