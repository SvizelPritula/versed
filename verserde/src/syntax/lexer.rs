use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    prelude::{any, choice, just, none_of, via_parser},
    select,
    text::{digits, ident},
};

use crate::syntax::{Span, Spanned};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Ident(String),
    QuotedIdent(String),

    GroupLeft(Group),
    GroupRight(Group),

    Punct(Punct),
    Keyword(Keyword),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Group {
    Paren,
    Bracket,
    Brace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Punct {
    Equals,
    Colon,
    Comma,
    Semicolon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Keyword {
    Version,
    Struct,
    Enum,
    Unit,
    String,
    Int,
}

pub fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token>>, extra::Err<Rich<'src, char, Span>>> {
    let ident_like = ident().map(|s: &str| match s {
        "version" => Token::Keyword(Keyword::Version),
        "struct" => Token::Keyword(Keyword::Struct),
        "enum" => Token::Keyword(Keyword::Enum),
        "unit" => Token::Keyword(Keyword::Unit),
        "string" => Token::Keyword(Keyword::String),
        "int" => Token::Keyword(Keyword::Int),

        ident => Token::Ident(ident.to_owned()),
    });

    const REPLACEMENT_CHARACTER: char = '\u{FFFD}';

    // Structure inspired by Rust, code inspired by the official JSON example
    let string_escape = just('\\').ignore_then(
        choice((
            just('\\'),
            just('\''),
            just('\"'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('u').ignore_then(
                digits(16)
                    .at_least(1)
                    .at_most(6)
                    .to_slice()
                    .validate(|digits, e, emitter| {
                        let code = u32::from_str_radix(digits, 16).unwrap();
                        if let Some(c) = char::from_u32(code) {
                            c
                        } else {
                            emitter.emit(Rich::custom(
                                e.span(),
                                format!("there is no unicode character with code 0x{code:x}",),
                            ));

                            REPLACEMENT_CHARACTER
                        }
                    })
                    .recover_with(via_parser(
                        none_of("}\\\"").repeated().to(REPLACEMENT_CHARACTER),
                    ))
                    .delimited_by(just('{'), just('}')),
            ),
        ))
        .recover_with(via_parser(any())),
    );

    let string_char = none_of("\\\"")
        .validate(|c: char, e, emitter| {
            if c.is_control() {
                emitter.emit(Rich::custom(
                    e.span(),
                    format!(
                        "control character with code 0x{:x} found in quoted identifier",
                        c as u32
                    ),
                ))
            }

            c
        })
        .labelled("non-control character")
        .or(string_escape);

    let quoted_ident = string_char
        .repeated()
        .collect()
        .map(Token::QuotedIdent)
        .delimited_by(just('"'), just('"'));

    let punct_or_group = select! {
        '=' => Token::Punct(Punct::Equals),
        ':' => Token::Punct(Punct::Colon),
        ',' => Token::Punct(Punct::Comma),
        ';' => Token::Punct(Punct::Semicolon),
        '(' => Token::GroupLeft(Group::Paren),
        ')' => Token::GroupRight(Group::Paren),
        '[' => Token::GroupLeft(Group::Bracket),
        ']' => Token::GroupRight(Group::Bracket),
        '{' => Token::GroupLeft(Group::Brace),
        '}' => Token::GroupRight(Group::Brace),
    };

    let token = choice((ident_like, quoted_ident, punct_or_group));

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded()
        .repeated()
        .collect()
        .padded()
}
