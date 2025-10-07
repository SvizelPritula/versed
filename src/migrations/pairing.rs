use std::collections::HashMap;

use crate::{
    ast::{Migration, Type, TypeSet, TypeType},
    metadata::Metadata,
};

#[derive(Debug)]
pub struct TypePair<'types, M: Metadata> {
    pub old: &'types Type<M>,
    pub new: &'types Type<M>,
}

impl<'types, M: Metadata> Copy for TypePair<'types, M> {}
impl<'types, M: Metadata> Clone for TypePair<'types, M> {
    fn clone(&self) -> Self {
        *self
    }
}

pub fn pair_types<'types, M: Metadata>(
    migration: &'types Migration<M>,
) -> Vec<TypePair<'types, M>> {
    type Element<'types, M> = (Option<&'types Type<M>>, Option<&'types Type<M>>);
    let mut map: HashMap<u64, Element<M>> = HashMap::new();

    collect_type_set(&migration.old, &mut map, |pair, value| pair.0 = Some(value));
    collect_type_set(&migration.new, &mut map, |pair, value| pair.1 = Some(value));

    let mut vec = map
        .into_iter()
        .flat_map(|(number, (old, new))| {
            old.zip(new)
                .map(|(old, new)| (number, TypePair { old, new }))
        })
        .collect::<Vec<_>>();

    vec.sort_by_key(|(n, _)| *n);
    vec.into_iter().map(|(_, p)| p).collect()
}

fn collect_type_set<'types, M, E, F>(types: &'types TypeSet<M>, map: &mut HashMap<u64, E>, set: F)
where
    M: Metadata,
    E: Default,
    F: Fn(&mut E, &'types Type<M>) + Copy,
{
    for r#type in &types.types {
        collect_type(&r#type.r#type, map, set);
    }
}

fn collect_type<'types, M, E, F>(r#type: &'types Type<M>, map: &mut HashMap<u64, E>, set: F)
where
    M: Metadata,
    E: Default,
    F: Fn(&mut E, &'types Type<M>) + Copy,
{
    if let Some(number) = r#type.number {
        set(map.entry(number).or_default(), r#type)
    }

    match &r#type.r#type {
        TypeType::Struct(r#struct) => {
            for field in &r#struct.fields {
                collect_type(&field.r#type, map, set);
            }
        }
        TypeType::Enum(r#enum) => {
            for variant in &r#enum.variants {
                collect_type(&variant.r#type, map, set);
            }
        }
        TypeType::List(list) => collect_type(&list.r#type, map, set),
        TypeType::Primitive(_primitive) => {}
        TypeType::Identifier(_identifier) => {}
    }
}
