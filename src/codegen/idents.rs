//! Utilities for case-converting and disambiguating identifiers.

use std::fmt::Write;

use icu_properties::props::{
    BinaryProperty, EnumeratedProperty, GeneralCategory, GeneralCategoryGroup, Lowercase, Uppercase,
};

/// Rules for valid identifiers in a target language.
pub trait IdentRules {
    /// Checks if a character may occur at the start of an identifier.
    ///
    /// Must be a subset of [`IdentRules::is_continue_char`] and allow `_`.
    fn is_start_char(&self, ch: char) -> bool;
    /// Checks if a character may occur in an identifier.
    ///
    /// Must allow `_` and `0` to `9`.
    fn is_continue_char(&self, ch: char) -> bool;
    /// Checks if `str` is a reserved identifier that *can* be escaped using [`IdentRules::reserved_prefix`].
    fn is_reserved(&self, str: &str) -> bool;
    /// A prefix that can be prepended to an identifier to make it not reserved.
    fn reserved_prefix(&self) -> &str;
    /// Checks if `str` is a reserved identifier that *can't* be escaped using [`IdentRules::reserved_prefix`].
    fn is_always_reserved(&self, _str: &str) -> bool {
        false
    }
}

/// A type of naming convention (camelCase, kebab-case, etc.).
pub trait CaseType {
    type Builder: CaseBuilder;
    /// Creates a [`CaseBuilder`] for this naming convention.
    fn builder(self) -> Self::Builder;
}

/// Converts letters and word boundaries to an identifier.
pub trait CaseBuilder {
    /// Adds a alphanumeric character to the identifier.
    fn add_letter(&mut self, ch: char);
    /// Adds a word boundary to the identifier.
    fn add_word_end(&mut self);
    /// Finishes building the identifier and extracts it.
    fn finish(self) -> String;
}

/// Converts a possibly multi-part identifier using a [`CaseType`] and [`IdentRules`].
pub fn convert_case<'a>(
    parts: impl IntoIterator<Item = &'a str>,
    case: impl CaseType,
    rules: impl IdentRules,
) -> String {
    let mut builder = case.builder();

    for part in parts {
        let mut prev_lowercase = false;
        let mut iter = part.chars().peekable();

        while let Some(ch) = iter.next() {
            let is_punct = !rules.is_continue_char(ch)
                || GeneralCategoryGroup::Punctuation.contains(GeneralCategory::for_char(ch));

            if is_punct {
                builder.add_word_end();
                continue;
            }

            let uppercase = Uppercase::for_char(ch);
            let next_lowercase = iter.peek().copied().is_some_and(Lowercase::for_char);

            if uppercase & (prev_lowercase | next_lowercase) {
                builder.add_word_end();
            }

            builder.add_letter(ch);
            prev_lowercase = Lowercase::for_char(ch);
        }

        builder.add_word_end();
    }

    let mut string = builder.finish();

    if string
        .chars()
        .next()
        .is_none_or(|ch| !rules.is_start_char(ch))
    {
        string.insert(0, '_');
    }

    if rules.is_always_reserved(&string) {
        string.push('_');
    }

    if rules.is_reserved(&string) {
        string.insert_str(0, rules.reserved_prefix());
    }

    string
}

/// Tries appending numeric suffixes to `ident` until `taken` returns `false`.
pub fn disambiguate(ident: &mut String, mut taken: impl FnMut(&str) -> bool) {
    if !taken(ident) {
        return;
    }

    let original_prefix = ident.len();
    for num in 2usize.. {
        write!(ident, "{num}").unwrap();

        if !taken(ident) {
            return;
        }

        ident.truncate(original_prefix);
    }
}

/// A builder for PascalCase and camelCase.
pub struct PascalCamelCaseBuilder {
    string: String,
    uppercase_pending: bool,
}

impl CaseBuilder for PascalCamelCaseBuilder {
    fn add_letter(&mut self, ch: char) {
        if self.uppercase_pending {
            self.string.extend(ch.to_uppercase());
        } else {
            self.string.extend(ch.to_lowercase());
        }

        self.uppercase_pending = false;
    }

    fn add_word_end(&mut self) {
        if !self.string.is_empty() {
            self.uppercase_pending = true;
        }
    }

    fn finish(self) -> String {
        self.string
    }
}

/// camelCase.
#[derive(Debug, Clone, Copy)]
pub struct CamelCase;
/// PascalCase.
#[derive(Debug, Clone, Copy)]
pub struct PascalCase;

impl CaseType for CamelCase {
    type Builder = PascalCamelCaseBuilder;

    fn builder(self) -> Self::Builder {
        PascalCamelCaseBuilder {
            string: String::new(),
            uppercase_pending: false,
        }
    }
}

impl CaseType for PascalCase {
    type Builder = PascalCamelCaseBuilder;

    fn builder(self) -> Self::Builder {
        PascalCamelCaseBuilder {
            string: String::new(),
            uppercase_pending: true,
        }
    }
}

/// A builder for snake_case and kebab-case.
pub struct SnakeKebabCaseBuilder {
    string: String,
    separator_pending: bool,
    separator: char,
}

impl CaseBuilder for SnakeKebabCaseBuilder {
    fn add_letter(&mut self, ch: char) {
        if self.separator_pending {
            self.string.push(self.separator);
            self.separator_pending = false;
        }

        self.string.extend(ch.to_lowercase());
    }

    fn add_word_end(&mut self) {
        if !self.string.is_empty() {
            self.separator_pending = true;
        }
    }

    fn finish(self) -> String {
        self.string
    }
}

/// snake_case.
#[derive(Debug, Clone, Copy)]
pub struct SnakeCase;
/// kebab-case.
#[derive(Debug, Clone, Copy)]
pub struct KebabCase;

impl CaseType for SnakeCase {
    type Builder = SnakeKebabCaseBuilder;

    fn builder(self) -> Self::Builder {
        SnakeKebabCaseBuilder {
            string: String::new(),
            separator: '_',
            separator_pending: false,
        }
    }
}

impl CaseType for KebabCase {
    type Builder = SnakeKebabCaseBuilder;

    fn builder(self) -> Self::Builder {
        SnakeKebabCaseBuilder {
            string: String::new(),
            separator: '-',
            separator_pending: false,
        }
    }
}
