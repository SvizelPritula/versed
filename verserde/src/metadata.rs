use std::fmt::Debug;

pub trait Metadata {
    type Name: Debug + Clone;

    type Struct: Debug + Clone;
    type Field: Debug + Clone;

    type Enum: Debug + Clone;
    type Variant: Debug + Clone;
}

impl Metadata for () {
    type Name = ();

    type Struct = ();
    type Field = ();

    type Enum = ();
    type Variant = ();
}
