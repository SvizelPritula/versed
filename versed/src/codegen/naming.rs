use std::{collections::HashSet, marker::PhantomData};

use crate::{
    ast::{Enum, Field, Identifier, List, NamedType, Primitive, Struct, Type, TypeSet, Variant},
    codegen::idents::{CaseType, IdentRules, convert_case, disambiguate},
    metadata::{MapMetadata, Metadata},
};

struct NamingContext<A, B, Map, Types, Fields, Variants, Rules> {
    ident_rules: Rules,
    type_case: Types,
    field_case: Fields,
    variant_case: Variants,
    map: Map,

    used_types: HashSet<String>,
    type_name_stack: Vec<String>,

    _phantom_a: PhantomData<A>,
    _phantom_b: PhantomData<B>,
}

impl<A, B, Map, Types, Fields, Variants, Rules>
    NamingContext<A, B, Map, Types, Fields, Variants, Rules>
where
    A: Metadata,
    B: Metadata,
    Map: MapMetadata<A, NameMetadata, B>,
    Types: CaseType + Copy,
    Fields: CaseType + Copy,
    Variants: CaseType + Copy,
    Rules: IdentRules + Copy,
{
    fn new(
        type_case: Types,
        field_case: Fields,
        variant_case: Variants,
        ident_rules: Rules,
        map: Map,
    ) -> Self {
        NamingContext {
            ident_rules,
            type_case,
            field_case,
            variant_case,
            map,
            used_types: HashSet::new(),
            type_name_stack: Vec::new(),
            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
        }
    }

    fn name_types(&mut self, TypeSet { version, types }: TypeSet<A>) -> TypeSet<B> {
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
                metadata: self.map.map_name(metadata, ()),
            });
        }

        TypeSet {
            version,
            types: new_types,
        }
    }

    fn name_type(&mut self, r#type: Type<A>) -> Type<B> {
        match r#type {
            Type::Struct(r#struct) => Type::Struct(self.name_struct(r#struct)),
            Type::Enum(r#enum) => Type::Enum(self.name_enum(r#enum)),
            Type::List(list) => Type::List(self.name_list(list)),
            Type::Primitive(primitive) => Type::Primitive(self.name_primitive(primitive)),
            Type::Identifier(identifier) => Type::Identifier(self.name_identifier(identifier)),
        }
    }

    fn push_and_name_type(&mut self, r#type: Type<A>, name: String) -> (Type<B>, String) {
        self.type_name_stack.push(name);
        let r#type = self.name_type(r#type);
        let name = self.type_name_stack.pop().unwrap();
        (r#type, name)
    }

    fn name_struct(&mut self, Struct { fields, metadata }: Struct<A>) -> Struct<B> {
        let name = self.current_type_name();
        let mut new_fields = Vec::with_capacity(fields.len());
        let mut used_names = HashSet::new();

        for Field {
            name,
            r#type,
            metadata,
        } in fields
        {
            let (r#type, name) = self.push_and_name_type(r#type, name);

            let mut converted_name =
                convert_case([name.as_str()], self.field_case, self.ident_rules);
            disambiguate(&mut converted_name, |name| used_names.contains(name));
            used_names.insert(converted_name.clone());

            new_fields.push(Field {
                name,
                r#type,
                metadata: self.map.map_field(metadata, converted_name),
            });
        }

        Struct {
            fields: new_fields,
            metadata: self.map.map_struct(metadata, name),
        }
    }

    fn name_enum(&mut self, Enum { variants, metadata }: Enum<A>) -> Enum<B> {
        let name = self.current_type_name();
        let mut new_variants = Vec::with_capacity(variants.len());
        let mut used_names = HashSet::new();

        for Variant {
            name,
            r#type,
            metadata,
        } in variants
        {
            let (r#type, name) = self.push_and_name_type(r#type, name);

            let mut converted_name =
                convert_case([name.as_str()], self.variant_case, self.ident_rules);
            disambiguate(&mut converted_name, |name| used_names.contains(name));
            used_names.insert(converted_name.clone());

            new_variants.push(Variant {
                name,
                r#type,
                metadata: self.map.map_variant(metadata, converted_name),
            });
        }

        Enum {
            variants: new_variants,
            metadata: self.map.map_enum(metadata, name),
        }
    }

    const LIST_ELEMENT_NAME: &str = "element";

    fn name_list(&mut self, List { r#type, metadata }: List<A>) -> List<B> {
        let name = self.current_type_name();
        let (r#type, _) = self.push_and_name_type(*r#type, Self::LIST_ELEMENT_NAME.to_owned());

        List {
            r#type: Box::new(r#type),
            metadata: self.map.map_list(metadata, name),
        }
    }

    fn name_primitive(&mut self, Primitive { r#type, metadata }: Primitive<A>) -> Primitive<B> {
        let name = self.current_type_name();

        Primitive {
            r#type,
            metadata: self.map.map_primitive(metadata, name),
        }
    }

    fn name_identifier(&mut self, Identifier { ident, metadata }: Identifier<A>) -> Identifier<B> {
        let name = self.current_type_name();

        Identifier {
            ident,
            metadata: self.map.map_identifier(metadata, name),
        }
    }

    fn current_type_name(&mut self) -> String {
        let parts = self.type_name_stack.iter().map(String::as_str);

        let mut name = convert_case(parts, self.type_case, self.ident_rules);
        disambiguate(&mut name, |name| self.used_types.contains(name));

        self.used_types.insert(name.clone());
        name
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NameMetadata;

impl Metadata for NameMetadata {
    type Struct = String;
    type Enum = String;
    type List = String;
    type Primitive = String;
    type Identifier = String;

    type Name = ();
    type Field = String;
    type Variant = String;
}

pub fn name<A, B, Map, Types, Fields, Variants, Rules>(
    types: TypeSet<A>,
    type_case: Types,
    field_case: Fields,
    variant_case: Variants,
    ident_rules: Rules,
    map: Map,
) -> TypeSet<B>
where
    A: Metadata,
    B: Metadata,
    Map: MapMetadata<A, NameMetadata, B>,
    Types: CaseType + Copy,
    Fields: CaseType + Copy,
    Variants: CaseType + Copy,
    Rules: IdentRules + Copy,
{
    NamingContext::new(type_case, field_case, variant_case, ident_rules, map).name_types(types)
}
