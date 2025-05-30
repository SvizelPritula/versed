use icu_properties::props::{EnumeratedProperty, GeneralCategory, GeneralCategoryGroup};

use crate::idents::IdentRules;

pub struct CSharpIdentRules;

impl IdentRules for CSharpIdentRules {
    fn is_start_char(&self, ch: char) -> bool {
        const GROUP: GeneralCategoryGroup =
            GeneralCategoryGroup::Letter.union(GeneralCategoryGroup::LetterNumber);

        GROUP.contains(GeneralCategory::for_char(ch)) || ch == '_'
    }

    fn is_continue_char(&self, ch: char) -> bool {
        const GROUP: GeneralCategoryGroup = GeneralCategoryGroup::Letter
            .union(GeneralCategoryGroup::LetterNumber)
            .union(GeneralCategoryGroup::DecimalNumber)
            .union(GeneralCategoryGroup::ConnectorPunctuation)
            .union(GeneralCategoryGroup::NonspacingMark)
            .union(GeneralCategoryGroup::EnclosingMark)
            .union(GeneralCategoryGroup::Format);

        GROUP.contains(GeneralCategory::for_char(ch))
    }

    fn reserved_prefix(&self) -> &str {
        "@"
    }

    fn is_reserved(&self, str: &str) -> bool {
        matches!(
            str,
            "abstract"
                | "as"
                | "base"
                | "bool"
                | "break"
                | "byte"
                | "case"
                | "catch"
                | "char"
                | "checked"
                | "class"
                | "const"
                | "continue"
                | "decimal"
                | "default"
                | "delegate"
                | "do"
                | "double"
                | "else"
                | "enum"
                | "event"
                | "explicit"
                | "extern"
                | "false"
                | "finally"
                | "fixed"
                | "float"
                | "for"
                | "foreach"
                | "goto"
                | "if"
                | "implicit"
                | "in"
                | "int"
                | "interface"
                | "internal"
                | "is"
                | "lock"
                | "long"
                | "namespace"
                | "new"
                | "null"
                | "object"
                | "operator"
                | "out"
                | "override"
                | "params"
                | "private"
                | "protected"
                | "public"
                | "readonly"
                | "ref"
                | "return"
                | "sbyte"
                | "sealed"
                | "short"
                | "sizeof"
                | "stackalloc"
                | "static"
                | "string"
                | "struct"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "uint"
                | "ulong"
                | "unchecked"
                | "unsafe"
                | "ushort"
                | "using"
                | "virtual"
                | "void"
                | "volatile"
                | "while"
        )
    }
}
