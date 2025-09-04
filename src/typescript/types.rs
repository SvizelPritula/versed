use std::io::{Result, Write};

use crate::{
    ast::{PrimitiveType, Type, TypeSet},
    codegen::source_writer::SourceWriter,
    typescript::TypeScriptMetadata,
};

pub fn emit_types(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<TypeScriptMetadata>,
) -> Result<()> {
    for (index, r#type) in types.types.iter().enumerate() {
        writer.write("export type ")?;
        writer.write(type_name(&r#type.r#type))?;
        writer.write(" = ")?;

        if is_anomalously_recursive(&r#type.r#type, index) {
            writer.write("never")?;
        } else {
            emit_type(writer, types, &r#type.r#type)?;
        }

        writer.write_nl(";")?;
        writer.blank_line();
    }

    if types.types.is_empty() {
        writer.write_nl("export {};")?;
    }

    Ok(())
}

fn emit_type(
    writer: &mut SourceWriter<impl Write>,
    types: &TypeSet<TypeScriptMetadata>,
    r#type: &Type<TypeScriptMetadata>,
) -> Result<()> {
    match r#type {
        Type::Struct(r#struct) => {
            writer.write_nl("{")?;
            writer.indent();

            for field in &r#struct.fields {
                writer.write(&field.metadata.name)?;
                writer.write(": ")?;
                emit_type(writer, types, &field.r#type)?;
                writer.write_nl(",")?;
            }

            writer.dedent();
            writer.write("}")?;
        }
        Type::Enum(r#enum) => {
            if !r#enum.variants.is_empty() {
                writer.write_nl("(")?;
                writer.indent();

                for (i, variant) in r#enum.variants.iter().enumerate() {
                    if i > 0 {
                        writer.write(" | ")?;
                    }

                    writer.write_nl("{")?;
                    writer.indent();

                    writer.write("type: \"")?;
                    writer.write(&variant.metadata.name)?;
                    writer.write_nl("\",")?;

                    writer.write("value: ")?;
                    emit_type(writer, types, &variant.r#type)?;
                    writer.write_nl(",")?;

                    writer.dedent();
                    writer.write("}")?;
                }

                writer.nl()?;
                writer.dedent();
                writer.write(")")?;
            } else {
                writer.write("never")?;
            }
        }
        Type::List(list) => {
            emit_type(writer, types, &list.r#type)?;
            writer.write("[]")?;
        }
        Type::Primitive(primitive) => {
            let keyword = match primitive.r#type {
                PrimitiveType::String => "string",
                PrimitiveType::Number => "number",
                PrimitiveType::Unit => "null",
            };
            writer.write(keyword)?;
        }
        Type::Identifier(identifier) => {
            let r#type = &types.types[identifier.metadata.resolution].r#type;
            writer.write(type_name(r#type))?;
        }
    }

    Ok(())
}

fn type_name(r#type: &Type<TypeScriptMetadata>) -> &str {
    match r#type {
        Type::Struct(r#struct) => &r#struct.metadata.name,
        Type::Enum(r#enum) => &r#enum.metadata.name,
        Type::List(list) => &list.metadata.name,
        Type::Primitive(primitive) => &primitive.metadata.name,
        Type::Identifier(identifier) => &identifier.metadata.name,
    }
}

fn is_anomalously_recursive(r#type: &Type<TypeScriptMetadata>, index: usize) -> bool {
    if let Type::Identifier(identifier) = r#type {
        identifier.metadata.resolution == index
    } else {
        false
    }
}
