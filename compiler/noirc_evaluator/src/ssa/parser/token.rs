use acvm::FieldElement;
use noirc_errors::{Position, Span, Spanned};
use noirc_frontend::token::IntType;

pub(crate) struct SpannedToken(Spanned<Token>);

impl SpannedToken {
    pub(crate) fn new(token: Token, span: Span) -> SpannedToken {
        SpannedToken(Spanned::from(span, token))
    }

    pub(crate) fn to_span(&self) -> Span {
        self.0.span()
    }

    pub(crate) fn token(&self) -> &Token {
        &self.0.contents
    }

    pub(crate) fn into_token(self) -> Token {
        self.0.contents
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Token {
    Ident(String),
    Int(FieldElement),
    Keyword(Keyword),
    IntType(IntType),
    /// (
    LeftParen,
    /// )
    RightParen,
    /// {
    LeftBrace,
    /// }
    RightBrace,
    /// [
    LeftBracket,
    /// ]
    RightBracket,
    /// ,
    Comma,
    /// :
    Colon,
    Eof,
}

impl Token {
    pub(super) fn into_single_span(self, position: Position) -> SpannedToken {
        self.into_span(position, position)
    }

    pub(super) fn into_span(self, start: Position, end: Position) -> SpannedToken {
        SpannedToken(Spanned::from_position(start, end, self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Keyword {
    Acir,
    Brillig,
    Inline,
    InlineAlways,
    Field,
    Fold,
    Fn,
    NoPredicates,
    Of,
    Return,
}

impl Keyword {
    pub(crate) fn lookup_keyword(word: &str) -> Option<Token> {
        let keyword = match word {
            "acir" => Keyword::Acir,
            "brillig" => Keyword::Brillig,
            "inline" => Keyword::Inline,
            "inline_always" => Keyword::InlineAlways,
            "Field" => Keyword::Field,
            "fold" => Keyword::Fold,
            "fn" => Keyword::Fn,
            "no_predicates" => Keyword::NoPredicates,
            "of" => Keyword::Of,
            "return" => Keyword::Return,
            _ => return None,
        };
        Some(Token::Keyword(keyword))
    }
}
