use acvm::FieldElement;
use noirc_errors::{Located, Location, Position, Span, Spanned};
use std::fmt::{self, Display};

use crate::{
    ast::{Expression, Path},
    graph::CrateId,
    node_interner::{
        ExprId, InternedExpressionKind, InternedPattern, InternedStatementKind,
        InternedUnresolvedTypeData, QuotedTypeId,
    },
};

/// Represents a token in noir's grammar - a word, number,
/// or symbol that can be used in noir's syntax. This is the
/// smallest unit of grammar. A parser may (will) decide to parse
/// items differently depending on the Tokens present but will
/// never parse the same ordering of identical tokens differently.
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum BorrowedToken<'input> {
    Ident(&'input str),
    Int(FieldElement, Option<IntegerTypeSuffix>),
    Bool(bool),
    Str(&'input str),
    /// the u8 is the number of hashes, i.e. r###..
    RawStr(&'input str, u8),
    FmtStr(&'input [FmtStrFragment], u32 /* length */),
    Keyword(Keyword),
    AttributeStart {
        is_inner: bool,
        is_tag: bool,
    },
    LineComment(&'input str, Option<DocStyle>),
    BlockComment(&'input str, Option<DocStyle>),
    Quote(&'input Tokens),
    QuotedType(QuotedTypeId),
    InternedExpression(InternedExpressionKind),
    InternedStatement(InternedStatementKind),
    InternedLValue(InternedExpressionKind),
    InternedUnresolvedTypeData(InternedUnresolvedTypeData),
    InternedPattern(InternedPattern),
    InternedCrate(CrateId),
    /// <
    Less,
    /// <=
    LessEqual,
    /// >
    Greater,
    /// >=
    GreaterEqual,
    /// ==
    Equal,
    /// !=
    NotEqual,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// \
    Backslash,
    /// %
    Percent,
    /// &
    Ampersand,
    /// &
    DeprecatedVectorStart,
    /// @
    At,
    /// ^
    Caret,
    /// <<
    ShiftLeft,
    /// >>
    ShiftRight,
    /// .
    Dot,
    /// ..
    DoubleDot,
    /// ..=
    DoubleDotEqual,
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
    /// ->
    Arrow,
    /// =>
    FatArrow,
    /// |
    Pipe,
    /// #
    Pound,
    /// ,
    Comma,
    /// :
    Colon,
    /// ::
    DoubleColon,
    /// ;
    Semicolon,
    /// !
    Bang,
    /// $
    DollarSign,
    /// =
    Assign,
    /// &&
    LogicalAnd,
    #[allow(clippy::upper_case_acronyms)]
    EOF,

    Whitespace(&'input str),

    /// This is an implementation detail on how macros are implemented by quoting token streams.
    /// This token marks where an unquote operation is performed. The ExprId argument is the
    /// resolved variable which is being unquoted at this position in the token stream.
    UnquoteMarker(ExprId),

    /// An invalid character is one that is not in noir's language or grammar.
    ///
    /// We don't report invalid tokens in the source as errors until parsing to
    /// avoid reporting the error twice (once while lexing, again when it is encountered
    /// during parsing). Reporting during lexing then removing these from the token stream
    /// would not be equivalent as it would change the resulting parse.
    Invalid(char),
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, PartialOrd, Ord)]
pub enum IntegerTypeSuffix {
    I8,
    I16,
    I32,
    I64,
    U1,
    U8,
    U16,
    U32,
    U64,
    U128,
    Field,
}

impl IntegerTypeSuffix {
    /// Returns the type of this integer suffix when used in a value position.
    /// Note that this is _not_ the type of the integer when the integer is in a type position!
    ///
    /// An integer value like `3u32` has type `u32` but when used in a type `[Field; 3u32]`,
    /// `3u32` will have the type `Type::Constant(3, Kind::Numeric(u32))`. As a result, using
    /// this method for any kind checks on integer types will result in a kind error! For those
    /// cases, use [IntegerTypeSuffix::as_kind] instead.
    pub(crate) fn as_type(self) -> crate::Type {
        use crate::{Type::Integer, ast::IntegerBitSize::*, shared::Signedness::*};
        match self {
            IntegerTypeSuffix::I8 => Integer(Signed, Eight),
            IntegerTypeSuffix::I16 => Integer(Signed, Sixteen),
            IntegerTypeSuffix::I32 => Integer(Signed, ThirtyTwo),
            IntegerTypeSuffix::I64 => Integer(Signed, SixtyFour),
            IntegerTypeSuffix::U1 => Integer(Unsigned, One),
            IntegerTypeSuffix::U8 => Integer(Unsigned, Eight),
            IntegerTypeSuffix::U16 => Integer(Unsigned, Sixteen),
            IntegerTypeSuffix::U32 => Integer(Unsigned, ThirtyTwo),
            IntegerTypeSuffix::U64 => Integer(Unsigned, SixtyFour),
            IntegerTypeSuffix::U128 => Integer(Unsigned, HundredTwentyEight),
            IntegerTypeSuffix::Field => crate::Type::FieldElement,
        }
    }

    /// Returns the kind of this integer constant when used in a type position.
    /// For example, when used as `[Field; 3u32]`, this [IntegerTypeSuffix::U32]
    /// will return `Kind::Numeric(Type::U32)`.
    ///
    /// This method should generally be used whenever an integer is used in a type position.
    /// [IntegerTypeSuffix::as_type] would return a raw `u32` type which is not the actual
    /// type of an integer in a type position - that'd be `Type::Constant(3, Kind::Numeric(u32))`
    /// for `3u32`.
    pub(crate) fn as_kind(self) -> crate::Kind {
        crate::Kind::Numeric(Box::new(self.as_type()))
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum Token {
    Ident(String),
    Int(FieldElement, Option<IntegerTypeSuffix>),
    Bool(bool),
    Str(String),
    /// the u8 is the number of hashes, i.e. r###..
    RawStr(String, u8),
    FmtStr(Vec<FmtStrFragment>, u32 /* length */),
    Keyword(Keyword),
    AttributeStart {
        is_inner: bool,
        is_tag: bool,
    },
    LineComment(String, Option<DocStyle>),
    BlockComment(String, Option<DocStyle>),
    // A `quote { ... }` along with the tokens in its token stream.
    Quote(Tokens),
    /// A quoted type resulting from a `Type` object in noir code being
    /// spliced into a macro's token stream. We preserve the original type
    /// to avoid having to tokenize it, re-parse it, and re-resolve it which
    /// may change the underlying type.
    QuotedType(QuotedTypeId),
    /// A reference to an interned `ExpressionKind`.
    InternedExpr(InternedExpressionKind),
    /// A reference to an interned `StatementKind`.
    InternedStatement(InternedStatementKind),
    /// A reference to an interned `LValue`.
    InternedLValue(InternedExpressionKind),
    /// A reference to an interned `UnresolvedTypeData`.
    InternedUnresolvedTypeData(InternedUnresolvedTypeData),
    /// A reference to an interned `Pattern`.
    InternedPattern(InternedPattern),
    /// A reference to an existing crate. This is a result of using `$crate` in a macro
    InternedCrate(CrateId),
    /// <
    Less,
    /// <=
    LessEqual,
    /// >
    Greater,
    /// >=
    GreaterEqual,
    /// ==
    Equal,
    /// !=
    NotEqual,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// \
    Backslash,
    /// %
    Percent,
    /// &
    Ampersand,
    /// &
    DeprecatedVectorStart,
    /// @
    At,
    /// ^
    Caret,
    /// <<
    ShiftLeft,
    /// >>
    ShiftRight,
    /// .
    Dot,
    /// ..
    DoubleDot,
    /// ..=
    DoubleDotEqual,
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
    /// ->
    Arrow,
    /// =>
    FatArrow,
    /// |
    Pipe,
    /// #
    Pound,
    /// ,
    Comma,
    /// :
    Colon,
    /// ::
    DoubleColon,
    /// ;
    Semicolon,
    /// !
    Bang,
    /// =
    Assign,
    /// $
    DollarSign,
    /// &&
    LogicalAnd,
    #[allow(clippy::upper_case_acronyms)]
    EOF,

    Whitespace(String),

    /// This is an implementation detail on how macros are implemented by quoting token streams.
    /// This token marks where an unquote operation is performed. The ExprId argument is the
    /// resolved variable which is being unquoted at this position in the token stream.
    UnquoteMarker(ExprId),

    /// An invalid character is one that is not in noir's language or grammar.
    ///
    /// We don't report invalid tokens in the source as errors until parsing to
    /// avoid reporting the error twice (once while lexing, again when it is encountered
    /// during parsing). Reporting during lexing then removing these from the token stream
    /// would not be equivalent as it would change the resulting parse.
    Invalid(char),
}

pub fn token_to_borrowed_token(token: &Token) -> BorrowedToken<'_> {
    match token {
        Token::Ident(s) => BorrowedToken::Ident(s),
        Token::Int(n, suffix) => BorrowedToken::Int(*n, *suffix),
        Token::Bool(b) => BorrowedToken::Bool(*b),
        Token::Str(b) => BorrowedToken::Str(b),
        Token::FmtStr(b, length) => BorrowedToken::FmtStr(b, *length),
        Token::RawStr(b, hashes) => BorrowedToken::RawStr(b, *hashes),
        Token::Keyword(k) => BorrowedToken::Keyword(*k),
        Token::AttributeStart { is_inner, is_tag } => {
            BorrowedToken::AttributeStart { is_inner: *is_inner, is_tag: *is_tag }
        }
        Token::LineComment(s, _style) => BorrowedToken::LineComment(s, *_style),
        Token::BlockComment(s, _style) => BorrowedToken::BlockComment(s, *_style),
        Token::Quote(stream) => BorrowedToken::Quote(stream),
        Token::QuotedType(id) => BorrowedToken::QuotedType(*id),
        Token::InternedExpr(id) => BorrowedToken::InternedExpression(*id),
        Token::InternedStatement(id) => BorrowedToken::InternedStatement(*id),
        Token::InternedLValue(id) => BorrowedToken::InternedLValue(*id),
        Token::InternedUnresolvedTypeData(id) => BorrowedToken::InternedUnresolvedTypeData(*id),
        Token::InternedPattern(id) => BorrowedToken::InternedPattern(*id),
        Token::InternedCrate(id) => BorrowedToken::InternedCrate(*id),
        Token::Less => BorrowedToken::Less,
        Token::LessEqual => BorrowedToken::LessEqual,
        Token::Greater => BorrowedToken::Greater,
        Token::GreaterEqual => BorrowedToken::GreaterEqual,
        Token::Equal => BorrowedToken::Equal,
        Token::NotEqual => BorrowedToken::NotEqual,
        Token::Plus => BorrowedToken::Plus,
        Token::Minus => BorrowedToken::Minus,
        Token::Star => BorrowedToken::Star,
        Token::Slash => BorrowedToken::Slash,
        Token::Backslash => BorrowedToken::Backslash,
        Token::Percent => BorrowedToken::Percent,
        Token::Ampersand => BorrowedToken::Ampersand,
        Token::DeprecatedVectorStart => BorrowedToken::DeprecatedVectorStart,
        Token::At => BorrowedToken::At,
        Token::Caret => BorrowedToken::Caret,
        Token::ShiftLeft => BorrowedToken::ShiftLeft,
        Token::ShiftRight => BorrowedToken::ShiftRight,
        Token::Dot => BorrowedToken::Dot,
        Token::DoubleDot => BorrowedToken::DoubleDot,
        Token::DoubleDotEqual => BorrowedToken::DoubleDotEqual,
        Token::LeftParen => BorrowedToken::LeftParen,
        Token::RightParen => BorrowedToken::RightParen,
        Token::LeftBrace => BorrowedToken::LeftBrace,
        Token::RightBrace => BorrowedToken::RightBrace,
        Token::LeftBracket => BorrowedToken::LeftBracket,
        Token::RightBracket => BorrowedToken::RightBracket,
        Token::Arrow => BorrowedToken::Arrow,
        Token::FatArrow => BorrowedToken::FatArrow,
        Token::Pipe => BorrowedToken::Pipe,
        Token::Pound => BorrowedToken::Pound,
        Token::Comma => BorrowedToken::Comma,
        Token::Colon => BorrowedToken::Colon,
        Token::DoubleColon => BorrowedToken::DoubleColon,
        Token::Semicolon => BorrowedToken::Semicolon,
        Token::Assign => BorrowedToken::Assign,
        Token::Bang => BorrowedToken::Bang,
        Token::DollarSign => BorrowedToken::DollarSign,
        Token::LogicalAnd => BorrowedToken::LogicalAnd,
        Token::EOF => BorrowedToken::EOF,
        Token::Invalid(c) => BorrowedToken::Invalid(*c),
        Token::Whitespace(s) => BorrowedToken::Whitespace(s),
        Token::UnquoteMarker(id) => BorrowedToken::UnquoteMarker(*id),
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum FmtStrFragment {
    String(String),
    Interpolation(String, Location),
}

impl Display for FmtStrFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FmtStrFragment::String(string) => {
                // Undo the escapes when displaying the fmt string
                let string = string
                    .replace('{', "{{")
                    .replace('}', "}}")
                    .replace('\r', "\\r")
                    .replace('\n', "\\n")
                    .replace('\t', "\\t")
                    .replace('\0', "\\0")
                    .replace('\'', "\\'")
                    .replace('\"', "\\\"");
                write!(f, "{string}")
            }
            FmtStrFragment::Interpolation(string, _) => {
                write!(f, "{{{string}}}")
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum DocStyle {
    Outer,
    Inner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocatedToken(Located<Token>);

impl PartialEq<LocatedToken> for Token {
    fn eq(&self, other: &LocatedToken) -> bool {
        self == other.token()
    }
}
impl PartialEq<Token> for LocatedToken {
    fn eq(&self, other: &Token) -> bool {
        self.token() == other
    }
}

impl From<LocatedToken> for Token {
    fn from(spt: LocatedToken) -> Self {
        spt.into_token()
    }
}

impl<'a> From<&'a LocatedToken> for &'a Token {
    fn from(spt: &'a LocatedToken) -> Self {
        spt.token()
    }
}

impl LocatedToken {
    pub fn new(token: Token, location: Location) -> LocatedToken {
        LocatedToken(Located::from(location, token))
    }
    pub fn location(&self) -> Location {
        self.0.location()
    }
    pub fn span(&self) -> Span {
        self.0.span()
    }
    pub fn token(&self) -> &Token {
        &self.0.contents
    }
    pub fn into_token(self) -> Token {
        self.0.contents
    }
    pub fn kind(&self) -> TokenKind {
        self.token().kind()
    }
    pub fn into_spanned_token(self) -> SpannedToken {
        let span = self.span();
        SpannedToken::new(self.into_token(), span)
    }
}

impl Display for LocatedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.token().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpannedToken(Spanned<Token>);

impl PartialEq<SpannedToken> for Token {
    fn eq(&self, other: &SpannedToken) -> bool {
        self == other.token()
    }
}
impl PartialEq<Token> for SpannedToken {
    fn eq(&self, other: &Token) -> bool {
        self.token() == other
    }
}

impl From<SpannedToken> for Token {
    fn from(spt: SpannedToken) -> Self {
        spt.into_token()
    }
}

impl<'a> From<&'a SpannedToken> for &'a Token {
    fn from(spt: &'a SpannedToken) -> Self {
        spt.token()
    }
}

impl SpannedToken {
    pub fn new(token: Token, span: Span) -> SpannedToken {
        SpannedToken(Spanned::from(span, token))
    }
    pub fn span(&self) -> Span {
        self.0.span()
    }
    pub fn token(&self) -> &Token {
        &self.0.contents
    }
    pub fn into_token(self) -> Token {
        self.0.contents
    }
    pub fn kind(&self) -> TokenKind {
        self.token().kind()
    }
}

impl Display for SpannedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.token().fmt(f)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Ident(ref s) => write!(f, "{s}"),
            Token::Int(n, Some(suffix)) => write!(f, "{n}_{suffix}"),
            Token::Int(n, None) => write!(f, "{n}"),
            Token::Bool(b) => write!(f, "{b}"),
            Token::Str(ref b) => write!(f, "{b:?}"),
            Token::FmtStr(ref b, _length) => write!(f, "f{b:?}"),
            Token::RawStr(ref b, hashes) => {
                let h: String = std::iter::once('#').cycle().take(hashes as usize).collect();
                write!(f, "r{h}{b:?}{h}")
            }
            Token::Keyword(k) => write!(f, "{k}"),
            Token::AttributeStart { is_inner, is_tag } => {
                write!(f, "#")?;
                if is_inner {
                    write!(f, "!")?;
                }
                write!(f, "[")?;
                if is_tag {
                    write!(f, "'")?;
                }
                Ok(())
            }
            Token::LineComment(ref s, style) => match style {
                Some(DocStyle::Inner) => write!(f, "//!{s}"),
                Some(DocStyle::Outer) => write!(f, "///{s}"),
                None => write!(f, "//{s}"),
            },
            Token::BlockComment(ref s, style) => match style {
                Some(DocStyle::Inner) => write!(f, "/*!{s}*/"),
                Some(DocStyle::Outer) => write!(f, "/**{s}*/"),
                None => write!(f, "/*{s}*/"),
            },
            Token::Quote(ref stream) => {
                write!(f, "quote {{")?;
                for token in stream.0.iter() {
                    write!(f, " {token}")?;
                }
                write!(f, "}}")
            }
            // Quoted types and exprs only have an ID so there is nothing to display
            Token::QuotedType(_) => write!(f, "(type)"),
            Token::InternedExpr(_)
            | Token::InternedStatement(_)
            | Token::InternedLValue(_)
            | Token::InternedPattern(_) => {
                write!(f, "(expr)")
            }
            Token::InternedUnresolvedTypeData(_) => write!(f, "(type)"),
            Token::InternedCrate(_) => write!(f, "$crate"),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Backslash => write!(f, "\\"),
            Token::Percent => write!(f, "%"),
            Token::Ampersand => write!(f, "&"),
            Token::DeprecatedVectorStart => write!(f, "&"),
            Token::At => write!(f, "@"),
            Token::Caret => write!(f, "^"),
            Token::ShiftLeft => write!(f, "<<"),
            Token::ShiftRight => write!(f, ">>"),
            Token::Dot => write!(f, "."),
            Token::DoubleDot => write!(f, ".."),
            Token::DoubleDotEqual => write!(f, "..="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::Pipe => write!(f, "|"),
            Token::Pound => write!(f, "#"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::DoubleColon => write!(f, "::"),
            Token::Semicolon => write!(f, ";"),
            Token::Assign => write!(f, "="),
            Token::Bang => write!(f, "!"),
            Token::DollarSign => write!(f, "$"),
            Token::LogicalAnd => write!(f, "&&"),
            Token::EOF => write!(f, "end of input"),
            Token::Invalid(c) => write!(f, "{c}"),
            Token::Whitespace(ref s) => write!(f, "{s}"),
            Token::UnquoteMarker(_) => write!(f, "(UnquoteMarker)"),
        }
    }
}

impl Display for IntegerTypeSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegerTypeSuffix::I8 => write!(f, "i8"),
            IntegerTypeSuffix::I16 => write!(f, "i16"),
            IntegerTypeSuffix::I32 => write!(f, "i32"),
            IntegerTypeSuffix::I64 => write!(f, "i64"),
            IntegerTypeSuffix::U1 => write!(f, "u1"),
            IntegerTypeSuffix::U8 => write!(f, "u8"),
            IntegerTypeSuffix::U16 => write!(f, "u16"),
            IntegerTypeSuffix::U32 => write!(f, "u32"),
            IntegerTypeSuffix::U64 => write!(f, "u64"),
            IntegerTypeSuffix::U128 => write!(f, "u128"),
            IntegerTypeSuffix::Field => write!(f, "Field"),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Ord, PartialOrd)]
/// The different kinds of tokens that are possible in the target language
pub enum TokenKind {
    Token(Token),
    Ident,
    Literal,
    Keyword,
    Attribute,
    InnerAttribute,
    Quote,
    QuotedType,
    InternedExpr,
    InternedStatement,
    InternedLValue,
    InternedUnresolvedTypeData,
    InternedPattern,
    UnquoteMarker,
    Comment,
    OuterDocComment,
    InnerDocComment,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Token(tok) => write!(f, "{tok}"),
            TokenKind::Ident => write!(f, "identifier"),
            TokenKind::Literal => write!(f, "literal"),
            TokenKind::Keyword => write!(f, "keyword"),
            TokenKind::Attribute => write!(f, "attribute"),
            TokenKind::InnerAttribute => write!(f, "inner attribute"),
            TokenKind::Quote => write!(f, "quote"),
            TokenKind::QuotedType => write!(f, "quoted type"),
            TokenKind::InternedExpr => write!(f, "interned expr"),
            TokenKind::InternedStatement => write!(f, "interned statement"),
            TokenKind::InternedLValue => write!(f, "interned lvalue"),
            TokenKind::InternedUnresolvedTypeData => write!(f, "interned unresolved type"),
            TokenKind::InternedPattern => write!(f, "interned pattern"),
            TokenKind::UnquoteMarker => write!(f, "macro result"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::OuterDocComment => write!(f, "outer doc comment"),
            TokenKind::InnerDocComment => write!(f, "inner doc comment"),
        }
    }
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Ident(_) => TokenKind::Ident,
            Token::Int(..)
            | Token::Bool(_)
            | Token::Str(_)
            | Token::RawStr(..)
            | Token::FmtStr(_, _) => TokenKind::Literal,
            Token::Keyword(_) => TokenKind::Keyword,
            Token::UnquoteMarker(_) => TokenKind::UnquoteMarker,
            Token::Quote(_) => TokenKind::Quote,
            Token::QuotedType(_) => TokenKind::QuotedType,
            Token::InternedExpr(_) => TokenKind::InternedExpr,
            Token::InternedStatement(_) => TokenKind::InternedStatement,
            Token::InternedLValue(_) => TokenKind::InternedLValue,
            Token::InternedUnresolvedTypeData(_) => TokenKind::InternedUnresolvedTypeData,
            Token::InternedPattern(_) => TokenKind::InternedPattern,
            Token::LineComment(_, None) | Token::BlockComment(_, None) => TokenKind::Comment,
            Token::LineComment(_, Some(DocStyle::Outer))
            | Token::BlockComment(_, Some(DocStyle::Outer)) => TokenKind::OuterDocComment,
            Token::LineComment(_, Some(DocStyle::Inner))
            | Token::BlockComment(_, Some(DocStyle::Inner)) => TokenKind::InnerDocComment,
            tok => TokenKind::Token(tok.clone()),
        }
    }

    pub fn is_ident(&self) -> bool {
        matches!(self, Token::Ident(_))
    }

    pub(super) fn into_single_span(self, position: Position) -> SpannedToken {
        self.into_span(position, position)
    }

    pub(super) fn into_span(self, start: Position, end: Position) -> SpannedToken {
        SpannedToken(Spanned::from_position(start, end, self))
    }

    /// These are all the operators allowed as part of
    /// a short-hand assignment: `a <op>= b`
    pub fn assign_shorthand_operators() -> [Token; 10] {
        use Token::*;
        [Plus, Minus, Star, Slash, Percent, Ampersand, Caret, ShiftLeft, ShiftRight, Pipe]
    }

    pub fn try_into_binary_op(self, span: Span) -> Option<Spanned<crate::ast::BinaryOpKind>> {
        use crate::ast::BinaryOpKind::*;
        let binary_op = match self {
            Token::Plus => Add,
            Token::Ampersand => And,
            Token::Caret => Xor,
            Token::ShiftLeft => ShiftLeft,
            Token::ShiftRight => ShiftRight,
            Token::Pipe => Or,
            Token::Minus => Subtract,
            Token::Star => Multiply,
            Token::Slash => Divide,
            Token::Equal => Equal,
            Token::NotEqual => NotEqual,
            Token::Less => Less,
            Token::LessEqual => LessEqual,
            Token::Greater => Greater,
            Token::GreaterEqual => GreaterEqual,
            Token::Percent => Modulo,
            _ => return None,
        };
        Some(Spanned::from(span, binary_op))
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum IntType {
    Unsigned(u32), // u32 = Unsigned(32)
    Signed(u32),   // i64 = Signed(64)
}

impl Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IntType::Unsigned(num) => write!(f, "u{num}"),
            IntType::Signed(num) => write!(f, "i{num}"),
        }
    }
}

impl IntType {
    // XXX: Result<Option<Token, LexerErrorKind>
    // Is not the best API. We could split this into two functions. One that checks if the
    // word is a integer, which only returns an Option
    pub fn lookup_int_type(word: &str) -> Option<IntType> {
        // Check if the first string is a 'u' or 'i'

        let is_signed = if word.starts_with('i') {
            true
        } else if word.starts_with('u') {
            false
        } else {
            return None;
        };

        // Word start with 'u' or 'i'. Check if the latter is an integer

        let str_as_u32 = match word[1..].parse::<u32>() {
            Ok(str_as_u32) => str_as_u32,
            Err(_) => return None,
        };

        if is_signed {
            Some(IntType::Signed(str_as_u32))
        } else {
            Some(IntType::Unsigned(str_as_u32))
        }
    }
}

/// TestScope is used to specify additional annotations for test functions
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum TestScope {
    /// If a test has a scope of ShouldFailWith, then it can only pass
    /// if it fails with the specified reason. If the reason is None, then
    /// the test must unconditionally fail
    ShouldFailWith { reason: Option<String> },
    /// If a test has a scope of OnlyFailWith, then it can only fail
    /// if it fails with the specified reason.
    OnlyFailWith { reason: String },
    /// No scope is applied and so the test must pass
    None,
}

impl Display for TestScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TestScope::None => write!(f, ""),
            TestScope::ShouldFailWith { reason } => match reason {
                Some(failure_reason) => write!(f, "(should_fail_with = {failure_reason:?})"),
                None => write!(f, "(should_fail)"),
            },
            TestScope::OnlyFailWith { reason } => {
                write!(f, "(only_fail_with = {reason:?})")
            }
        }
    }
}

