//! A generic pass that assigns every named entity a new name in the target language.

use std::{collections::HashSet, marker::PhantomData};

use crate::{
    ast::{
        Enum, Field, Identifier, List, NamedType, Primitive, Struct, Type, TypeSet, TypeType,
        Variant,
    },
    codegen::idents::{CaseType, IdentRules, convert_case, disambiguate},
    metadata::{MapMetadata, Metadata},
};

/// Gives names to a specific language entity.
///
/// Has an implementation for tuples of [`CaseType`] and [`IdentRules`]
/// that calls [`convert_case`] and [`disambiguate`],
/// but a custom implementation can be provided as well.
pub trait NamingRule {
    /// Gives a name to an entity constructed from a sequence of parts.
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

/// A collection of rules to name every type of named entity.
pub trait NamingRules {
    /// Gets the rules for naming top level types.
    fn r#type(&self) -> impl NamingRule;
    /// Gets the rules for naming fields.
    fn field(&self) -> impl NamingRule;
    /// Gets the rules for naming variants.
    fn variant(&self) -> impl NamingRule;
    /// Gets the rules for naming versions, which usually get converted to namespaces or modules.
    fn version(&self) -> impl NamingRule;
}

/// Holds all information needed during the naming pass.
struct NamingContext<A, B, Map, Rules> {
    /// The [`NamingRules`] to use.
    rules: Rules,
    /// How to combine `A` and [`NameMetadata`] into `B`.
    map: Map,

    /// The names already used for some types.
    used_types: HashSet<String>,
    /// The type, field, variant and element names on the path to the current type.
    ///
    /// Used for naming anonymous types.
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
    /// Constructs a new context.
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

    /// Names all types in a [`TypeSet`].
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

    /// Visits and names a type recursively.
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

    /// Visits and names a type recursively, with `name` added to [`NamingContext::type_name_stack`].
    fn push_and_name_type(&mut self, r#type: Type<A>, name: String) -> (Type<B>, String) {
        self.type_name_stack.push(name);

        let Type {
            r#type,
            number,
            metadata,
        } = r#type;
        let name = self.current_type_name();

        let r#type = self.name_type(r#type);

        let r#type = Type {
            r#type,
            number,
            metadata: self.map.map_type(metadata, name),
        };

        let name = self.type_name_stack.pop().unwrap();
        (r#type, name)
    }

    /// Visits and names a struct recursively.
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

    /// Visits and names an enum recursively.
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

    /// The name added to [`NamingContext::type_name_stack`] to refer to the element of a list.
    const LIST_ELEMENT_NAME: &str = "element";

    /// Visits and names a list recursively.
    fn name_list(&mut self, List { r#type, metadata }: List<A>) -> List<B> {
        let (r#type, _) = self.push_and_name_type(*r#type, Self::LIST_ELEMENT_NAME.to_owned());

        List {
            r#type: Box::new(r#type),
            metadata: self.map.map_list(metadata, ()),
        }
    }

    /// Visits and names a primitive.
    fn name_primitive(&mut self, Primitive { r#type, metadata }: Primitive<A>) -> Primitive<B> {
        Primitive {
            r#type,
            metadata: self.map.map_primitive(metadata, ()),
        }
    }

    /// Visits and names an identifier.
    fn name_identifier(&mut self, Identifier { ident, metadata }: Identifier<A>) -> Identifier<B> {
        Identifier {
            ident,
            metadata: self.map.map_identifier(metadata, ()),
        }
    }

    /// Constructs a name for the current type based on [`NamingContext::type_name_stack`].
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

/// Metadata that assigns every named entity a name in the target language.
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

/// Assigns every named entity a new name in the target language.
///
/// `A` is the metadata type of the input, `B` is the metadata type of the output.
/// `Map` combines `A` with [`NameMetadata`] to produce `B`.
/// `Rules` defines the naming conventions and restrictions.
pub fn name<A, B, Map, Rules>(types: TypeSet<A>, rules: Rules, map: Map) -> TypeSet<B>
where
    A: Metadata,
    B: Metadata,
    Map: MapMetadata<A, NameMetadata, B>,
    Rules: NamingRules,
{
    NamingContext::new(rules, map).name_types(types)
}
