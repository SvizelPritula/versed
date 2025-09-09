use std::collections::{HashSet, VecDeque};

use crate::{
    ast::{Type, TypeSet, TypeType},
    metadata::Metadata,
    rust::RustMetadata,
};

struct BoxContext {
    queue: VecDeque<usize>,
    visited: HashSet<usize>,
    source: usize,
}

impl BoxContext {
    fn enqueue(&mut self, idx: usize) {
        if self.visited.insert(idx) {
            self.queue.push_front(idx);
        }
    }
}

pub fn mark_boxes(types: &mut TypeSet<RustMetadata>) {
    for source in 0..types.types.len() {
        let mut context = BoxContext {
            queue: VecDeque::new(),
            visited: HashSet::new(),
            source,
        };
        context.enqueue(source);

        while let Some(idx) = context.queue.pop_front() {
            let r#type = &mut types.types[idx];

            if !r#type.metadata.r#box {
                r#type.metadata.r#box |= process_type(&mut r#type.r#type, &mut context);
            }
        }
    }
}

fn process_type(r#type: &mut Type<RustMetadata>, context: &mut BoxContext) -> bool {
    match &mut r#type.r#type {
        TypeType::Struct(r#struct) => {
            for field in &mut r#struct.fields {
                if !field.metadata.r#box {
                    field.metadata.r#box |= process_type(&mut field.r#type, context);
                }
            }

            false
        }
        TypeType::Enum(r#enum) => {
            for variant in &mut r#enum.variants {
                if !variant.metadata.r#box {
                    variant.metadata.r#box |= process_type(&mut variant.r#type, context);
                }
            }

            false
        }
        TypeType::List(_list) => false,
        TypeType::Primitive(_primitive) => false,
        TypeType::Identifier(identifier) => {
            let idx = identifier.metadata.resolution;

            if idx == context.source {
                true
            } else {
                context.enqueue(idx);
                false
            }
        }
    }
}

struct NewtypeContext<'a> {
    types: &'a TypeSet<RustMetadata>,
    visited: HashSet<usize>,
    source: usize,
}

pub fn mark_newtypes(types: &mut TypeSet<RustMetadata>) {
    for source in 0..types.types.len() {
        let mut context = NewtypeContext {
            types,
            visited: [source].into_iter().collect(),
            source,
        };
        context.visited.insert(source);

        let result = has_type_reference_through_alias(&types.types[source].r#type, &mut context);
        types.types[source].metadata.newtype = result;
    }
}

fn has_type_reference_through_alias(
    r#type: &Type<RustMetadata>,
    context: &mut NewtypeContext,
) -> bool {
    match &r#type.r#type {
        TypeType::Struct(_struct) => false,
        TypeType::Enum(_enum) => false,
        TypeType::List(list) => has_type_reference_through_alias(&list.r#type, context),
        TypeType::Primitive(_primitive) => false,
        TypeType::Identifier(identifier) => {
            let index = identifier.metadata.resolution;

            if index == context.source {
                true
            } else {
                let r#type = &context.types.types[index];

                if !r#type.metadata.newtype && context.visited.insert(index) {
                    has_type_reference_through_alias(&r#type.r#type, context)
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoxMetadata;

impl Metadata for BoxMetadata {
    type Type = ();
    type TypeSet = ();
    type Named = bool;

    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type Field = bool;
    type Variant = bool;
}

#[derive(Debug, Clone, Copy)]
pub struct NewtypeMetadata;

impl Metadata for NewtypeMetadata {
    type Type = ();
    type TypeSet = ();
    type Named = bool;

    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type Field = ();
    type Variant = ();
}
