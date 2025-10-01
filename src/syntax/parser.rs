use std::{borrow::Cow, str::FromStr};

use chumsky::{
    ConfigParser, IterParser, Parser,
    error::Rich,
    extra,
    input::ValueInput,
    prelude::{any, choice, empty, end, just, recursive, skip_until, via_parser},
    select,
};
use icu_normalizer::ComposingNormalizerBorrowed;

use crate::{
    ast::{
        Enum, Field, Identifier, List, NamedType, Primitive, PrimitiveType, Struct, Type, TypeSet,
        TypeType, Variant,
    },
    syntax::{
        ExtendVec, MemberSpanInfo, Span, SpanMetadata, TypeSetSpanInfo, TypeSpanInfo,
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

fn normalize(ident: String) -> String {
    const NORMALIZER: ComposingNormalizerBorrowed<'static> = ComposingNormalizerBorrowed::new_nfc();

    match NORMALIZER.normalize(&ident) {
        Cow::Borrowed(_) => ident,
        Cow::Owned(string) => string,
    }
}

fn ident<'tokens, I: Input<'tokens>>() -> Parser![String] {
    select! {
        Token::Ident(ident) => ident,
        Token::QuotedIdent(ident) => ident,
    }
    .map(normalize)
    .labelled("identifier")
}

fn number<'tokens, I: Input<'tokens>>() -> Parser![Option<u64>] {
    select! {
        Token::Number(string) => string,
    }
    .validate(|digits, e, emitter| match u64::from_str(&digits) {
        Ok(n) => Some(n),
        Err(_) => {
            // The only possible error kind should be PosOverflow
            emitter.emit(Rich::custom(
                e.span(),
                format!("type numbers must be smaller than {}", u64::MAX),
            ));

            None
        }
    })
    .labelled("number")
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

fn type_number<'tokens, I: Input<'tokens>>() -> Parser![Option<(u64, Span)>] {
    punct(Punct::Pound)
        .ignore_then(number().recover_with(via_parser(empty().to(None))))
        .or_not()
        .map(Option::flatten)
        .map_with(|n, e| n.map(|n| (n, e.span())))
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
    let to_default_version = |()| (String::new(), Span::from(0..0));

    let version = keyword(Keyword::Version)
        .ignore_then(
            ident()
                .map_with(|ident, e| (ident, e.span()))
                .recover_with(via_parser(single_or_group().map(to_default_version)))
                .recover_with(via_parser(empty().map(to_default_version)))
                .then_ignore(
                    punct(Punct::Semicolon)
                        .ignored()
                        .recover_with(via_parser(ident().rewind().to(()))),
                )
                .recover_with(via_parser(empty().map(to_default_version))),
        )
        .recover_with(via_parser(empty().map(to_default_version)));

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
            TypeType::Primitive(Primitive {
                r#type,
                metadata: (),
            })
        });

        let identifier = ident().map(|ident| {
            TypeType::Identifier(Identifier {
                ident,
                metadata: (),
            })
        });

        let list = r#type
            .clone()
            .delimited_by(left(Group::Bracket), right(Group::Bracket))
            .map(|r#type| {
                TypeType::List(List {
                    r#type: Box::new(r#type),
                    metadata: (),
                })
            });

        fn composite<'tokens, I: Input<'tokens>, F, T>(
            leading_keyword: Keyword,
            map_field: impl Fn(String, Type<SpanMetadata>, Span) -> F + Clone,
            map_type: impl Fn(Vec<F>) -> T + Clone,
            r#type: Parser![Type<SpanMetadata>],
        ) -> Parser![T] {
            let field = ident()
                .map_with(|ident, e| (ident, e.span()))
                .then_with_ctx(
                    punct(Punct::Colon)
                        .ignore_then(r#type.clone())
                        .with_ctx(())
                        .or(punct(Punct::Colon)
                            .not()
                            .ignore_then(type_number())
                            .with_ctx(())
                            .map_with(|number, e| Type {
                                r#type: TypeType::Primitive(UNIT),
                                number: number.map(|(number, _)| number),
                                metadata: TypeSpanInfo {
                                    r#type: {
                                        let (_, Span { end, .. }) = e.ctx();
                                        Span::from(*end..*end)
                                    },
                                    number: number.map(|(_, span)| span),
                                },
                            })),
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
                metadata: MemberSpanInfo { name: span },
            },
            |fields| {
                TypeType::Struct(Struct {
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
                metadata: MemberSpanInfo { name: span },
            },
            |variants| {
                TypeType::Enum(Enum {
                    variants,
                    metadata: (),
                })
            },
            r#type.clone(),
        );

        let real_type = type_number()
            .then(
                choice((list, r#struct, r#enum, primitive, identifier))
                    .map_with(|r#type, e| (r#type, e.span())),
            )
            .map(|(number, (r#type, span))| Type {
                r#type,
                number: number.map(|(number, _)| number),
                metadata: TypeSpanInfo {
                    r#type: span,
                    number: number.map(|(_, span)| span),
                },
            });

        choice((parens, real_type))
    });

    let named_type = ident()
        .map_with(|ident, e| (ident, e.span()))
        .then_ignore(punct(Punct::Equals))
        .then(r#type.clone().recover_with(skip_until(
            any().ignored(),
            punct(Punct::Semicolon).rewind().ignored().or(end()),
            || Type {
                r#type: TypeType::Primitive(UNIT),
                number: None,
                metadata: TypeSpanInfo {
                    r#type: Span::from(0..0),
                    number: None,
                },
            },
        )))
        .then_ignore(
            punct(Punct::Semicolon)
                .ignored()
                .recover_with(via_parser(empty())),
        )
        .map(|((name, span), r#type)| NamedType {
            name,
            r#type,
            metadata: MemberSpanInfo { name: span },
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
        .map(|((version, version_span), types)| TypeSet {
            version,
            types,
            metadata: TypeSetSpanInfo {
                version: version_span,
            },
        })
}
