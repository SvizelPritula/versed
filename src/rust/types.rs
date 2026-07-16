//! The backend for Rust type declarations.

use std::io::{Result, Write};

use crate::{
    ast::{Enum, NamedType, Struct, Type, TypeSet, TypeType},
    codegen::source_writer::SourceWriter,
    metadata::GetIdentity,
    rust::{
        RustMetadata, RustOptions,
        codegen::{self, NamingContext, all_rust_type_names},
    },
};

/// The context for the Rust type declaration backend.
#[derive(Debug, Clone, Copy)]
struct TypeCodegenContext<'a> {
    pub naming: NamingContext<'a, RustMetadata>,
    pub options: &'a RustOptions,
}

/// Emits all type declarations.
pub fn emit_types(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<RustMetadata>,
    options: &RustOptions,
) -> Result<()> {
    let used_type_names = all_rust_type_names(types, GetIdentity);

    let context = TypeCodegenContext {
        naming: NamingContext {
            types,
            used_type_names: &used_type_names,
        },
        options,
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

/// Visits a type and emits its type declaration recursively.
fn emit_type_recursive(
    writer: &mut SourceWriter<impl Write>,
    context: TypeCodegenContext,
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

/// Emits the type declaration for a struct.
fn emit_struct(
    writer: &mut SourceWriter<impl Write>,
    context: TypeCodegenContext,
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
        write_type_name(writer, context, &field.r#type)?;
        writer.write_nl(",")?;
    }

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

/// Emits the type declaration for an enum.
fn emit_enum(
    writer: &mut SourceWriter<impl Write>,
    context: TypeCodegenContext,
    r#enum: &Enum<RustMetadata>,
    name: &str,
) -> Result<()> {
    write_derive(writer, context)?;
    if context.options.serde && !context.options.serde_external_tag {
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
        write_type_name(writer, context, &variant.r#type)?;
        writer.write_nl("),")?;
    }

    writer.dedent();
    writer.write_nl("}")?;
    writer.blank_line();

    Ok(())
}

/// Emits a type alias.
fn emit_type_alias(
    writer: &mut SourceWriter<impl Write>,
    context: TypeCodegenContext,
    r#type: &NamedType<RustMetadata>,
) -> Result<()> {
    let r#type = &r#type.r#type;

    if r#type.metadata.newtype {
        write_derive(writer, context)?;
        if context.options.serde {
            writer.write_nl("#[serde(transparent)]")?;
        }

        writer.write("pub struct ")?;
        writer.write(&r#type.metadata.name)?;
        writer.write("(pub ")?;
        write_type_name(writer, context, r#type)?;
        writer.write_nl(");")?;
    } else {
        writer.write("pub type ")?;
        writer.write(&r#type.metadata.name)?;
        writer.write(" = ")?;
        write_type_name(writer, context, r#type)?;
        writer.write_nl(";")?;
    }

    Ok(())
}

/// Writes the name of the Rust type corresponding to a Versed type.
fn write_type_name(
    writer: &mut SourceWriter<impl Write>,
    context: TypeCodegenContext,
    r#type: &Type<RustMetadata>,
) -> Result<()> {
    codegen::write_type_name(
        writer,
        context.naming,
        r#type,
        format_args!(""),
        true,
        GetIdentity,
    )
}

/// Writes a [`derive`] macro.
fn write_derive(writer: &mut SourceWriter<impl Write>, context: TypeCodegenContext) -> Result<()> {
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

/// Checks if a top-level type needs to get a type alias.
///
/// Will return `true` of `type` is not a struct or enum.
fn needs_type_alias(r#type: &Type<RustMetadata>) -> bool {
    !matches!(r#type.r#type, TypeType::Struct(_) | TypeType::Enum(_))
}
