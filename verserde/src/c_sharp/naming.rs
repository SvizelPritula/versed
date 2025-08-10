use std::collections::HashSet;

use crate::{
    ast::{Enum, Field, NamedType, Struct, Type, TypeSet, Variant},
    codegen::idents::{CaseType, PascalCase, convert_case, disambiguate},
};

use super::{CSharpMetadata, IdentMetadata, idents::CSharpIdentRules};

struct Scopes<'a> {
    local: HashSet<String>,
    global: &'a HashSet<String>,
}

impl<'a> Scopes<'a> {
    fn with_local(&self, local: impl IntoIterator<Item = String>) -> Scopes<'a> {
        Scopes {
            local: HashSet::from_iter(local),
            global: self.global,
        }
    }
}

fn make_type_ident(hint: &str, scopes: &mut Scopes, case: impl CaseType) -> String {
    let mut ident = convert_case(hint, case, CSharpIdentRules);
    disambiguate(&mut ident, |ident| {
        scopes.local.contains(ident) || scopes.global.contains(ident)
    });

    scopes.local.insert(ident.clone());
    ident
}

fn make_field_ident(hint: &str, scopes: &mut Scopes, case: impl CaseType) -> String {
    let mut ident = convert_case(hint, case, CSharpIdentRules);
    disambiguate(&mut ident, |ident| scopes.local.contains(ident));

    scopes.local.insert(ident.clone());
    ident
}

pub fn name(TypeSet { types, version }: TypeSet<()>) -> TypeSet<CSharpMetadata> {
    let mut named_types = Vec::with_capacity(types.len());

    let mut global = HashSet::new();
    let mut local = HashSet::new();

    for NamedType {
        name,
        r#type,
        metadata: (),
    } in types
    {
        let mut scopes = Scopes {
            global: &global,
            local,
        };

        let r#type = name_type(r#type, &name, &mut scopes);

        let ident = match name_of(&r#type) {
            Some(ident) => ident.to_owned(),
            None => make_type_ident(&name, &mut scopes, PascalCase),
        };

        local = scopes.local;
        global.insert(ident.to_owned());

        named_types.push(NamedType {
            name,
            r#type,
            metadata: IdentMetadata { ident },
        });
    }

    TypeSet {
        types: named_types,
        version,
    }
}

fn name_type(r#type: Type<()>, hint: &str, scopes: &mut Scopes) -> Type<CSharpMetadata> {
    match r#type {
        Type::Struct(r#struct) => Type::Struct(name_struct(r#struct, hint, scopes)),
        Type::Enum(r#enum) => Type::Enum(name_enum(r#enum, hint, scopes)),
        Type::List(inner) => Type::List(Box::new(name_type(*inner, hint, scopes))),
        Type::Primitive(primitive) => Type::Primitive(primitive),
        Type::Identifier(ident) => Type::Identifier(ident),
    }
}

fn name_struct(
    Struct {
        fields,
        metadata: (),
    }: Struct<()>,
    hint: &str,
    scopes: &mut Scopes,
) -> Struct<CSharpMetadata> {
    let ident = make_type_ident(hint, scopes, PascalCase);
    let mut scopes = scopes.with_local([ident.clone()]);

    let mut named_fields = Vec::with_capacity(fields.len());

    for Field {
        name,
        r#type,
        metadata: (),
    } in fields
    {
        let ident = make_field_ident(&name, &mut scopes, PascalCase);
        let r#type = name_type(r#type, &format!("{name} type"), &mut scopes);

        named_fields.push(Field {
            name,
            r#type,
            metadata: IdentMetadata { ident },
        });
    }

    Struct {
        fields: named_fields,
        metadata: IdentMetadata { ident },
    }
}

fn name_enum(
    Enum {
        variants,
        metadata: (),
    }: Enum<()>,
    hint: &str,
    scopes: &mut Scopes,
) -> Enum<CSharpMetadata> {
    let ident = make_type_ident(hint, scopes, PascalCase);
    let mut scopes = scopes.with_local([ident.to_owned()]);

    let mut named_variants = Vec::with_capacity(variants.len());

    for Variant {
        name,
        r#type,
        metadata: (),
    } in variants
    {
        let ident = make_type_ident(&name, &mut scopes, PascalCase);

        let r#type = {
            let mut scopes = scopes.with_local([ident.to_owned(), "Value".to_owned()]);
            name_type(r#type, &format!("{name} type"), &mut scopes)
        };

        named_variants.push(Variant {
            name,
            r#type,
            metadata: IdentMetadata { ident },
        });
    }

    Enum {
        variants: named_variants,
        metadata: IdentMetadata { ident },
    }
}

fn name_of(r#type: &Type<CSharpMetadata>) -> Option<&str> {
    match r#type {
        Type::Struct(r#struct) => Some(&r#struct.metadata.ident),
        Type::Enum(r#enum) => Some(&r#enum.metadata.ident),
        Type::List(_) => None,
        Type::Primitive(_) => None,
        Type::Identifier(_) => None,
    }
}
