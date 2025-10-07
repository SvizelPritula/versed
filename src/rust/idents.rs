use icu_properties::props::{BinaryProperty, XidContinue, XidStart};

use crate::codegen::{
    idents::{IdentRules, PascalCase, SnakeCase},
    naming_pass::{NamingRule, NamingRules},
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
            | "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern"
            | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod"
            | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct"
            | "super" | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async"
            | "await" | "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro"
            | "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" | "gen"
            | "_"
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
    fn r#type(&self) -> impl NamingRule {
        (PascalCase, RustIdentRules)
    }
    fn field(&self) -> impl NamingRule {
        (SnakeCase, RustIdentRules)
    }
    fn variant(&self) -> impl NamingRule {
        (PascalCase, RustIdentRules)
    }
    fn version(&self) -> impl NamingRule {
        (SnakeCase, RustModIdentRules)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RustMigrationSuffixIdentRules;

impl IdentRules for RustMigrationSuffixIdentRules {
    fn is_start_char(&self, ch: char) -> bool {
        RustIdentRules.is_start_char(ch)
    }

    fn is_continue_char(&self, ch: char) -> bool {
        RustIdentRules.is_continue_char(ch)
    }

    fn is_reserved(&self, _str: &str) -> bool {
        false
    }

    fn reserved_prefix(&self) -> &str {
        unreachable!("there are no reserved keywords starting with \"upgrade_\" or \"downgrade_\"")
    }
}

pub struct RustMigrationSuffixNamingRules;

impl NamingRules for RustMigrationSuffixNamingRules {
    fn r#type(&self) -> impl NamingRule {
        (SnakeCase, RustMigrationSuffixIdentRules)
    }

    // These are not actually used.
    // It would be nice to redesing the naming pass to be modular.
    fn field(&self) -> impl NamingRule {
        (SnakeCase, RustMigrationSuffixIdentRules)
    }
    fn variant(&self) -> impl NamingRule {
        (SnakeCase, RustMigrationSuffixIdentRules)
    }
    fn version(&self) -> impl NamingRule {
        (SnakeCase, RustMigrationSuffixIdentRules)
    }
}
