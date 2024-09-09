use chumsky::Parser;

use crate::{
    parser::NoirParser,
    token::{DocStyle, Token, TokenKind},
};

use super::primitives::token_kind;

fn outer_doc_comment() -> impl NoirParser<String> {
    token_kind(TokenKind::OuterDocComment).map(|token| match token {
        Token::LineComment(comment, Some(DocStyle::Outer)) => comment,
        Token::BlockComment(comment, Some(DocStyle::Outer)) => comment,
        _ => unreachable!(
            "Parser should have already errored due to token not being an outer doc comment"
        ),
    })
}

pub(super) fn outer_doc_comments() -> impl NoirParser<Vec<String>> {
    outer_doc_comment().repeated()
}

fn inner_doc_comment() -> impl NoirParser<String> {
    token_kind(TokenKind::InnerDocComment).map(|token| match token {
        Token::LineComment(comment, Some(DocStyle::Inner)) => comment,
        Token::BlockComment(comment, Some(DocStyle::Inner)) => comment,
        _ => unreachable!(
            "Parser should have already errored due to token not being an inner doc comment"
        ),
    })
}

pub(super) fn inner_doc_comments() -> impl NoirParser<Vec<String>> {
    inner_doc_comment().repeated()
}
