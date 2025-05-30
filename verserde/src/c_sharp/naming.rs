use std::collections::HashSet;

use crate::{
    ast::{Enum, Field, NamedType, Struct, Type, TypeSet, Variant, Versioned},
    idents::{CaseType, PascalCase, convert_case, disambiguate},
};

use super::{CSharpMetadata, IdentMetadata, idents::CSharpIdentRules};

struct Scopes<'a> {
    local: &'a mut HashSet<String>,
    global: &'a mut HashSet<String>,
}

impl<'a> Scopes<'a> {
    fn with_local<'b>(&'b mut self, local: &'b mut HashSet<String>) -> Scopes<'b> {
        Scopes {
            local,
            global: self.global,
        }
    }

    fn copy<'b>(&'b mut self) -> Scopes<'b> {
        Scopes {
            local: &mut self.local,
            global: &mut self.global,
        }
    }
}

fn make_ident(hint: &str, scopes: Scopes, case: impl CaseType) -> String {
    let mut ident = convert_case(hint, case, CSharpIdentRules);
    disambiguate(&mut ident, |ident| {
        scopes.local.contains(ident) || scopes.global.contains(ident)
    });

    scopes.local.insert(ident.clone());

    ident
}

pub fn name(TypeSet { types }: TypeSet<()>) -> TypeSet<CSharpMetadata> {
    let mut named_types = Vec::with_capacity(types.len());

    let mut local = HashSet::new();
    let mut global = HashSet::new();

    for NamedType {
        name,
        r#type,
        metadata: (),
    } in types
    {
        let mut scopes = Scopes {
            local: &mut local,
            global: &mut global,
        };

        let r#type = name_type(r#type, &name, scopes.copy());

        let ident = match name_of(&r#type) {
            Some(ident) => ident.to_owned(),
            None => make_ident(&name, scopes.copy(), PascalCase),
        };

        global.insert(ident.to_owned());

        named_types.push(NamedType {
            name,
            r#type,
            metadata: IdentMetadata { ident },
        });
    }

    TypeSet { types: named_types }
}

fn name_type(r#type: Type<()>, hint: &str, scopes: Scopes) -> Type<CSharpMetadata> {
    match r#type {
        Type::Struct(r#struct) => Type::Struct(name_struct(r#struct, hint, scopes)),
        Type::Enum(r#enum) => Type::Enum(name_enum(r#enum, hint, scopes)),
        Type::Versioned(versioned) => Type::Versioned(name_versioned(versioned, hint, scopes)),
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
    mut scopes: Scopes,
) -> Struct<CSharpMetadata> {
    let ident = make_ident(hint, scopes.copy(), PascalCase);

    let mut local = HashSet::new();
    let mut scopes = scopes.with_local(&mut local);

    let mut named_fields = Vec::with_capacity(fields.len());

    for Field {
        name,
        r#type,
        metadata: (),
    } in fields
    {
        let ident = make_ident(&name, scopes.copy(), PascalCase);
        let r#type = name_type(r#type, &format!("{name} type"), scopes.copy());

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
    mut scopes: Scopes,
) -> Enum<CSharpMetadata> {
    let ident = make_ident(hint, scopes.copy(), PascalCase);

    let mut local = HashSet::new();
    let mut scopes = scopes.with_local(&mut local);

    let mut named_variants = Vec::with_capacity(variants.len());

    for Variant {
        name,
        r#type,
        metadata: (),
    } in variants
    {
        let ident = make_ident(&name, scopes.copy(), PascalCase);

        let r#type = {
            let mut local = HashSet::new();
            local.insert(ident.clone());
            let mut scopes = scopes.with_local(&mut local);

            name_type(r#type, &format!("{name} type"), scopes.copy())
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

fn name_versioned(
    Versioned { r#type }: Versioned<()>,
    hint: &str,
    scopes: Scopes,
) -> Versioned<CSharpMetadata> {
    Versioned {
        r#type: Box::new(name_type(*r#type, hint, scopes)),
    }
}

fn name_of(r#type: &Type<CSharpMetadata>) -> Option<&str> {
    match r#type {
        Type::Struct(r#struct) => Some(&r#struct.metadata.ident),
        Type::Enum(r#enum) => Some(&r#enum.metadata.ident),
        Type::Versioned(versioned) => name_of(&versioned.r#type),
        Type::Primitive(_) => None,
        Type::Identifier(_) => None,
    }
}
