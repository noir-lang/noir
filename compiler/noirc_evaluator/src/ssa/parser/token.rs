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
    /// ==
    Equal,
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
    Add,
    Allocate,
    And,
    ArrayGet,
    ArraySet,
    As,
    Bits,
    Bool,
    Brillig,
    Call,
    Cast,
    Constrain,
    Div,
    Inline,
    InlineAlways,
    Else,
    EnableSideEffects,
    Eq,
    Field,
    Fold,
    Fn,
    Index,
    Jmp,
    Jmpif,
    Load,
    Lt,
    MaxBitSize,
    Mod,
    Mul,
    NoPredicates,
    Not,
    Of,
    Or,
    RangeCheck,
    Return,
    Shl,
    Shr,
    Sub,
    Then,
    To,
    Truncate,
    Value,
    Xor,
}

impl Keyword {
    pub(crate) fn lookup_keyword(word: &str) -> Option<Token> {
        let keyword = match word {
            "acir" => Keyword::Acir,
            "add" => Keyword::Add,
            "allocate" => Keyword::Allocate,
            "and" => Keyword::And,
            "array_get" => Keyword::ArrayGet,
            "array_set" => Keyword::ArraySet,
            "as" => Keyword::As,
            "bits" => Keyword::Bits,
            "bool" => Keyword::Bool,
            "brillig" => Keyword::Brillig,
            "call" => Keyword::Call,
            "cast" => Keyword::Cast,
            "constrain" => Keyword::Constrain,
            "div" => Keyword::Div,
            "else" => Keyword::Else,
            "enable_side_effects" => Keyword::EnableSideEffects,
            "eq" => Keyword::Eq,
            "inline" => Keyword::Inline,
            "inline_always" => Keyword::InlineAlways,
            "Field" => Keyword::Field,
            "fold" => Keyword::Fold,
            "fn" => Keyword::Fn,
            "index" => Keyword::Index,
            "jmp" => Keyword::Jmp,
            "jmpif" => Keyword::Jmpif,
            "load" => Keyword::Load,
            "lt" => Keyword::Lt,
            "max_bit_size" => Keyword::MaxBitSize,
            "mod" => Keyword::Mod,
            "mul" => Keyword::Mul,
            "no_predicates" => Keyword::NoPredicates,
            "not" => Keyword::Not,
            "of" => Keyword::Of,
            "or" => Keyword::Or,
            "range_check" => Keyword::RangeCheck,
            "return" => Keyword::Return,
            "shl" => Keyword::Shl,
            "shr" => Keyword::Shr,
            "sub" => Keyword::Sub,
            "then" => Keyword::Then,
            "to" => Keyword::To,
            "truncate" => Keyword::Truncate,
            "value" => Keyword::Value,
            "xor" => Keyword::Xor,
            _ => return None,
        };
        Some(Token::Keyword(keyword))
    }
}
