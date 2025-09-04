use name_resolution::resolve_names;

mod name_resolution;
mod recursion_check;

pub use name_resolution::ResolutionMetadata;

use crate::{
    ast::TypeSet, composite, preprocessing::recursion_check::check_recursion, reports::Reports,
    syntax::SpanMetadata,
};

pub fn preprocess<'filename>(
    types: TypeSet<SpanMetadata>,
    reports: &mut Reports<'filename>,
    filename: &'filename str,
) -> TypeSet<BasicMetadata> {
    let types = resolve_names(types, reports, filename);
    check_recursion(&types, reports, filename);

    types
}

composite! {
    pub struct (BasicInfo, BasicMetadata) {
        resolution: ResolutionMetadata | R,
        span: SpanMetadata | S
    }
}
