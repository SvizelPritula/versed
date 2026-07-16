//! The passes that detect locations where [`Box`]es and newtypes need to be added.

use std::collections::{HashSet, VecDeque};

use crate::{
    ast::{Type, TypeSet, TypeType},
    metadata::Metadata,
    rust::RustMetadata,
};

/// The context for one iteration of the [`Box`]-insertion pass.
struct BoxContext {
    queue: VecDeque<usize>,
    visited: HashSet<usize>,
    source: usize,
}

impl BoxContext {
    /// Queues a new type to be visited.
    fn enqueue(&mut self, idx: usize) {
        if self.visited.insert(idx) {
            self.queue.push_front(idx);
        }
    }
}

/// Runs the [`Box`]-insertion pass.
pub fn mark_boxes(types: &mut TypeSet<RustMetadata>) {
    for source in 0..types.types.len() {
        let mut context = BoxContext {
            queue: VecDeque::new(),
            visited: HashSet::new(),
            source,
        };
        context.enqueue(source);

        while let Some(idx) = context.queue.pop_front() {
            let r#type = &mut types.types[idx].r#type;

            if !r#type.metadata.r#box {
                r#type.metadata.r#box |= process_type(r#type, &mut context);
            }
        }
    }
}

/// Visits a type and checks if it needs to be boxed.
///
/// Returns `true` if it needs to be boxed.
fn process_type(r#type: &mut Type<RustMetadata>, context: &mut BoxContext) -> bool {
    match &mut r#type.r#type {
        TypeType::Struct(r#struct) => {
            for field in &mut r#struct.fields {
                let r#type = &mut field.r#type;
                if !r#type.metadata.r#box {
                    r#type.metadata.r#box |= process_type(r#type, context);
                }
            }

            false
        }
        TypeType::Enum(r#enum) => {
            for variant in &mut r#enum.variants {
                let r#type = &mut variant.r#type;
                if !r#type.metadata.r#box {
                    r#type.metadata.r#box |= process_type(r#type, context);
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

/// The context for one iteration of the newtype-insertion pass.
struct NewtypeContext<'a> {
    types: &'a TypeSet<RustMetadata>,
    visited: HashSet<usize>,
    source: usize,
}

/// Runs the newtype-insertion pass.
pub fn mark_newtypes(types: &mut TypeSet<RustMetadata>) {
    for source in 0..types.types.len() {
        let mut context = NewtypeContext {
            types,
            visited: [source].into_iter().collect(),
            source,
        };
        context.visited.insert(source);

        let result = has_type_reference_through_alias(&types.types[source].r#type, &mut context);
        types.types[source].r#type.metadata.newtype = result;
    }
}

/// Checks if the type contains a reference to [`NewtypeContext::source`] not through structs or enums.
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
                let r#type = &context.types.types[index].r#type;

                if !r#type.metadata.newtype && context.visited.insert(index) {
                    has_type_reference_through_alias(r#type, context)
                } else {
                    false
                }
            }
        }
    }
}

/// Metadata that marks types than need to be boxed.
#[derive(Debug, Clone, Copy)]
pub struct BoxMetadata;

impl Metadata for BoxMetadata {
    type Type = bool;
    type TypeSet = ();
    type Named = ();

    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type Field = ();
    type Variant = ();
}

/// Metadata that marks types than need to become newtypes.
#[derive(Debug, Clone, Copy)]
pub struct NewtypeMetadata;

impl Metadata for NewtypeMetadata {
    type Type = bool;
    type TypeSet = ();
    type Named = ();

    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type Field = ();
    type Variant = ();
}
