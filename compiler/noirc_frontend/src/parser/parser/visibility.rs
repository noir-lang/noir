use chumsky::{
    prelude::{choice, empty, just},
    Parser,
};

use crate::{
    ast::{ItemVisibility, Visibility},
    parser::NoirParser,
    token::{Keyword, Token},
};

use super::{call_data, primitives::keyword};

/// visibility_modifier: 'pub(crate)'? 'pub'? ''
pub(crate) fn item_visibility() -> impl NoirParser<ItemVisibility> {
    let is_pub_crate = (keyword(Keyword::Pub)
        .then_ignore(just(Token::LeftParen))
        .then_ignore(keyword(Keyword::Crate))
        .then_ignore(just(Token::RightParen)))
    .map(|_| ItemVisibility::PublicCrate);

    let is_pub = keyword(Keyword::Pub).map(|_| ItemVisibility::Public);

    let is_private = empty().map(|_| ItemVisibility::Private);

    choice((is_pub_crate, is_pub, is_private))
}

pub fn visibility() -> impl NoirParser<Visibility> {
    keyword(Keyword::Pub)
        .map(|_| Visibility::Public)
        .or(call_data())
        .or(keyword(Keyword::ReturnData).map(|_| Visibility::ReturnData))
        .or_not()
        .map(|opt| opt.unwrap_or(Visibility::Private))
}
