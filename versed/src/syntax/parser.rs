use chumsky::{
    ConfigParser, IterParser, Parser,
    error::Rich,
    extra,
    input::ValueInput,
    prelude::{any, choice, empty, end, just, recursive, skip_until, via_parser},
    select,
};

use crate::{
    ast::{
        Enum, Field, Identifier, NamedType, Primitive, PrimitiveType, Struct, Type, TypeSet,
        Variant,
    },
    syntax::{
        ExtendVec, IdentSpan, Span, SpanMetadata,
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

/// Matches any token or a bracketed expression (without semicolons),
/// but doesn't allow for semicolons, as those cannot appear within types
/// and are used to synchronize broken named types.
fn single_or_group<'tokens, I: Input<'tokens>>() -> Parser![()] {
    recursive(|single_or_group| {
        let single = any()
            .filter(|t| {
                !matches!(
                    t,
                    Token::GroupLeft(_) | Token::GroupRight(_) | Token::Punct(Punct::Semicolon)
                )
            })
            .ignored();

        let group = select! {
            Token::GroupLeft(group) => group
        }
        .then_ignore(single_or_group.repeated())
        .ignore_with_ctx(
            just(Token::Punct(Punct::Semicolon))
                .configure(|cfg, ctx| cfg.seq(Token::GroupRight(*ctx))),
        )
        .ignored();

        choice((single, group))
    })
}

const UNIT: Primitive<SpanMetadata> = Primitive {
    r#type: PrimitiveType::Unit,
    metadata: (),
};

pub fn parser<'tokens, I: Input<'tokens>>() -> Parser![TypeSet<SpanMetadata>] {
    let version = keyword(Keyword::Version)
        .ignore_then(
            ident()
                .recover_with(via_parser(single_or_group().map(|_| "".into())))
                .recover_with(via_parser(empty().map(|()| "".into())))
                .then_ignore(
                    punct(Punct::Semicolon)
                        .ignored()
                        .recover_with(via_parser(ident().rewind().to(()))),
                )
                .recover_with(via_parser(empty().map(|()| "".into()))),
        )
        .recover_with(via_parser(empty().map(|()| "".into())));

    let r#type = recursive(|r#type| {
        let parens = r#type
            .clone()
            .delimited_by(left(Group::Paren), right(Group::Paren));

        let primitive = choice([
            keyword(Keyword::Unit).to(PrimitiveType::Unit),
            keyword(Keyword::Int).to(PrimitiveType::Number),
            keyword(Keyword::String).to(PrimitiveType::String),
        ])
        .map(|r#type| {
            Type::Primitive(Primitive {
                r#type,
                metadata: (),
            })
        });

        let identifier = ident().map_with(|ident, e| {
            Type::Identifier(Identifier {
                ident,
                metadata: IdentSpan { span: e.span() },
            })
        });

        let list = r#type
            .clone()
            .delimited_by(left(Group::Bracket), right(Group::Bracket));

        fn composite<'tokens, I: Input<'tokens>, F, T>(
            leading_keyword: Keyword,
            map_field: impl Fn(String, Type<SpanMetadata>, Span) -> F + Clone,
            map_type: impl Fn(Vec<F>) -> T + Clone,
            r#type: Parser![Type<SpanMetadata>],
        ) -> Parser![T] {
            let field = ident()
                .map_with(|ident, e| (ident, e.span()))
                .then(
                    punct(Punct::Colon)
                        .ignore_then(r#type.clone())
                        .or(punct(Punct::Colon).not().to(Type::Primitive(UNIT))),
                )
                .map(move |((ident, span), r#type)| map_field(ident, r#type, span));

            let skip_to_comma = skip_until(
                single_or_group(),
                punct(Punct::Comma).rewind().ignored(),
                || None,
            );
            let skip_to_brace = via_parser(
                single_or_group()
                    .repeated()
                    .at_least(1)
                    .then_ignore(right(Group::Brace).rewind())
                    .map(|()| None),
            );

            let body = field
                .map(Some)
                .recover_with(skip_to_comma)
                .recover_with(skip_to_brace)
                .separated_by(
                    punct(Punct::Comma)
                        .ignored()
                        .recover_with(via_parser(right(Group::Brace).not())),
                )
                .allow_trailing()
                .collect()
                .map(|ExtendVec(inner)| inner)
                .delimited_by(left(Group::Brace), right(Group::Brace));

            keyword(leading_keyword).ignore_then(body).map(map_type)
        }

        let r#struct = composite(
            Keyword::Struct,
            |name, r#type, span| Field {
                name,
                r#type,
                metadata: IdentSpan { span },
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
            |name, r#type, span| Variant {
                name,
                r#type,
                metadata: IdentSpan { span },
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
        .map_with(|ident, e| (ident, e.span()))
        .then_ignore(punct(Punct::Equals))
        .then(r#type.clone().recover_with(skip_until(
            any().ignored(),
            punct(Punct::Semicolon).rewind().ignored().or(end()),
            || Type::Primitive(UNIT),
        )))
        .then_ignore(
            punct(Punct::Semicolon)
                .ignored()
                .recover_with(via_parser(empty())),
        )
        .map(|((name, span), r#type)| NamedType {
            name,
            r#type,
            metadata: IdentSpan { span },
        });

    let types = named_type
        .map(Some)
        .recover_with(skip_until(
            any().ignored(),
            punct(Punct::Semicolon).ignored(),
            || None,
        ))
        .recover_with(via_parser(
            any().repeated().at_least(1).then(end()).to(None),
        ))
        .repeated()
        .collect()
        .map(|ExtendVec(inner)| inner);

    version
        .then(types)
        .map(|(version, types)| TypeSet { version, types })
}
