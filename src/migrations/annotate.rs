//! Adds or removes migration markers.

use std::collections::HashSet;

use crate::{
    ast::{Type, TypeSet, TypeType},
    codegen::file_patching::{AddEdit, RemoveEdit},
    preprocessing::BasicMetadata,
    syntax::Span,
};

/// The context for the pass to add migration markers.
#[derive(Debug)]
struct AnnotationContext {
    edits: Vec<AddEdit>,
    used: HashSet<u64>,
    next_number: u64,
}

/// Returns a list of [`AddEdit`]s that add migration markers to the schema file.
pub fn annotate(types: &TypeSet<BasicMetadata>) -> Vec<AddEdit> {
    let mut used = HashSet::new();

    for r#type in &types.types {
        collect_used_numbers(&r#type.r#type, &mut used);
    }

    let mut context = AnnotationContext {
        edits: vec![],
        used,
        next_number: 1,
    };

    for r#type in &types.types {
        annotate_type(&r#type.r#type, &mut context);
    }

    context.edits
}

/// Visits a type and generates [`AddEdit`]s recursively.
fn annotate_type(r#type: &Type<BasicMetadata>, context: &mut AnnotationContext) {
    if r#type.number.is_none() {
        let number = loop {
            let number = context.next_number;
            context.next_number += 1;

            if !context.used.contains(&number) {
                break number;
            }
        };

        context.edits.push(AddEdit::new(
            r#type.metadata.span.r#type.start,
            if is_span_empty(r#type.metadata.span.r#type) {
                format!(" #{number}")
            } else {
                format!("#{number} ")
            },
        ));
    }

    match &r#type.r#type {
        TypeType::Struct(r#struct) => {
            for field in &r#struct.fields {
                annotate_type(&field.r#type, context);
            }
        }
        TypeType::Enum(r#enum) => {
            for variant in &r#enum.variants {
                annotate_type(&variant.r#type, context);
            }
        }
        TypeType::List(list) => annotate_type(&list.r#type, context),
        TypeType::Primitive(_) => {}
        TypeType::Identifier(_) => {}
    }
}

/// Adds all used type numbers to `numbers` recursively.
fn collect_used_numbers(r#type: &Type<BasicMetadata>, numbers: &mut HashSet<u64>) {
    if let Some(number) = r#type.number {
        numbers.insert(number);
    }

    match &r#type.r#type {
        TypeType::Struct(r#struct) => {
            for field in &r#struct.fields {
                collect_used_numbers(&field.r#type, numbers)
            }
        }
        TypeType::Enum(r#enum) => {
            for variant in &r#enum.variants {
                collect_used_numbers(&variant.r#type, numbers)
            }
        }
        TypeType::List(list) => collect_used_numbers(&list.r#type, numbers),
        TypeType::Primitive(_) => {}
        TypeType::Identifier(_) => {}
    }
}

/// Returns a list of [`RemoveEdit`]s that remove migration markers from the schema file.
pub fn strip_annotations(types: &TypeSet<BasicMetadata>) -> Vec<RemoveEdit> {
    let mut edits = vec![];

    for r#type in &types.types {
        strip_annotations_in_type(&r#type.r#type, &mut edits);
    }

    edits
}

/// Visits a type and generates [`RemoveEdit`]s recursively.
fn strip_annotations_in_type(r#type: &Type<BasicMetadata>, edits: &mut Vec<RemoveEdit>) {
    if let Some(span) = r#type.metadata.span.number {
        if is_span_empty(r#type.metadata.span.r#type) {
            edits.push(RemoveEdit::new_trim_left(span.into_range()));
        } else {
            edits.push(RemoveEdit::new_trim_right(span.into_range()));
        }
    }

    match &r#type.r#type {
        TypeType::Struct(r#struct) => {
            for field in &r#struct.fields {
                strip_annotations_in_type(&field.r#type, edits);
            }
        }
        TypeType::Enum(r#enum) => {
            for variant in &r#enum.variants {
                strip_annotations_in_type(&variant.r#type, edits);
            }
        }
        TypeType::List(list) => strip_annotations_in_type(&list.r#type, edits),
        TypeType::Primitive(_) => {}
        TypeType::Identifier(_) => {}
    }
}

/// Checks if a span is empty.
fn is_span_empty(span: Span) -> bool {
    span.start == span.end
}