/// FuzzingScope is used to specify additional annotations for fuzzing harnesses
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum FuzzingScope {
    /// If the fuzzing harness has a scope of ShouldFailWith, then it should only pass
    /// if it fails with the specified reason. If the reason is None, then
    /// the harness must unconditionally fail
    ShouldFailWith {
        reason: Option<String>,
    },
    /// If a fuzzing harness has a scope of OnlyFailWith, then it will only detect an assert
    /// if it fails with the specified reason.
    OnlyFailWith {
        reason: String,
    },
    None,
}

impl Display for FuzzingScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FuzzingScope::None => write!(f, ""),
            FuzzingScope::OnlyFailWith { reason } => write!(f, "(only_fail_with = {reason:?})"),
            FuzzingScope::ShouldFailWith { reason } => match reason {
                Some(failure_reason) => write!(f, "(should_fail_with = {failure_reason:?})"),
                None => write!(f, "(should_fail)"),
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
// Attributes are special language markers in the target language
// An example of one is `#[SHA256]` . Currently only Foreign attributes are supported
// Calls to functions which have the foreign attribute are executed in the host language
pub struct Attributes {
    // Each function can have a single Primary Attribute
    pub function: Option<(FunctionAttribute, usize /* index in list */)>,
    // Each function can have many Secondary Attributes
    pub secondary: Vec<SecondaryAttribute>,
}

impl Attributes {
    pub fn empty() -> Self {
        Self { function: None, secondary: Vec::new() }
    }

    pub fn function(&self) -> Option<&FunctionAttribute> {
        self.function.as_ref().map(|(attr, _)| attr)
    }

    pub fn set_function(&mut self, function: FunctionAttribute) {
        // Assume the index in the list doesn't matter anymore at this point
        self.function = Some((function, 0));
    }

    /// Returns true if one of the secondary attributes is `contract_library_method`
    ///
    /// This is useful for finding out if we should compile a contract method
    /// as an entry point or not.
    pub fn has_contract_library_method(&self) -> bool {
        self.has_secondary_attr(&SecondaryAttributeKind::ContractLibraryMethod)
    }

    pub fn is_test_function(&self) -> bool {
        self.as_test_function().is_some()
    }

    pub fn as_test_function(&self) -> Option<(&TestScope, Location)> {
        self.function().and_then(|attr| {
            if let FunctionAttributeKind::Test(scope) = &attr.kind {
                Some((scope, attr.location))
            } else {
                None
            }
        })
    }

    pub fn is_fuzzing_harness(&self) -> bool {
        self.as_fuzzing_harness().is_some()
    }

    pub fn as_fuzzing_harness(&self) -> Option<(&FuzzingScope, Location)> {
        self.function().and_then(|attr| {
            if let FunctionAttributeKind::FuzzingHarness(scope) = &attr.kind {
                Some((scope, attr.location))
            } else {
                None
            }
        })
    }

    /// True if these attributes mean the given function is an entry point function if it was
    /// defined within a contract. Note that this does not check if the function is actually part
    /// of a contract.
    pub fn is_contract_entry_point(&self) -> bool {
        !self.has_contract_library_method()
            && !self.is_test_function()
            && !self.is_fuzzing_harness()
    }

    /// Returns note if a deprecated secondary attribute is found
    pub fn get_deprecated_note(&self) -> Option<Option<String>> {
        self.secondary.iter().find_map(|attr| match &attr.kind {
            SecondaryAttributeKind::Deprecated(note) => Some(note.clone()),
            _ => None,
        })
    }

    pub fn get_field_attribute(&self) -> Option<String> {
        for secondary in &self.secondary {
            if let SecondaryAttributeKind::Field(field) = &secondary.kind {
                return Some(field.to_lowercase());
            }
        }
        None
    }

    pub fn is_foldable(&self) -> bool {
        self.function().is_some_and(|func_attribute| func_attribute.kind.is_foldable())
    }

    pub fn is_no_predicates(&self) -> bool {
        self.function().is_some_and(|func_attribute| func_attribute.kind.is_no_predicates())
    }

    pub fn has_varargs(&self) -> bool {
        self.has_secondary_attr(&SecondaryAttributeKind::Varargs)
    }

    pub fn has_use_callers_scope(&self) -> bool {
        self.has_secondary_attr(&SecondaryAttributeKind::UseCallersScope)
    }

    /// True if the function is marked with an `#[export]` attribute.
    pub fn has_export(&self) -> bool {
        self.has_secondary_attr(&SecondaryAttributeKind::Export)
    }

    pub fn has_allow(&self, name: &'static str) -> bool {
        self.secondary.iter().any(|attr| attr.kind.is_allow(name))
    }

    /// Check if secondary attributes contain a specific instance.
    pub fn has_secondary_attr(&self, kind: &SecondaryAttributeKind) -> bool {
        self.secondary.iter().any(|attr| &attr.kind == kind)
    }
}

/// An Attribute can be either a Primary Attribute or a Secondary Attribute
/// A Primary Attribute can alter the function type, thus there can only be one
/// A secondary attribute has no effect and is either consumed by a library or used as a notice for the developer
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Attribute {
    Function(FunctionAttribute),
    Secondary(SecondaryAttribute),
}

impl Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Attribute::Function(attribute) => write!(f, "{attribute}"),
            Attribute::Secondary(attribute) => write!(f, "{attribute}"),
        }
    }
}

