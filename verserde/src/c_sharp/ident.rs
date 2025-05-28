use icu_properties::props::{EnumeratedProperty, GeneralCategory, GeneralCategoryGroup};

pub fn is_start_char(c: char) -> bool {
    const GROUP: GeneralCategoryGroup =
        GeneralCategoryGroup::Letter.union(GeneralCategoryGroup::LetterNumber);

    GROUP.contains(GeneralCategory::for_char(c)) || c == '_'
}

pub fn is_continue_char(c: char) -> bool {
    const GROUP: GeneralCategoryGroup = GeneralCategoryGroup::Letter
        .union(GeneralCategoryGroup::LetterNumber)
        .union(GeneralCategoryGroup::DecimalNumber)
        .union(GeneralCategoryGroup::ConnectorPunctuation)
        .union(GeneralCategoryGroup::NonspacingMark)
        .union(GeneralCategoryGroup::EnclosingMark)
        .union(GeneralCategoryGroup::Format);

    GROUP.contains(GeneralCategory::for_char(c))
}
