use icu_properties::props::{BinaryProperty, IdContinue, IdStart};

use crate::codegen::{
    idents::{CamelCase, IdentRules, KebabCase, PascalCase},
    naming_pass::{NamingRule, NamingRules},
};

fn is_start_char(ch: char) -> bool {
    IdStart::for_char(ch) || ch == '_' || ch == '$'
}

fn is_continue_char(ch: char) -> bool {
    IdContinue::for_char(ch) || ch == '_' || ch == '$'
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TypeScriptTypeIdentRules;

impl IdentRules for TypeScriptTypeIdentRules {
    fn is_start_char(&self, ch: char) -> bool {
        is_start_char(ch)
    }

    fn is_continue_char(&self, ch: char) -> bool {
        is_continue_char(ch)
    }

    fn is_reserved(&self, _str: &str) -> bool {
        false
    }

    #[rustfmt::skip]
    fn is_always_reserved(&self, str: &str) -> bool {
        matches!(
            str,
            | "break" | "case" | "catch" | "class" | "const" | "continue" | "debugger" | "default"
            | "delete" | "do" | "else" | "enum" | "export" | "extends" | "false" | "finally"
            | "for" | "function" | "if" | "import" | "in" | "instanceof" | "new" | "null"
            | "return" | "super" | "switch" | "this" | "throw" | "true" | "try" | "typeof" | "var"
            | "void" | "while" | "with" | "any" | "boolean" | "never" | "number" | "object"
            | "string" | "symbol" | "undefined" | "unknown" | "bigint"
        )
    }

    fn reserved_prefix(&self) -> &str {
        unreachable!("TypeScript has no escapable keywords")
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TypeScriptMemberIdentRules;

impl IdentRules for TypeScriptMemberIdentRules {
    fn is_start_char(&self, ch: char) -> bool {
        is_start_char(ch)
    }

    fn is_continue_char(&self, ch: char) -> bool {
        is_continue_char(ch)
    }

    fn is_reserved(&self, _str: &str) -> bool {
        false
    }

    fn is_always_reserved(&self, _str: &str) -> bool {
        false
    }

    fn reserved_prefix(&self) -> &str {
        unreachable!("TypeScript has keywords in member positions")
    }
}

pub struct TypeScriptNamingRules;

impl NamingRules for TypeScriptNamingRules {
    fn r#type(&self) -> impl NamingRule {
        (PascalCase, TypeScriptTypeIdentRules)
    }
    fn field(&self) -> impl NamingRule {
        (CamelCase, TypeScriptMemberIdentRules)
    }
    fn variant(&self) -> impl NamingRule {
        (KebabCase, TypeScriptMemberIdentRules)
    }
    fn version(&self) -> impl NamingRule {
        (CamelCase, TypeScriptTypeIdentRules)
    }
}
