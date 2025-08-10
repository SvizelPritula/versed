use std::fmt::{self, Display, Formatter, Write};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Ident(String),
    QuotedIdent(String),

    GroupLeft(Group),
    GroupRight(Group),

    Punct(Punct),
    Keyword(Keyword),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Group {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Punct {
    Equals,
    Colon,
    Comma,
    Semicolon,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Keyword {
    Version,
    Struct,
    Enum,
    Unit,
    String,
    Int,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(ident) | Token::QuotedIdent(ident) => f.write_str(&ident),
            Token::GroupLeft(Group::Paren) => f.write_char('('),
            Token::GroupLeft(Group::Bracket) => f.write_char('['),
            Token::GroupLeft(Group::Brace) => f.write_char('{'),
            Token::GroupRight(Group::Paren) => f.write_char(')'),
            Token::GroupRight(Group::Bracket) => f.write_char(']'),
            Token::GroupRight(Group::Brace) => f.write_char('}'),
            Token::Punct(Punct::Equals) => f.write_char('='),
            Token::Punct(Punct::Colon) => f.write_char(':'),
            Token::Punct(Punct::Comma) => f.write_char(','),
            Token::Punct(Punct::Semicolon) => f.write_char(';'),
            Token::Keyword(Keyword::Version) => f.write_str("version"),
            Token::Keyword(Keyword::Struct) => f.write_str("struct"),
            Token::Keyword(Keyword::Enum) => f.write_str("enum"),
            Token::Keyword(Keyword::Unit) => f.write_str("unit"),
            Token::Keyword(Keyword::String) => f.write_str("string"),
            Token::Keyword(Keyword::Int) => f.write_str("int"),
        }
    }
}
