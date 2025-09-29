use crate::{
    ast::{Type, TypeSet, TypeType},
    codegen::file_patching::AddEdit,
    preprocessing::BasicMetadata,
    syntax::Span,
};

pub fn annotate(types: &TypeSet<BasicMetadata>) -> Vec<AddEdit> {
    let mut edits = Vec::new();

    for r#type in &types.types {
        annotate_type(&r#type.r#type, &mut edits);
    }

    edits
}

fn annotate_type(r#type: &Type<BasicMetadata>, edits: &mut Vec<AddEdit>) {
    let num = edits.len() + 1;
    edits.push(AddEdit::new(
        r#type.metadata.span.span.start,
        if is_span_empty(r#type.metadata.span.span) {
            format!(" #{num}")
        } else {
            format!("#{num} ")
        },
    ));

    match &r#type.r#type {
        TypeType::Struct(r#struct) => {
            for field in &r#struct.fields {
                annotate_type(&field.r#type, edits)
            }
        }
        TypeType::Enum(r#enum) => {
            for variant in &r#enum.variants {
                annotate_type(&variant.r#type, edits)
            }
        }
        TypeType::List(list) => annotate_type(&list.r#type, edits),
        TypeType::Primitive(_) => {}
        TypeType::Identifier(_) => {}
    }
}

fn is_span_empty(span: Span) -> bool {
    span.start == span.end
}
