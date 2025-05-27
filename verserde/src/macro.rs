#[macro_export]
macro_rules! r#type {
    (($($t:tt)*)) => {r#type!($($t)*)};
    (struct { $($field:ident : $type:tt),* }) => {
        crate::ast::Type {
            r#type: crate::ast::TypeType::Struct(crate::ast::Struct {
                fields: vec![
                    $(crate::ast::Field {
                        name: stringify!($field).into(),
                        r#type: r#type!($type)
                    }),*
                ]
            }),
            metadata: ()
        }
    };
    (enum { $($variant:ident : $type:tt),* }) => {
        crate::ast::Type {
            r#type: crate::ast::TypeType::Enum(crate::ast::Enum {
                variants: vec![
                    $(crate::ast::Variant {
                        name: stringify!($variant).into(),
                        r#type: r#type!($type)
                    }),*
                ]
            }),
            metadata: ()
        }
    };
    (versioned $($t:tt)*) => {
        crate::ast::Type {
            r#type: crate::ast::TypeType::Versioned(crate::ast::Versioned {
                r#type: Box::new(r#type!($($t)*))
            }),
            metadata: ()
        }
    };
    (string) => {
        crate::ast::Type {
            r#type: crate::ast::TypeType::Primitive(crate::ast::Primitive::String),
            metadata: ()
        }
    };
    (number) => {
        crate::ast::Type {
            r#type: crate::ast::TypeType::Primitive(crate::ast::Primitive::Number),
            metadata: ()
        }
    };
    (unit) => {
        crate::ast::Type {
            r#type: crate::ast::TypeType::Primitive(crate::ast::Primitive::Unit),
            metadata: ()
        }
    };
    ($name:ident) => {
        crate::ast::Type {
            r#type: crate::ast::TypeType::Identifier(stringify!($name).into()),
            metadata: ()
        }
    };
}

#[macro_export]
macro_rules! r#types {
    { $($name:ident = $type:tt);* } => {
        crate::ast::TypeSet::<()> {
            types: vec![
                $(crate::ast::NamedType {
                    name: stringify!($name).into(),
                    r#type: r#type!($type)
                }),*
            ]
        }
    };
}
