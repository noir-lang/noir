use acvm::FieldElement;
use noirc_errors::{Position, Span, Spanned};
use noirc_frontend::token::IntType;

#[derive(Debug)]
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
    /// =
    Assign,
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
    /// ;
    Semicolon,
    /// ->
    Arrow,
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
    As,
    Brillig,
    Call,
    Cast,
    Inline,
    InlineAlways,
    Else,
    Field,
    Fold,
    Fn,
    Jmp,
    Jmpif,
    NoPredicates,
    Of,
    Return,
    Then,
}

impl Keyword {
    pub(crate) fn lookup_keyword(word: &str) -> Option<Token> {
        let keyword = match word {
            "acir" => Keyword::Acir,
            "as" => Keyword::As,
            "brillig" => Keyword::Brillig,
            "call" => Keyword::Call,
            "cast" => Keyword::Cast,
            "else" => Keyword::Else,
            "inline" => Keyword::Inline,
            "inline_always" => Keyword::InlineAlways,
            "Field" => Keyword::Field,
            "fold" => Keyword::Fold,
            "fn" => Keyword::Fn,
            "jmp" => Keyword::Jmp,
            "jmpif" => Keyword::Jmpif,
            "no_predicates" => Keyword::NoPredicates,
            "of" => Keyword::Of,
            "return" => Keyword::Return,
            "then" => Keyword::Then,
            _ => return None,
        };
        Some(Token::Keyword(keyword))
    }
}
