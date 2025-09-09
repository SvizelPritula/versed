use std::{collections::HashSet, marker::PhantomData};

use crate::{
    ast::{
        Enum, Field, Identifier, List, NamedType, Primitive, Struct, Type, TypeSet, TypeType,
        Variant,
    },
    codegen::idents::{CaseType, IdentRules, convert_case, disambiguate},
    metadata::{MapMetadata, Metadata},
};

pub trait NamingRule {
    fn name<'a, P, F>(&self, parts: P, taken: F) -> String
    where
        P: IntoIterator<Item = &'a str>,
        F: for<'b> FnMut(&'b str) -> bool;
}

impl<C: CaseType + Copy, I: IdentRules + Copy> NamingRule for (C, I) {
    fn name<'a, P, F>(&self, parts: P, taken: F) -> String
    where
        P: IntoIterator<Item = &'a str>,
        F: for<'b> FnMut(&'b str) -> bool,
    {
        let mut name = convert_case(parts, self.0, self.1);
        disambiguate(&mut name, taken);
        name
    }
}

pub trait NamingRules {
    fn r#type(&self) -> impl NamingRule;
    fn field(&self) -> impl NamingRule;
    fn variant(&self) -> impl NamingRule;
    fn version(&self) -> impl NamingRule;
}

struct NamingContext<A, B, Map, Rules> {
    rules: Rules,
    map: Map,

    used_types: HashSet<String>,
    type_name_stack: Vec<String>,

    _phantom_a: PhantomData<A>,
    _phantom_b: PhantomData<B>,
}

impl<A, B, Map, Rules> NamingContext<A, B, Map, Rules>
where
    A: Metadata,
    B: Metadata,
    Map: MapMetadata<A, NameMetadata, B>,
    Rules: NamingRules,
{
    fn new(rules: Rules, map: Map) -> Self {
        NamingContext {
            rules,
            map,

            used_types: HashSet::new(),
            type_name_stack: Vec::new(),

            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
        }
    }

    fn name_types(
        &mut self,
        TypeSet {
            version,
            types,
            metadata,
        }: TypeSet<A>,
    ) -> TypeSet<B> {
        let mut new_types = Vec::with_capacity(types.len());

        for NamedType {
            name,
            r#type,
            metadata,
        } in types
        {
            let (r#type, name) = self.push_and_name_type(r#type, name);

            new_types.push(NamedType {
                name,
                r#type,
                metadata: self.map.map_named(metadata, ()),
            });
        }

        let version_name = self.rules.version().name([version.as_str()], |_| false);

        TypeSet {
            version,
            types: new_types,
            metadata: self.map.map_type_set(metadata, version_name),
        }
    }

    fn name_type(&mut self, r#type: TypeType<A>) -> TypeType<B> {
        match r#type {
            TypeType::Struct(r#struct) => TypeType::Struct(self.name_struct(r#struct)),
            TypeType::Enum(r#enum) => TypeType::Enum(self.name_enum(r#enum)),
            TypeType::List(list) => TypeType::List(self.name_list(list)),
            TypeType::Primitive(primitive) => TypeType::Primitive(self.name_primitive(primitive)),
            TypeType::Identifier(identifier) => {
                TypeType::Identifier(self.name_identifier(identifier))
            }
        }
    }

    fn push_and_name_type(&mut self, r#type: Type<A>, name: String) -> (Type<B>, String) {
        self.type_name_stack.push(name);

        let Type { r#type, metadata } = r#type;
        let name = self.current_type_name();

        let r#type = self.name_type(r#type);

        let r#type = Type {
            r#type,
            metadata: self.map.map_type(metadata, name),
        };

        let name = self.type_name_stack.pop().unwrap();
        (r#type, name)
    }

    fn name_struct(&mut self, Struct { fields, metadata }: Struct<A>) -> Struct<B> {
        let mut new_fields = Vec::with_capacity(fields.len());
        let mut used_names = HashSet::new();

        for Field {
            name,
            r#type,
            metadata,
        } in fields
        {
            let (r#type, name) = self.push_and_name_type(r#type, name);

            let converted_name = self
                .rules
                .field()
                .name([name.as_str()], |name| used_names.contains(name));
            used_names.insert(converted_name.clone());

            new_fields.push(Field {
                name,
                r#type,
                metadata: self.map.map_field(metadata, converted_name),
            });
        }

        Struct {
            fields: new_fields,
            metadata: self.map.map_struct(metadata, ()),
        }
    }

    fn name_enum(&mut self, Enum { variants, metadata }: Enum<A>) -> Enum<B> {
        let mut new_variants = Vec::with_capacity(variants.len());
        let mut used_names = HashSet::new();

        for Variant {
            name,
            r#type,
            metadata,
        } in variants
        {
            let (r#type, name) = self.push_and_name_type(r#type, name);

            let converted_name = self
                .rules
                .variant()
                .name([name.as_str()], |name| used_names.contains(name));
            used_names.insert(converted_name.clone());

            new_variants.push(Variant {
                name,
                r#type,
                metadata: self.map.map_variant(metadata, converted_name),
            });
        }

        Enum {
            variants: new_variants,
            metadata: self.map.map_enum(metadata, ()),
        }
    }

    const LIST_ELEMENT_NAME: &str = "element";

    fn name_list(&mut self, List { r#type, metadata }: List<A>) -> List<B> {
        let (r#type, _) = self.push_and_name_type(*r#type, Self::LIST_ELEMENT_NAME.to_owned());

        List {
            r#type: Box::new(r#type),
            metadata: self.map.map_list(metadata, ()),
        }
    }

    fn name_primitive(&mut self, Primitive { r#type, metadata }: Primitive<A>) -> Primitive<B> {
        Primitive {
            r#type,
            metadata: self.map.map_primitive(metadata, ()),
        }
    }

    fn name_identifier(&mut self, Identifier { ident, metadata }: Identifier<A>) -> Identifier<B> {
        Identifier {
            ident,
            metadata: self.map.map_identifier(metadata, ()),
        }
    }

    fn current_type_name(&mut self) -> String {
        let parts = self.type_name_stack.iter().map(String::as_str);

        let name = self
            .rules
            .r#type()
            .name(parts, |name| self.used_types.contains(name));

        self.used_types.insert(name.clone());
        name
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NameMetadata;

impl Metadata for NameMetadata {
    type Type = String;
    type TypeSet = String;
    type Named = ();

    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type Field = String;
    type Variant = String;
}

pub fn name<A, B, Map, Rules>(types: TypeSet<A>, rules: Rules, map: Map) -> TypeSet<B>
where
    A: Metadata,
    B: Metadata,
    Map: MapMetadata<A, NameMetadata, B>,
    Rules: NamingRules,
{
    NamingContext::new(rules, map).name_types(types)
}
