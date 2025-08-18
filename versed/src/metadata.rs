use std::fmt::Debug;

pub trait Metadata: Clone {
    type Struct: Debug + Clone;
    type Enum: Debug + Clone;
    type List: Debug + Clone;
    type Primitive: Debug + Clone;
    type Identifier: Debug + Clone;

    type Name: Debug + Clone;
    type Field: Debug + Clone;
    type Variant: Debug + Clone;
}

impl Metadata for () {
    type Struct = ();
    type Enum = ();
    type List = ();
    type Primitive = ();
    type Identifier = ();

    type Name = ();
    type Field = ();
    type Variant = ();
}

pub trait MapMetadata<A: Metadata, B: Metadata, R: Metadata> {
    fn map_struct(left: A::Struct, right: B::Struct) -> R::Struct;
    fn map_enum(left: A::Enum, right: B::Enum) -> R::Enum;
    fn map_identifier(left: A::Identifier, right: B::Identifier) -> R::Identifier;

    fn map_name(left: A::Name, right: B::Name) -> R::Name;
    fn map_field(left: A::Field, right: B::Field) -> R::Field;
    fn map_variant(left: A::Variant, right: B::Variant) -> R::Variant;
}

pub trait GetMetadata<A: Metadata, R: Metadata> {
    fn get_struct(metadata: &A::Struct) -> &R::Struct;
    fn get_enum(metadata: &A::Enum) -> &R::Enum;
    fn get_identifier(metadata: &A::Identifier) -> &R::Identifier;

    fn get_name(metadata: &A::Name) -> &R::Name;
    fn get_field(metadata: &A::Field) -> &R::Field;
    fn get_variant(metadata: &A::Variant) -> &R::Variant;
}

macro_rules! mapper {
    {fn $name: ident($left_var: ident: $left: ty, $right_var: ident: $right: ty) -> $result: ty $body: block} => {
        struct $name;

        impl MapMetadata<$left, $right, $result> for $name {
            fn map_struct($left_var: <$left as Metadata>::Struct, $right_var: <$right as Metadata>::Struct) -> <$result as Metadata>::Struct $body
            fn map_enum($left_var: <$left as Metadata>::Enum, $right_var: <$right as Metadata>::Enum) -> <$result as Metadata>::Enum $body
            fn map_identifier($left_var: <$left as Metadata>::Identifier, $right_var: <$right as Metadata>::Identifier) -> <$result as Metadata>::Identifier $body

            fn map_name($left_var: <$left as Metadata>::Name, $right_var: <$right as Metadata>::Name) -> <$result as Metadata>::Name $body
            fn map_field($left_var: <$left as Metadata>::Field, $right_var: <$right as Metadata>::Field) -> <$result as Metadata>::Field $body
            fn map_variant($left_var: <$left as Metadata>::Variant, $right_var: <$right as Metadata>::Variant) -> <$result as Metadata>::Variant $body
        }
    };
}

macro_rules! getter {
    {fn $name: ident($metadata_var: ident: $metadata: ty) -> $result: ty $body: block} => {
        struct $name;

        impl GetMetadata<$metadata, $result> for $name {
            fn get_struct($metadata_var: <$metadata as Metadata>::Struct) -> <$result as Metadata>::Struct $body
            fn get_enum($metadata_var: <$metadata as Metadata>::Enum) -> <$result as Metadata>::Enum $body
            fn get_identifier($metadata_var: <$metadata as Metadata>::Identifier) -> <$result as Metadata>::Identifier $body

            fn get_name($metadata_var: <$metadata as Metadata>::Name) -> <$result as Metadata>::Name $body
            fn get_field($metadata_var: <$metadata as Metadata>::Field) -> <$result as Metadata>::Field $body
            fn get_variant($metadata_var: <$metadata as Metadata>::Variant) -> <$result as Metadata>::Variant $body
        }
    };
}
