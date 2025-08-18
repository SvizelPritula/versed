use std::{collections::HashSet, marker::PhantomData};

use crate::{
    ast::{Enum, Field, Identifier, List, NamedType, Primitive, Struct, Type, TypeSet, Variant},
    codegen::idents::{CaseType, IdentRules, convert_case, disambiguate},
    metadata::{MapMetadata, Metadata},
};

struct NamingContext<A, B, Map, Case, Rules> {
    case_type: Case,
    ident_rules: Rules,
    map: Map,

    used_types: HashSet<String>,
    type_name_stack: Vec<String>,

    _phantom_a: PhantomData<A>,
    _phantom_b: PhantomData<B>,
}

impl<A, B, Map, Case, Rules> NamingContext<A, B, Map, Case, Rules>
where
    A: Metadata,
    B: Metadata,
    Map: MapMetadata<A, NameMetadata, B>,
    Case: CaseType + Copy,
    Rules: IdentRules + Copy,
{
    fn new(case_type: Case, ident_rules: Rules, map: Map) -> Self {
        NamingContext {
            case_type,
            ident_rules,
            map,
            used_types: HashSet::new(),
            type_name_stack: Vec::new(),
            _phantom_a: PhantomData::default(),
            _phantom_b: PhantomData::default(),
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

        for Field {
            name,
            r#type,
            metadata,
        } in fields
        {
            let (r#type, name) = self.push_and_name_type(r#type, name);

            new_fields.push(Field {
                name,
                r#type,
                metadata: self.map.map_field(metadata, ()),
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

        for Variant {
            name,
            r#type,
            metadata,
        } in variants
        {
            let (r#type, name) = self.push_and_name_type(r#type, name);

            new_variants.push(Variant {
                name,
                r#type,
                metadata: self.map.map_variant(metadata, ()),
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

    fn current_type_name(&self) -> String {
        self.type_name(self.type_name_stack.iter().map(String::as_str))
    }

    fn type_name<'a>(&self, parts: impl IntoIterator<Item = &'a str>) -> String {
        let mut name = convert_case(parts, self.case_type, self.ident_rules);
        disambiguate(&mut name, |name| self.used_types.contains(name));
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
    type Field = ();
    type Variant = ();
}

pub fn name<A, B, Map, Case, Rules>(
    types: TypeSet<A>,
    case_type: Case,
    ident_rules: Rules,
    map: Map,
) -> TypeSet<B>
where
    A: Metadata,
    B: Metadata,
    Map: MapMetadata<A, NameMetadata, B>,
    Case: CaseType + Copy,
    Rules: IdentRules + Copy,
{
    NamingContext::new(case_type, ident_rules, map).name_types(types)
}
