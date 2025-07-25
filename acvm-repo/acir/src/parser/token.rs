use acir_field::FieldElement;
use noirc_errors::{Position, Span, Spanned};

#[derive(Debug)]
pub(crate) struct SpannedToken(Spanned<Token>);

impl SpannedToken {
    pub(crate) fn new(token: Token, span: Span) -> SpannedToken {
        SpannedToken(Spanned::from(span, token))
    }

    pub(crate) fn span(&self) -> Span {
        self.0.span()
    }

    pub(crate) fn token(&self) -> &Token {
        &self.0.contents
    }

    pub(crate) fn into_token(self) -> Token {
        self.0.contents
    }
}

/// Token types used in the ACIR text format.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    /// Identifier such as `RANGE`, `AND`, etc.
    Ident(String),
    /// Reserved identifiers such as `EXPR`.
    /// Most words in ACIR's human readable are expected to be keywords
    Keyword(Keyword),
    /// Witness index, like `_42`
    Witness(u32),
    /// Integer value represented using the underlying native field element
    Int(FieldElement),
    /// :
    Colon,
    /// ;
    Semicolon,
    /// ,
    Comma,
    /// [
    LeftBracket,
    /// ]
    RightBracket,
    /// (
    LeftParen,
    /// )
    RightParen,
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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(ident) => write!(f, "{ident}"),
            Token::Keyword(keyword) => write!(f, "{keyword}"),
            Token::Witness(index) => write!(f, "_{index}"),
            Token::Int(int) => write!(f, "{int}"),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Eof => write!(f, "(end of stream)"),
        }
    }
}

/// ACIR human readable text format keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Keyword {
    /// current
    Current,
    /// witness
    Witness,
    /// index
    Index,
    /// private
    Private,
    /// parameters
    Parameters,
    /// indices
    Indices,
    /// public
    Public,
    /// return
    Return,
    /// value
    Value,
    /// EXPR
    Expression,
    /// BLACKBOX
    BlackBoxFuncCall,
}

impl Keyword {
    pub(super) fn lookup_keyword(word: &str) -> Option<Token> {
        let keyword = match word {
            "current" => Keyword::Current,
            "witness" => Keyword::Witness,
            "index" => Keyword::Index,
            "private" => Keyword::Private,
            "parameters" => Keyword::Parameters,
            "indices" => Keyword::Indices,
            "public" => Keyword::Public,
            "return" => Keyword::Return,
            "value" => Keyword::Value,
            "EXPR" => Keyword::Expression,
            "BLACKBOX" => Keyword::BlackBoxFuncCall,
            _ => return None,
        };
        Some(Token::Keyword(keyword))
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Current => write!(f, "current"),
            Keyword::Witness => write!(f, "witness"),
            Keyword::Index => write!(f, "index"),
            Keyword::Private => write!(f, "private"),
            Keyword::Parameters => write!(f, "parameters"),
            Keyword::Indices => write!(f, "indices"),
            Keyword::Public => write!(f, "public"),
            Keyword::Return => write!(f, "return"),
            Keyword::Value => write!(f, "value"),
            Keyword::Expression => write!(f, "EXPR"),
            Keyword::BlackBoxFuncCall => write!(f, "BLACKBOX"),
        }
    }
}
