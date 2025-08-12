use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    prelude::{any, choice, empty, just, none_of, via_parser},
    text::{digits, ident, whitespace},
};

use crate::syntax::{tokens::{Group, Keyword, Punct, Token}, ExtendVec, Span, Spanned};

pub type Error<'src> = Rich<'src, char, Span>;

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Spanned<Token>>, extra::Err<Error<'src>>> {
    let ident_like = ident().map(|s: &str| match s {
        "version" => Token::Keyword(Keyword::Version),
        "struct" => Token::Keyword(Keyword::Struct),
        "enum" => Token::Keyword(Keyword::Enum),
        "unit" => Token::Keyword(Keyword::Unit),
        "string" => Token::Keyword(Keyword::String),
        "int" => Token::Keyword(Keyword::Int),

        ident => Token::Ident(ident.to_owned()),
    });

    // Structure of string parsing inspired by Rust, code inspired by the official JSON example

    const REPLACEMENT_CHARACTER: char = '\u{FFFD}';

    let unicode_escape = digits(16)
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
        .delimited_by(
            just('{'),
            just('}').ignored().recover_with(via_parser(empty())),
        );

    let string_escape = just('\\').ignore_then(
        choice((
            just('\\'),
            just('\''),
            just('\"'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('u').ignore_then(unicode_escape),
        ))
        .recover_with(via_parser(any())),
    );

    let string_char = none_of("\\\"\r\n")
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
        .delimited_by(
            just('"'),
            just('"')
                .ignored()
                .recover_with(via_parser(none_of("\r\n").repeated())),
        );

    let punct_or_group = choice([
        just('=').to(Token::Punct(Punct::Equals)),
        just(':').to(Token::Punct(Punct::Colon)),
        just(',').to(Token::Punct(Punct::Comma)),
        just(';').to(Token::Punct(Punct::Semicolon)),
        just('(').to(Token::GroupLeft(Group::Paren)),
        just(')').to(Token::GroupRight(Group::Paren)),
        just('[').to(Token::GroupLeft(Group::Bracket)),
        just(']').to(Token::GroupRight(Group::Bracket)),
        just('{').to(Token::GroupLeft(Group::Brace)),
        just('}').to(Token::GroupRight(Group::Brace)),
    ]);

    let token = choice((ident_like, quoted_ident, punct_or_group));

    let comment = just("//").ignore_then(none_of("\r\n").repeated());

    let skip = whitespace().then(comment.then_ignore(whitespace()).repeated());

    let body = token
        .map_with(|tok, e| (tok, e.span()))
        .map(Some)
        .recover_with(via_parser(any().to(None)))
        .then_ignore(skip)
        .repeated()
        .collect()
        .map(|ExtendVec(inner)| inner);

    skip.ignore_then(body)
}
