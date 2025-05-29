#[macro_export]
macro_rules! r#type {
    (($($t:tt)*)) => {r#type!($($t)*)};
    (struct { $($field:ident : $type:tt),* }) => {
        crate::ast::Type::<()>::Struct(crate::ast::Struct {
            fields: vec![
                $(crate::ast::Field {
                    name: stringify!($field).into(),
                    r#type: r#type!($type),
                    metadata: ()
                }),*
            ],
            metadata: ()
        })
    };
    (enum { $($variant:ident : $type:tt),* }) => {
        crate::ast::Type::<()>::Enum(crate::ast::Enum {
            variants: vec![
                $(crate::ast::Variant {
                    name: stringify!($variant).into(),
                    r#type: r#type!($type),
                    metadata: ()
                }),*
            ],
            metadata: ()
        })
    };
    (versioned $($t:tt)*) => {
        crate::ast::Type::<()>::Versioned(crate::ast::Versioned {
            r#type: Box::new(r#type!($($t)*))
        })
    };
    (string) => {
        crate::ast::Type::<()>::Primitive(crate::ast::Primitive::String)
    };
    (number) => {
        crate::ast::Type::<()>::Primitive(crate::ast::Primitive::Number)
    };
    (unit) => {
        crate::ast::Type::<()>::Primitive(crate::ast::Primitive::Unit)
    };
    ($name:ident) => {
        crate::ast::Type::<()>::Identifier(stringify!($name).into())
    };
}

#[macro_export]
macro_rules! r#types {
    { $($name:ident = $type:tt);* } => {
        crate::ast::TypeSet::<()> {
            types: vec![
                $(crate::ast::NamedType {
                    name: stringify!($name).into(),
                    r#type: r#type!($type),
                    metadata: ()
                }),*
            ]
        }
    };
}