/// Primary Attributes are those which a function can only have one of.
/// They change the FunctionKind and thus have direct impact on the IR output
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub struct FunctionAttribute {
    pub kind: FunctionAttributeKind,
    pub location: Location,
}

/// Primary Attributes are those which a function can only have one of.
/// They change the FunctionKind and thus have direct impact on the IR output
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum FunctionAttributeKind {
    Foreign(String),
    Builtin(String),
    Oracle(String),
    Test(TestScope),
    Fold,
    NoPredicates,
    InlineAlways,
    InlineNever,
    FuzzingHarness(FuzzingScope),
}

impl FunctionAttributeKind {
    pub fn builtin(&self) -> Option<&String> {
        match self {
            FunctionAttributeKind::Builtin(name) => Some(name),
            _ => None,
        }
    }

    pub fn foreign(&self) -> Option<&String> {
        match self {
            FunctionAttributeKind::Foreign(name) => Some(name),
            _ => None,
        }
    }

    pub fn oracle(&self) -> Option<&String> {
        match self {
            FunctionAttributeKind::Oracle(name) => Some(name),
            _ => None,
        }
    }

    pub fn is_oracle(&self) -> bool {
        matches!(self, FunctionAttributeKind::Oracle(_))
    }

    pub fn is_low_level(&self) -> bool {
        matches!(self, FunctionAttributeKind::Foreign(_) | FunctionAttributeKind::Builtin(_))
    }

