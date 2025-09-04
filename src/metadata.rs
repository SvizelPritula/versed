use std::fmt::Debug;

pub trait Metadata: Clone {
    type Struct: Debug + Clone;
    type Enum: Debug + Clone;
    type List: Debug + Clone;
    type Primitive: Debug + Clone;
    type Identifier: Debug + Clone;

    type TypeSet: Debug + Clone;
    type Named: Debug + Clone;
    type Field: Debug + Clone;
    type Variant: Debug + Clone;
}

impl Metadata for () {
    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type TypeSet = ();
    type Named = ();
    type Field = ();
    type Variant = ();
}

pub trait MapMetadata<A: Metadata, B: Metadata, R: Metadata> {
    fn map_struct(&self, left: A::Struct, right: B::Struct) -> R::Struct;
    fn map_enum(&self, left: A::Enum, right: B::Enum) -> R::Enum;
    fn map_list(&self, left: A::List, right: B::List) -> R::List;
    fn map_primitive(&self, left: A::Primitive, right: B::Primitive) -> R::Primitive;
    fn map_identifier(&self, left: A::Identifier, right: B::Identifier) -> R::Identifier;

    fn map_type_set(&self, left: A::TypeSet, right: B::TypeSet) -> R::TypeSet;
    fn map_named(&self, left: A::Named, right: B::Named) -> R::Named;
    fn map_field(&self, left: A::Field, right: B::Field) -> R::Field;
    fn map_variant(&self, left: A::Variant, right: B::Variant) -> R::Variant;
}

pub trait GetMetadata<A: Metadata, R: Metadata> {
    fn get_struct<'a>(&self, metadata: &'a A::Struct) -> &'a R::Struct;
    fn get_enum<'a>(&self, metadata: &'a A::Enum) -> &'a R::Enum;
    fn get_list<'a>(&self, metadata: &'a A::List) -> &'a R::List;
    fn get_primitive<'a>(&self, metadata: &'a A::Primitive) -> &'a R::Primitive;
    fn get_identifier<'a>(&self, metadata: &'a A::Identifier) -> &'a R::Identifier;

    fn get_type_set<'a>(&self, metadata: &'a A::TypeSet) -> &'a R::TypeSet;
    fn get_named<'a>(&self, metadata: &'a A::Named) -> &'a R::Named;
    fn get_field<'a>(&self, metadata: &'a A::Field) -> &'a R::Field;
    fn get_variant<'a>(&self, metadata: &'a A::Variant) -> &'a R::Variant;
}

#[macro_export]
macro_rules! mapper {
    {fn $name: ident($left_var: ident: $left: ty, $right_var: ident: $right: ty) -> $result: ty $body: block} => {
        struct $name;

        macro_rules! mapper_func {
            ($func: ident, $subtype: ident, $metadata: ty) => {
                fn $func(&self, $left_var: <$left as $metadata>::$subtype, $right_var: <$right as $metadata>::$subtype) -> <$result as $metadata>::$subtype $body
            };
        }

        impl $crate::metadata::MapMetadata<$left, $right, $result> for $name {
            mapper_func!(map_struct, Struct, $crate::metadata::Metadata);
            mapper_func!(map_enum, Enum, $crate::metadata::Metadata);
            mapper_func!(map_list, List, $crate::metadata::Metadata);
            mapper_func!(map_primitive, Primitive, $crate::metadata::Metadata);
            mapper_func!(map_identifier, Identifier, $crate::metadata::Metadata);

            mapper_func!(map_type_set, TypeSet, $crate::metadata::Metadata);
            mapper_func!(map_named, Named, $crate::metadata::Metadata);
            mapper_func!(map_field, Field, $crate::metadata::Metadata);
            mapper_func!(map_variant, Variant, $crate::metadata::Metadata);
        }
    };
}

#[macro_export]
macro_rules! getter {
    {fn $name: ident($metadata_var: ident: $metadata: ty) -> $result: ty $body: block} => {
        struct $name;

        macro_rules! getter_func {
            ($func: ident, $subtype: ident, $meta_trait: ty) => {
                fn $func<'a>(&self, $metadata_var: &'a <$metadata as $meta_trait>::$subtype) -> &'a <$result as $meta_trait>::$subtype $body
            };
        }

        impl GetMetadata<$metadata, $result> for $name {
            getter_func!(get_struct, Struct, $crate::metadata::Metadata);
            getter_func!(get_enum, Enum, $crate::metadata::Metadata);
            getter_func!(get_list, List, $crate::metadata::Metadata);
            getter_func!(get_primitive, Primitive, $crate::metadata::Metadata);
            getter_func!(get_identifier, Identifier, $crate::metadata::Metadata);

            getter_func!(get_type_set, TypeSet, $crate::metadata::Metadata);
            getter_func!(get_named, Named, $crate::metadata::Metadata);
            getter_func!(get_field, Field, $crate::metadata::Metadata);
            getter_func!(get_variant, Variant, $crate::metadata::Metadata);
        }
    };
}

#[macro_export]
macro_rules! composite {
    {$visibility: vis struct ($element: ident, $metadata: ident) {$($field: ident: $type: ty | $generic: ident),*}} => {
        #[derive(Debug, Clone)]
        $visibility struct $element<$($generic),*> {
            $(pub $field: $generic),*
        }

        #[derive(Debug, Clone)]
        $visibility struct $metadata;

        impl $crate::metadata::Metadata for $metadata {
            type Struct = $element<$(<$type as $crate::metadata::Metadata>::Struct),*>;
            type Enum = $element<$(<$type as $crate::metadata::Metadata>::Enum),*>;
            type List = $element<$(<$type as $crate::metadata::Metadata>::List),*>;
            type Primitive = $element<$(<$type as $crate::metadata::Metadata>::Primitive),*>;
            type Identifier = $element<$(<$type as $crate::metadata::Metadata>::Identifier),*>;

            type TypeSet = $element<$(<$type as $crate::metadata::Metadata>::TypeSet),*>;
            type Named = $element<$(<$type as $crate::metadata::Metadata>::Named),*>;
            type Field = $element<$(<$type as $crate::metadata::Metadata>::Field),*>;
            type Variant = $element<$(<$type as $crate::metadata::Metadata>::Variant),*>;
        }
    };
}
