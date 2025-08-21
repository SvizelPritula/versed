use std::collections::{HashSet, VecDeque};

use crate::{
    ast::{Type, TypeSet},
    metadata::Metadata,
    rust::RustMetadata,
};

struct Context {
    queue: VecDeque<usize>,
    visited: HashSet<usize>,
    source: usize,
}

impl Context {
    fn enqueue(&mut self, idx: usize) {
        if self.visited.insert(idx) {
            self.queue.push_front(idx);
        }
    }
}

pub fn mark_boxes(types: &mut TypeSet<RustMetadata>) {
    for source in 0..types.types.len() {
        let mut context = Context {
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

fn process_type(r#type: &mut Type<RustMetadata>, context: &mut Context) -> bool {
    match r#type {
        Type::Struct(r#struct) => {
            for field in &mut r#struct.fields {
                if !field.metadata.r#box {
                    field.metadata.r#box |= process_type(&mut field.r#type, context);
                }
            }

            false
        }
        Type::Enum(r#enum) => {
            for variant in &mut r#enum.variants {
                if !variant.metadata.r#box {
                    variant.metadata.r#box |= process_type(&mut variant.r#type, context);
                }
            }

            false
        }
        Type::List(_list) => false,
        Type::Primitive(_primitive) => false,
        Type::Identifier(identifier) => {
            let idx = identifier.metadata.resolution.index;

            if idx == context.source {
                true
            } else {
                context.enqueue(idx);
                false
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoxMetadata;

impl Metadata for BoxMetadata {
    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type TypeSet = ();
    type Named = bool;
    type Field = bool;
    type Variant = bool;
}