    pub fn is_foldable(&self) -> bool {
        matches!(self, FunctionAttributeKind::Fold)
    }

    /// Check whether we have an `inline` attribute
    /// Although we also do not want to inline foldable functions,
    /// we keep the two attributes distinct for clarity.
    pub fn is_no_predicates(&self) -> bool {
        matches!(self, FunctionAttributeKind::NoPredicates)
    }

    pub fn name(&self) -> &'static str {
        match self {
            FunctionAttributeKind::Foreign(_) => "foreign",
            FunctionAttributeKind::Builtin(_) => "builtin",
            FunctionAttributeKind::Oracle(_) => "oracle",
            FunctionAttributeKind::Test(_) => "test",
            FunctionAttributeKind::Fold => "fold",
            FunctionAttributeKind::NoPredicates => "no_predicates",
            FunctionAttributeKind::InlineAlways => "inline_always",
            FunctionAttributeKind::InlineNever => "inline_never",
            FunctionAttributeKind::FuzzingHarness(_) => "fuzz",
        }
    }
}

impl Display for FunctionAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl Display for FunctionAttributeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionAttributeKind::Test(scope) => write!(f, "#[test{scope}]"),
            FunctionAttributeKind::Foreign(k) => write!(f, "#[foreign({k})]"),
            FunctionAttributeKind::Builtin(k) => write!(f, "#[builtin({k})]"),
            FunctionAttributeKind::Oracle(k) => write!(f, "#[oracle({k})]"),
            FunctionAttributeKind::Fold => write!(f, "#[fold]"),
            FunctionAttributeKind::NoPredicates => write!(f, "#[no_predicates]"),
            FunctionAttributeKind::InlineAlways => write!(f, "#[inline_always]"),
            FunctionAttributeKind::InlineNever => write!(f, "#[inline_never]"),
            FunctionAttributeKind::FuzzingHarness(scope) => write!(f, "#[fuzz{scope}]"),
        }
    }
}

