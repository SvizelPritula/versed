use icu_properties::props::{BinaryProperty, XidContinue, XidStart};

use crate::codegen::{
    idents::{CaseType, IdentRules, PascalCase, SnakeCase},
    naming_pass::NamingRules,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct RustIdentRules;

impl IdentRules for RustIdentRules {
    fn is_start_char(&self, ch: char) -> bool {
        XidStart::for_char(ch) || ch == '_'
    }

    fn is_continue_char(&self, ch: char) -> bool {
        XidContinue::for_char(ch)
    }

    #[rustfmt::skip]
    fn is_reserved(&self, str: &str) -> bool {
        matches!(
            str,
            "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern"
            | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod"
            | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct"
            | "super" | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async"
            | "await" | "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro"
            | "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" | "gen"
        )
    }

    fn is_always_reserved(&self, str: &str) -> bool {
        matches!(str, "crate" | "self" | "super" | "Self" | "_")
    }

    fn reserved_prefix(&self) -> &str {
        "r#"
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RustModIdentRules;

impl IdentRules for RustModIdentRules {
    fn is_start_char(&self, ch: char) -> bool {
        ch.is_ascii() && RustIdentRules.is_start_char(ch)
    }

    fn is_continue_char(&self, ch: char) -> bool {
        ch.is_ascii() && RustIdentRules.is_continue_char(ch)
    }

    fn is_reserved(&self, str: &str) -> bool {
        RustIdentRules.is_reserved(str)
    }

    fn is_always_reserved(&self, str: &str) -> bool {
        RustIdentRules.is_always_reserved(str)
    }

    fn reserved_prefix(&self) -> &str {
        RustIdentRules.reserved_prefix()
    }
}

pub struct RustNamingRules;

impl NamingRules for RustNamingRules {
    fn ident_rules(&self) -> impl IdentRules {
        RustIdentRules
    }
    fn type_case(&self) -> impl CaseType {
        PascalCase
    }
    fn field_case(&self) -> impl CaseType {
        SnakeCase
    }
    fn variant_case(&self) -> impl CaseType {
        PascalCase
    }
    fn version_case(&self) -> impl CaseType {
        SnakeCase
    }
    fn version_ident_rules(&self) -> impl IdentRules {
        RustModIdentRules
    }
}
