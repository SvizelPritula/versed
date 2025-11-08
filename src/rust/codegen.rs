use std::{
    collections::HashSet,
    fmt,
    io::{Result, Write},
};

use crate::{
    ast::{PrimitiveType, Type, TypeSet, TypeType},
    codegen::source_writer::SourceWriter,
    metadata::{GetMetadata, Metadata},
    rust::RustMetadata,
};

#[derive(Debug, Clone, Copy)]
pub struct NamingContext<'a, M: Metadata> {
    pub types: &'a TypeSet<M>,
    pub used_type_names: &'a HashSet<&'a str>,
}

impl<M: Metadata> NamingContext<'_, M> {
    pub fn rust_type<'b>(&self, name: &'b str, fallback: &'b str) -> &'b str {
        if self.used_type_names.contains(name) {
            fallback
        } else {
            name
        }
    }
}

pub fn write_type_name<M, GM, W>(
    writer: &mut SourceWriter<W>,
    context: NamingContext<M>,
    r#type: &Type<M>,
    self_path: fmt::Arguments,
    inspect_newtypes: bool,
    get: GM,
) -> Result<()>
where
    M: Metadata,
    W: Write,
    GM: GetMetadata<M, RustMetadata>,
{
    let metadata = get.get_type(&r#type.metadata);

    if metadata.newtype && !inspect_newtypes {
        writer.write_fmt(self_path)?;
        writer.write(&metadata.name)?;
        return Ok(());
    }

    if metadata.r#box {
        writer.write(context.rust_type("Box", "::std::boxed::Box"))?;
        writer.write("<")?;
    }

    match &r#type.r#type {
        TypeType::Struct(_) | TypeType::Enum(_) => {
            writer.write_fmt(self_path)?;
            writer.write(&metadata.name)?;
        }
        TypeType::List(list) => {
            writer.write(context.rust_type("Vec", "::std::vec::Vec"))?;
            writer.write("<")?;
            write_type_name(
                writer,
                context,
                &list.r#type,
                self_path,
                inspect_newtypes,
                get,
            )?;
            writer.write(">")?;
        }
        TypeType::Primitive(primitive) => {
            writer.write(match primitive.r#type {
                PrimitiveType::String => context.rust_type("String", "::std::string::String"),
                PrimitiveType::Number => context.rust_type("i64", "::std::primitive::i64"),
                PrimitiveType::Unit => "()",
            })?;
        }
        TypeType::Identifier(identifier) => {
            let index = get.get_identifier(&identifier.metadata).resolution;
            let r#type = &context.types.types[index].r#type;
            writer.write_fmt(self_path)?;
            writer.write(&get.get_type(&r#type.metadata).name)?;
        }
    }

    if metadata.r#box {
        writer.write(">")?;
    }

    Ok(())
}

pub fn all_rust_type_names<M: Metadata>(
    types: &TypeSet<M>,
    get_metadata: impl GetMetadata<M, RustMetadata> + Copy,
) -> HashSet<&str> {
    let mut set = HashSet::new();

    for r#type in &types.types {
        set.insert(get_metadata.get_type(&r#type.r#type.metadata).name.as_str());
        add_all_rust_type_names_for_type(&r#type.r#type, &mut set, get_metadata);
    }

    set
}

fn add_all_rust_type_names_for_type<'a, M: Metadata>(
    r#type: &'a Type<M>,
    set: &mut HashSet<&'a str>,
    get_metadata: impl GetMetadata<M, RustMetadata> + Copy,
) {
    let metadata = get_metadata.get_type(&r#type.metadata);
    match &r#type.r#type {
        TypeType::Struct(r#struct) => {
            set.insert(&metadata.name);

            for field in &r#struct.fields {
                add_all_rust_type_names_for_type(&field.r#type, set, get_metadata);
            }
        }
        TypeType::Enum(r#enum) => {
            set.insert(&metadata.name);

            for variant in &r#enum.variants {
                add_all_rust_type_names_for_type(&variant.r#type, set, get_metadata);
            }
        }
        TypeType::List(list) => add_all_rust_type_names_for_type(&list.r#type, set, get_metadata),
        TypeType::Primitive(_primitive) => {}
        TypeType::Identifier(_identifier) => {}
    }
}