/// Secondary attributes are those which a function can have many of.
/// They are not able to change the `FunctionKind` and thus do not have direct impact on the IR output
/// They are often consumed by libraries or used as notices for the developer
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SecondaryAttribute {
    pub kind: SecondaryAttributeKind,
    pub location: Location,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SecondaryAttributeKind {
    Deprecated(Option<String>),
    // This is an attribute to specify that a function
    // is a helper method for a contract and should not be seen as
    // the entry point.
    ContractLibraryMethod,
    Export,
    Field(String),

    /// A custom tag attribute: `#['foo]`
    Tag(String),

    /// An attribute expected to run a comptime function of the same name: `#[foo]`
    Meta(MetaAttribute),

    Abi(String),

    /// A variable-argument comptime function.
    Varargs,

    /// Treat any metaprogramming functions within this one as resolving
    /// within the scope of the calling function/module rather than this one.
    /// This affects functions such as `Expression::resolve` or `Quoted::as_type`.
    UseCallersScope,

    /// Allow chosen warnings to happen so they are silenced.
    Allow(String),

    /// Unlike Rust, all values in Noir already warn if they are not used.
    ///
    /// Instead, `#[must_use]` in Noir promotes this warning to a hard error, with
    /// an optional message for the error.
    MustUse(Option<String>),
}

impl SecondaryAttributeKind {
    pub(crate) fn is_allow(&self, name: &'static str) -> bool {
        match self {
            SecondaryAttributeKind::Allow(string) => string == name,
            _ => false,
        }
    }

    pub(crate) fn is_abi(&self) -> bool {
        matches!(self, SecondaryAttributeKind::Abi(_))
    }

    pub(crate) fn contents(&self) -> String {
        match self {
            SecondaryAttributeKind::Deprecated(None) => "deprecated".to_string(),
            SecondaryAttributeKind::Deprecated(Some(note)) => {
                format!("deprecated({note:?})")
            }
            SecondaryAttributeKind::Tag(contents) => format!("'{contents}"),
            SecondaryAttributeKind::Meta(meta) => meta.to_string(),
            SecondaryAttributeKind::ContractLibraryMethod => "contract_library_method".to_string(),
            SecondaryAttributeKind::Export => "export".to_string(),
            SecondaryAttributeKind::Field(k) => format!("field({k})"),
            SecondaryAttributeKind::Abi(k) => format!("abi({k})"),
            SecondaryAttributeKind::Varargs => "varargs".to_string(),
            SecondaryAttributeKind::UseCallersScope => "use_callers_scope".to_string(),
            SecondaryAttributeKind::Allow(k) => format!("allow({k})"),
            SecondaryAttributeKind::MustUse(None) => "must_use".to_string(),
            SecondaryAttributeKind::MustUse(Some(msg)) => format!("must_use = \"{msg}\""),
        }
    }

    /// If this is a `#[must_use]` attribute, return `Some(message)` where message is the
    /// optional message. Otherwise, return `None`. Since `message` itself is optional,
    /// `Some(None)` indicates there is a `must_use` but no message was provided.
    pub(crate) fn must_use_message(&self) -> Option<Option<String>> {
        match self {
            SecondaryAttributeKind::MustUse(message) => Some(message.clone()),
            _ => None,
        }
    }
}

impl Display for SecondaryAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl Display for SecondaryAttributeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#[{}]", self.contents())
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MetaAttribute {
    pub name: MetaAttributeName,
    pub arguments: Vec<Expression>,
}

impl Display for MetaAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.arguments.is_empty() {
            write!(f, "{}", self.name)
        } else {
            let args =
                self.arguments.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
            write!(f, "{}({})", self.name, args)
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MetaAttributeName {
    /// For example `foo::bar` in `#[foo::bar(...)]`
    Path(Path),
    /// For example `$expr` in `#[$expr(...)]` inside a `quote { ... }` expression.
    Resolved(ExprId),
}

impl Display for MetaAttributeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaAttributeName::Path(path) => path.fmt(f),
            MetaAttributeName::Resolved(_) => write!(f, "(quoted)"),
        }
    }
}

