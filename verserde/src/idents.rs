use icu_properties::props::{
    BinaryProperty, EnumeratedProperty, GeneralCategory, GeneralCategoryGroup, Lowercase, Uppercase,
};

pub trait IdentRules {
    fn is_start_char(&self, ch: char) -> bool;
    fn is_continue_char(&self, ch: char) -> bool;
    fn is_reserved(&self, str: &str) -> bool;
    fn reserved_prefix(&self) -> &str;
    fn is_always_reserved(&self, _str: &str) -> bool {
        false
    }
}

pub trait CaseType {
    type Builder: CaseBuilder;
    fn builder(self) -> Self::Builder;
}

pub trait CaseBuilder {
    fn add_letter(&mut self, ch: char);
    fn add_word_end(&mut self);
    fn finish(self) -> String;
}

pub fn convert_case(ident: &str, case: impl CaseType, rules: impl IdentRules) -> String {
    let mut builder = case.builder();
    let mut prev_lowercase = false;

    for ch in ident.chars() {
        if !rules.is_continue_char(ch) {
            builder.add_word_end();
            continue;
        }

        if GeneralCategoryGroup::Punctuation.contains(GeneralCategory::for_char(ch)) {
            builder.add_word_end();
            continue;
        }

        let uppercase = Uppercase::for_char(ch);
        if uppercase & prev_lowercase {
            builder.add_word_end();
        }

        builder.add_letter(ch);

        prev_lowercase = Lowercase::for_char(ch);
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

pub struct PascalCamelCaseBuilder {
    string: String,
    word_start_pending: bool,
}

impl CaseBuilder for PascalCamelCaseBuilder {
    fn add_letter(&mut self, ch: char) {
        if self.word_start_pending {
            self.string.extend(ch.to_uppercase());
        } else {
            self.string.extend(ch.to_lowercase());
        }

        self.word_start_pending = false;
    }

    fn add_word_end(&mut self) {
        self.word_start_pending = true;
    }

    fn finish(self) -> String {
        self.string
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CamelCase;
#[derive(Debug, Clone, Copy)]
pub struct PascalCase;

impl CaseType for CamelCase {
    type Builder = PascalCamelCaseBuilder;

    fn builder(self) -> Self::Builder {
        PascalCamelCaseBuilder {
            string: String::new(),
            word_start_pending: false,
        }
    }
}

impl CaseType for PascalCase {
    type Builder = PascalCamelCaseBuilder;

    fn builder(self) -> Self::Builder {
        PascalCamelCaseBuilder {
            string: String::new(),
            word_start_pending: true,
        }
    }
}
