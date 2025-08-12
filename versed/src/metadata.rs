use std::fmt::Debug;

pub trait Metadata: Clone {
    type Struct: Debug + Clone;
    type Enum: Debug + Clone;
    type Identifier: Debug + Clone;

    type Name: Debug + Clone;
    type Field: Debug + Clone;
    type Variant: Debug + Clone;
}

impl Metadata for () {
    type Struct = ();
    type Enum = ();
    type Identifier = ();

    type Name = ();
    type Field = ();
    type Variant = ();
}