/// Note that `self` is not present - it is a contextual keyword rather than a true one as it is
/// only special within `impl`s. Otherwise `self` functions as a normal identifier.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, PartialOrd, Ord, strum_macros::EnumIter)]
pub enum Keyword {
    As,
    Assert,
    AssertEq,
    Break,
    CallData,
    Comptime,
    Constrain,
    Constrained,
    Continue,
    Contract,
    Crate,
    Dep,
    Dual,
    Else,
    Enum,
    Fn,
    For,
    Global,
    If,
    Impl,
    In,
    Let,
    Loop,
    Match,
    Mod,
    Mut,
    Pub,
    Return,
    ReturnData,
    Struct,
    Super,
    Trait,
    Type,
    Unchecked,
    Unconstrained,
    Unsafe,
    Use,
    Where,
    While,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Keyword::As => write!(f, "as"),
            Keyword::Assert => write!(f, "assert"),
            Keyword::AssertEq => write!(f, "assert_eq"),
            Keyword::Break => write!(f, "break"),
            Keyword::CallData => write!(f, "call_data"),
            Keyword::Comptime => write!(f, "comptime"),
            Keyword::Constrain => write!(f, "constrain"),
            Keyword::Constrained => write!(f, "constrained"),
            Keyword::Continue => write!(f, "continue"),
            Keyword::Contract => write!(f, "contract"),
            Keyword::Crate => write!(f, "crate"),
            Keyword::Dep => write!(f, "dep"),
            Keyword::Dual => write!(f, "dual"),
            Keyword::Else => write!(f, "else"),
            Keyword::Enum => write!(f, "enum"),
            Keyword::Fn => write!(f, "fn"),
            Keyword::For => write!(f, "for"),
            Keyword::Global => write!(f, "global"),
            Keyword::If => write!(f, "if"),
            Keyword::Impl => write!(f, "impl"),
            Keyword::In => write!(f, "in"),
            Keyword::Let => write!(f, "let"),
            Keyword::Loop => write!(f, "loop"),
            Keyword::Match => write!(f, "match"),
            Keyword::Mod => write!(f, "mod"),
            Keyword::Mut => write!(f, "mut"),
            Keyword::Pub => write!(f, "pub"),
            Keyword::Return => write!(f, "return"),
            Keyword::ReturnData => write!(f, "return_data"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::Super => write!(f, "super"),
            Keyword::Trait => write!(f, "trait"),
            Keyword::Type => write!(f, "type"),
            Keyword::Unchecked => write!(f, "unchecked"),
            Keyword::Unconstrained => write!(f, "unconstrained"),
            Keyword::Unsafe => write!(f, "unsafe"),
            Keyword::Use => write!(f, "use"),
            Keyword::Where => write!(f, "where"),
            Keyword::While => write!(f, "while"),
        }
    }
}

