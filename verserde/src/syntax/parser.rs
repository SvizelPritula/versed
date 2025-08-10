use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    input::ValueInput,
    prelude::{choice, empty, just, recursive},
    select,
};

use crate::{
    ast::{Enum, Field, NamedType, Primitive, Struct, Type, TypeSet, Variant},
    syntax::{
        Span,
        tokens::{Group, Keyword, Punct, Token},
    },
};

pub type Error<'tokens> = Rich<'tokens, Token, Span>;

macro_rules! Parser {
    [$type: ty] => {
        impl Parser<'tokens, I, $type, extra::Err<Error<'tokens>>> + Clone
    };
}

pub trait Input<'tokens>: ValueInput<'tokens, Token = Token, Span = Span> {}
impl<'tokens, I: ValueInput<'tokens, Token = Token, Span = Span>> Input<'tokens> for I {}

fn ident<'tokens, I: Input<'tokens>>() -> Parser![String] {
    select! {
        Token::Ident(ident) => ident,
        Token::QuotedIdent(ident) => ident,
    }
    .labelled("identifier")
}

fn keyword<'tokens, I: Input<'tokens>>(keyword: Keyword) -> Parser![Token] {
    just(Token::Keyword(keyword))
}
fn punct<'tokens, I: Input<'tokens>>(punct: Punct) -> Parser![Token] {
    just(Token::Punct(punct))
}
fn left<'tokens, I: Input<'tokens>>(group: Group) -> Parser![Token] {
    just(Token::GroupLeft(group))
}
fn right<'tokens, I: Input<'tokens>>(group: Group) -> Parser![Token] {
    just(Token::GroupRight(group))
}

pub fn parser<'tokens, I: Input<'tokens>>() -> Parser![TypeSet<()>] {
    let version = keyword(Keyword::Version)
        .ignore_then(ident())
        .then_ignore(punct(Punct::Semicolon));

    let r#type = recursive(|r#type| {
        let parens = r#type
            .clone()
            .delimited_by(left(Group::Paren), right(Group::Paren));

        let primitive = choice([
            keyword(Keyword::Unit).to(Type::Primitive(Primitive::Unit)),
            keyword(Keyword::Int).to(Type::Primitive(Primitive::Number)),
            keyword(Keyword::String).to(Type::Primitive(Primitive::String)),
        ]);

        let identifier = ident().map(Type::Identifier);

        let list = r#type
            .clone()
            .delimited_by(left(Group::Bracket), right(Group::Bracket));

        fn composite<'tokens, I: Input<'tokens>, F, T>(
            leading_keyword: Keyword,
            map_field: impl Fn((String, Type<()>)) -> F + Clone,
            map_type: impl Fn(Vec<F>) -> T + Clone,
            r#type: Parser![Type<()>],
        ) -> Parser![T] {
            let field = ident()
                .then(
                    punct(Punct::Colon)
                        .ignore_then(r#type.clone())
                        .or(empty().to(Type::Primitive(Primitive::Unit))),
                )
                .map(map_field);

            let body = field
                .separated_by(punct(Punct::Comma))
                .allow_trailing()
                .collect()
                .delimited_by(left(Group::Brace), right(Group::Brace));

            keyword(leading_keyword).ignore_then(body).map(map_type)
        }

        let r#struct = composite(
            Keyword::Struct,
            |(name, r#type)| Field {
                name,
                r#type,
                metadata: (),
            },
            |fields| {
                Type::Struct(Struct {
                    fields,
                    metadata: (),
                })
            },
            r#type.clone(),
        );

        let r#enum = composite(
            Keyword::Enum,
            |(name, r#type)| Variant {
                name,
                r#type,
                metadata: (),
            },
            |variants| {
                Type::Enum(Enum {
                    variants,
                    metadata: (),
                })
            },
            r#type.clone(),
        );

        choice((parens, list, r#struct, r#enum, primitive, identifier))
    });

    let named_type = ident()
        .then_ignore(punct(Punct::Equals))
        .then(r#type.clone())
        .then_ignore(punct(Punct::Semicolon))
        .map(|(name, r#type)| NamedType {
            name,
            r#type,
            metadata: (),
        });

    let types = named_type.repeated().collect();

    version
        .then(types)
        .map(|(version, types)| TypeSet { version, types })
}
