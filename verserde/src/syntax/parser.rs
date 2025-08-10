use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    input::ValueInput,
    prelude::{choice, empty, just, recursive},
    select,
};

use crate::{
    ast::{Enum, Field, NamedType, Primitive, Struct, Type, Variant},
    syntax::{
        Span,
        tokens::{Group, Keyword, Punct, Token},
    },
};

pub type Error<'tokens> = Rich<'tokens, Token, Span>;

pub fn parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (String, Vec<NamedType<()>>), extra::Err<Error<'tokens>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let ident = select! {
        Token::Ident(ident) => ident,
        Token::QuotedIdent(ident) => ident,
    }
    .labelled("identifier");

    let keyword = |keyword| just(Token::Keyword(keyword));
    let punct = |punct| just(Token::Punct(punct));
    let left = |group| just(Token::GroupLeft(group));
    let right = |group| just(Token::GroupRight(group));

    let version = keyword(Keyword::Version)
        .ignore_then(ident)
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

        let identifier = ident.map(Type::Identifier);

        let list = r#type
            .clone()
            .delimited_by(left(Group::Bracket), right(Group::Bracket));

        macro_rules! composite {
            ($keyword: path, $type: ident, $field: ident, $fields_name: ident) => {{
                let field = ident
                    .then(
                        punct(Punct::Colon)
                            .ignore_then(r#type.clone())
                            .or(empty().to(Type::Primitive(Primitive::Unit))),
                    )
                    .map(|(name, r#type)| $field {
                        name,
                        r#type,
                        metadata: (),
                    });

                let body = field
                    .separated_by(punct(Punct::Comma))
                    .allow_trailing()
                    .collect()
                    .delimited_by(left(Group::Brace), right(Group::Brace));

                keyword($keyword).ignore_then(body).map(|$fields_name| {
                    Type::$type($type {
                        $fields_name,
                        metadata: (),
                    })
                })
            }};
        }

        let r#struct = composite!(Keyword::Struct, Struct, Field, fields);
        let r#enum = composite!(Keyword::Enum, Enum, Variant, variants);

        choice((parens, list, r#struct, r#enum, primitive, identifier))
    });

    let named_type = ident
        .then_ignore(punct(Punct::Equals))
        .then(r#type.clone())
        .then_ignore(punct(Punct::Semicolon))
        .map(|(name, r#type)| NamedType {
            name,
            r#type,
            metadata: (),
        });

    let types = named_type.repeated().collect();

    version.then(types)
}