impl Keyword {
    /// Looks up a word in the source program and returns the associated keyword, if found.
    pub(crate) fn lookup_keyword(word: &str) -> Option<Token> {
        let keyword = match word {
            "as" => Keyword::As,
            "assert" => Keyword::Assert,
            "assert_eq" => Keyword::AssertEq,
            "break" => Keyword::Break,
            "call_data" => Keyword::CallData,
            "comptime" => Keyword::Comptime,
            "constrain" => Keyword::Constrain,
            "constrained" => Keyword::Constrained,
            "continue" => Keyword::Continue,
            "contract" => Keyword::Contract,
            "crate" => Keyword::Crate,
            "dep" => Keyword::Dep,
            "dual" => Keyword::Dual,
            "else" => Keyword::Else,
            "enum" => Keyword::Enum,
            "fn" => Keyword::Fn,
            "for" => Keyword::For,
            "global" => Keyword::Global,
            "if" => Keyword::If,
            "impl" => Keyword::Impl,
            "in" => Keyword::In,
            "let" => Keyword::Let,
            "loop" => Keyword::Loop,
            "match" => Keyword::Match,
            "mod" => Keyword::Mod,
            "mut" => Keyword::Mut,
            "pub" => Keyword::Pub,
            "return" => Keyword::Return,
            "return_data" => Keyword::ReturnData,
            "struct" => Keyword::Struct,
            "super" => Keyword::Super,
            "trait" => Keyword::Trait,
            "type" => Keyword::Type,
            "unchecked" => Keyword::Unchecked,
            "unconstrained" => Keyword::Unconstrained,
            "unsafe" => Keyword::Unsafe,
            "use" => Keyword::Use,
            "where" => Keyword::Where,
            "while" => Keyword::While,

            "true" => return Some(Token::Bool(true)),
            "false" => return Some(Token::Bool(false)),
            _ => return None,
        };

        Some(Token::Keyword(keyword))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tokens(pub Vec<LocatedToken>);

#[cfg(test)]
mod keywords {
    use strum::IntoEnumIterator;

    use super::{Keyword, Token};

    #[test]
    fn lookup_consistency() {
        for keyword in Keyword::iter() {
            let resolved_token =
                Keyword::lookup_keyword(&format!("{keyword}")).unwrap_or_else(|| {
                    panic!("Keyword::lookup_keyword couldn't find Keyword {keyword}")
                });

            assert_eq!(
                resolved_token,
                Token::Keyword(keyword),
                "Keyword::lookup_keyword returns unexpected Keyword"
            );
        }
    }
}
