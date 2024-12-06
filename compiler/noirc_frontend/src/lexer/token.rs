use acvm::FieldElement;
use noirc_errors::{Position, Span, Spanned};
use std::fmt::{self, Display};

use crate::{
    ast::{Expression, Path},
    node_interner::{
        ExprId, InternedExpressionKind, InternedPattern, InternedStatementKind,
        InternedUnresolvedTypeData, QuotedTypeId,
    },
};

use super::Lexer;

/// Represents a token in noir's grammar - a word, number,
/// or symbol that can be used in noir's syntax. This is the
/// smallest unit of grammar. A parser may (will) decide to parse
/// items differently depending on the Tokens present but will
/// never parse the same ordering of identical tokens differently.
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum BorrowedToken<'input> {
    Ident(&'input str),
    Int(FieldElement),
    Bool(bool),
    Str(&'input str),
    /// the u8 is the number of hashes, i.e. r###..
    RawStr(&'input str, u8),
    FmtStr(&'input str),
    Keyword(Keyword),
    IntType(IntType),
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
    /// %
    Percent,
    /// &
    Ampersand,
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

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum Token {
    Ident(String),
    Int(FieldElement),
    Bool(bool),
    Str(String),
    /// the u8 is the number of hashes, i.e. r###..
    RawStr(String, u8),
    FmtStr(String),
    Keyword(Keyword),
    IntType(IntType),
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
    /// A reference to an interned `Patter`.
    InternedPattern(InternedPattern),
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
    /// %
    Percent,
    /// &
    Ampersand,
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
        Token::Ident(ref s) => BorrowedToken::Ident(s),
        Token::Int(n) => BorrowedToken::Int(*n),
        Token::Bool(b) => BorrowedToken::Bool(*b),
        Token::Str(ref b) => BorrowedToken::Str(b),
        Token::FmtStr(ref b) => BorrowedToken::FmtStr(b),
        Token::RawStr(ref b, hashes) => BorrowedToken::RawStr(b, *hashes),
        Token::Keyword(k) => BorrowedToken::Keyword(*k),
        Token::AttributeStart { is_inner, is_tag } => {
            BorrowedToken::AttributeStart { is_inner: *is_inner, is_tag: *is_tag }
        }
        Token::LineComment(ref s, _style) => BorrowedToken::LineComment(s, *_style),
        Token::BlockComment(ref s, _style) => BorrowedToken::BlockComment(s, *_style),
        Token::Quote(stream) => BorrowedToken::Quote(stream),
        Token::QuotedType(id) => BorrowedToken::QuotedType(*id),
        Token::InternedExpr(id) => BorrowedToken::InternedExpression(*id),
        Token::InternedStatement(id) => BorrowedToken::InternedStatement(*id),
        Token::InternedLValue(id) => BorrowedToken::InternedLValue(*id),
        Token::InternedUnresolvedTypeData(id) => BorrowedToken::InternedUnresolvedTypeData(*id),
        Token::InternedPattern(id) => BorrowedToken::InternedPattern(*id),
        Token::IntType(ref i) => BorrowedToken::IntType(i.clone()),
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
        Token::Percent => BorrowedToken::Percent,
        Token::Ampersand => BorrowedToken::Ampersand,
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
        Token::Pipe => BorrowedToken::Pipe,
        Token::Pound => BorrowedToken::Pound,
        Token::Comma => BorrowedToken::Comma,
        Token::Colon => BorrowedToken::Colon,
        Token::DoubleColon => BorrowedToken::DoubleColon,
        Token::Semicolon => BorrowedToken::Semicolon,
        Token::Assign => BorrowedToken::Assign,
        Token::Bang => BorrowedToken::Bang,
        Token::DollarSign => BorrowedToken::DollarSign,
        Token::EOF => BorrowedToken::EOF,
        Token::Invalid(c) => BorrowedToken::Invalid(*c),
        Token::Whitespace(ref s) => BorrowedToken::Whitespace(s),
        Token::UnquoteMarker(id) => BorrowedToken::UnquoteMarker(*id),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum DocStyle {
    Outer,
    Inner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpannedToken(Spanned<Token>);

impl PartialEq<SpannedToken> for Token {
    fn eq(&self, other: &SpannedToken) -> bool {
        self == &other.0.contents
    }
}
impl PartialEq<Token> for SpannedToken {
    fn eq(&self, other: &Token) -> bool {
        &self.0.contents == other
    }
}

impl From<SpannedToken> for Token {
    fn from(spt: SpannedToken) -> Self {
        spt.0.contents
    }
}

impl<'a> From<&'a SpannedToken> for &'a Token {
    fn from(spt: &'a SpannedToken) -> Self {
        &spt.0.contents
    }
}

impl SpannedToken {
    pub fn new(token: Token, span: Span) -> SpannedToken {
        SpannedToken(Spanned::from(span, token))
    }
    pub fn to_span(&self) -> Span {
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

impl std::fmt::Display for SpannedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.token().fmt(f)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Ident(ref s) => write!(f, "{s}"),
            Token::Int(n) => write!(f, "{}", n),
            Token::Bool(b) => write!(f, "{b}"),
            Token::Str(ref b) => write!(f, "{b:?}"),
            Token::FmtStr(ref b) => write!(f, "f{b:?}"),
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
            Token::IntType(ref i) => write!(f, "{i}"),
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
            Token::Percent => write!(f, "%"),
            Token::Ampersand => write!(f, "&"),
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
            Token::Pipe => write!(f, "|"),
            Token::Pound => write!(f, "#"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::DoubleColon => write!(f, "::"),
            Token::Semicolon => write!(f, ";"),
            Token::Assign => write!(f, "="),
            Token::Bang => write!(f, "!"),
            Token::DollarSign => write!(f, "$"),
            Token::EOF => write!(f, "end of input"),
            Token::Invalid(c) => write!(f, "{c}"),
            Token::Whitespace(ref s) => write!(f, "{s}"),
            Token::UnquoteMarker(_) => write!(f, "(UnquoteMarker)"),
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

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Token(ref tok) => write!(f, "{tok}"),
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
            Token::Int(_)
            | Token::Bool(_)
            | Token::Str(_)
            | Token::RawStr(..)
            | Token::FmtStr(_) => TokenKind::Literal,
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
    /// a short-hand assignment: a <op>= b
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

impl fmt::Display for IntType {
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
    /// No scope is applied and so the test must pass
    None,
}

impl fmt::Display for TestScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TestScope::None => write!(f, ""),
            TestScope::ShouldFailWith { reason } => match reason {
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
        self.has_secondary_attr(&SecondaryAttribute::ContractLibraryMethod)
    }

    pub fn is_test_function(&self) -> bool {
        matches!(self.function(), Some(FunctionAttribute::Test(_)))
    }

    /// True if these attributes mean the given function is an entry point function if it was
    /// defined within a contract. Note that this does not check if the function is actually part
    /// of a contract.
    pub fn is_contract_entry_point(&self) -> bool {
        !self.has_contract_library_method() && !self.is_test_function()
    }

    /// Returns note if a deprecated secondary attribute is found
    pub fn get_deprecated_note(&self) -> Option<Option<String>> {
        self.secondary.iter().find_map(|attr| match attr {
            SecondaryAttribute::Deprecated(note) => Some(note.clone()),
            _ => None,
        })
    }

    pub fn get_field_attribute(&self) -> Option<String> {
        for secondary in &self.secondary {
            if let SecondaryAttribute::Field(field) = secondary {
                return Some(field.to_lowercase());
            }
        }
        None
    }

    pub fn is_foldable(&self) -> bool {
        self.function().map_or(false, |func_attribute| func_attribute.is_foldable())
    }

    pub fn is_no_predicates(&self) -> bool {
        self.function().map_or(false, |func_attribute| func_attribute.is_no_predicates())
    }

    pub fn has_varargs(&self) -> bool {
        self.has_secondary_attr(&SecondaryAttribute::Varargs)
    }

    pub fn has_use_callers_scope(&self) -> bool {
        self.has_secondary_attr(&SecondaryAttribute::UseCallersScope)
    }

    /// True if the function is marked with an `#[export]` attribute.
    pub fn has_export(&self) -> bool {
        self.has_secondary_attr(&SecondaryAttribute::Export)
    }

    /// Check if secondary attributes contain a specific instance.
    pub fn has_secondary_attr(&self, attr: &SecondaryAttribute) -> bool {
        self.secondary.contains(attr)
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

impl fmt::Display for Attribute {
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
pub enum FunctionAttribute {
    Foreign(String),
    Builtin(String),
    Oracle(String),
    Test(TestScope),
    Fold,
    NoPredicates,
    InlineAlways,
}

impl FunctionAttribute {
    pub fn builtin(&self) -> Option<&String> {
        match self {
            FunctionAttribute::Builtin(name) => Some(name),
            _ => None,
        }
    }

    pub fn foreign(&self) -> Option<&String> {
        match self {
            FunctionAttribute::Foreign(name) => Some(name),
            _ => None,
        }
    }

    pub fn oracle(&self) -> Option<&String> {
        match self {
            FunctionAttribute::Oracle(name) => Some(name),
            _ => None,
        }
    }

    pub fn is_foreign(&self) -> bool {
        matches!(self, FunctionAttribute::Foreign(_))
    }

    pub fn is_oracle(&self) -> bool {
        matches!(self, FunctionAttribute::Oracle(_))
    }

    pub fn is_low_level(&self) -> bool {
        matches!(self, FunctionAttribute::Foreign(_) | FunctionAttribute::Builtin(_))
    }

    pub fn is_foldable(&self) -> bool {
        matches!(self, FunctionAttribute::Fold)
    }

    /// Check whether we have an `inline` attribute
    /// Although we also do not want to inline foldable functions,
    /// we keep the two attributes distinct for clarity.
    pub fn is_no_predicates(&self) -> bool {
        matches!(self, FunctionAttribute::NoPredicates)
    }

    /// Check whether we have an `inline_always` attribute
    /// This is used to indicate that a function should always be inlined
    /// regardless of the target runtime.
    pub fn is_inline_always(&self) -> bool {
        matches!(self, FunctionAttribute::InlineAlways)
    }

    pub fn name(&self) -> &'static str {
        match self {
            FunctionAttribute::Foreign(_) => "foreign",
            FunctionAttribute::Builtin(_) => "builtin",
            FunctionAttribute::Oracle(_) => "oracle",
            FunctionAttribute::Test(_) => "test",
            FunctionAttribute::Fold => "fold",
            FunctionAttribute::NoPredicates => "no_predicates",
            FunctionAttribute::InlineAlways => "inline_always",
        }
    }
}

impl fmt::Display for FunctionAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionAttribute::Test(scope) => write!(f, "#[test{scope}]"),
            FunctionAttribute::Foreign(ref k) => write!(f, "#[foreign({k})]"),
            FunctionAttribute::Builtin(ref k) => write!(f, "#[builtin({k})]"),
            FunctionAttribute::Oracle(ref k) => write!(f, "#[oracle({k})]"),
            FunctionAttribute::Fold => write!(f, "#[fold]"),
            FunctionAttribute::NoPredicates => write!(f, "#[no_predicates]"),
            FunctionAttribute::InlineAlways => write!(f, "#[inline_always]"),
        }
    }
}

/// Secondary attributes are those which a function can have many of.
/// They are not able to change the `FunctionKind` and thus do not have direct impact on the IR output
/// They are often consumed by libraries or used as notices for the developer
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SecondaryAttribute {
    Deprecated(Option<String>),
    // This is an attribute to specify that a function
    // is a helper method for a contract and should not be seen as
    // the entry point.
    ContractLibraryMethod,
    Export,
    Field(String),

    /// A custom tag attribute: #['foo]
    Tag(CustomAttribute),

    /// An attribute expected to run a comptime function of the same name: #[foo]
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
}

impl SecondaryAttribute {
    pub(crate) fn name(&self) -> Option<String> {
        match self {
            SecondaryAttribute::Deprecated(_) => Some("deprecated".to_string()),
            SecondaryAttribute::ContractLibraryMethod => {
                Some("contract_library_method".to_string())
            }
            SecondaryAttribute::Export => Some("export".to_string()),
            SecondaryAttribute::Field(_) => Some("field".to_string()),
            SecondaryAttribute::Tag(custom) => custom.name(),
            SecondaryAttribute::Meta(meta) => Some(meta.name.last_name().to_string()),
            SecondaryAttribute::Abi(_) => Some("abi".to_string()),
            SecondaryAttribute::Varargs => Some("varargs".to_string()),
            SecondaryAttribute::UseCallersScope => Some("use_callers_scope".to_string()),
            SecondaryAttribute::Allow(_) => Some("allow".to_string()),
        }
    }

    pub(crate) fn is_allow_unused_variables(&self) -> bool {
        match self {
            SecondaryAttribute::Allow(string) => string == "unused_variables",
            _ => false,
        }
    }

    pub(crate) fn is_abi(&self) -> bool {
        matches!(self, SecondaryAttribute::Abi(_))
    }

    pub(crate) fn contents(&self) -> String {
        match self {
            SecondaryAttribute::Deprecated(None) => "deprecated".to_string(),
            SecondaryAttribute::Deprecated(Some(ref note)) => {
                format!("deprecated({note:?})")
            }
            SecondaryAttribute::Tag(ref attribute) => format!("'{}", attribute.contents),
            SecondaryAttribute::Meta(ref meta) => meta.to_string(),
            SecondaryAttribute::ContractLibraryMethod => "contract_library_method".to_string(),
            SecondaryAttribute::Export => "export".to_string(),
            SecondaryAttribute::Field(ref k) => format!("field({k})"),
            SecondaryAttribute::Abi(ref k) => format!("abi({k})"),
            SecondaryAttribute::Varargs => "varargs".to_string(),
            SecondaryAttribute::UseCallersScope => "use_callers_scope".to_string(),
            SecondaryAttribute::Allow(ref k) => format!("allow({k})"),
        }
    }
}

impl fmt::Display for SecondaryAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#[{}]", self.contents())
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MetaAttribute {
    pub name: Path,
    pub arguments: Vec<Expression>,
    pub span: Span,
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

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub struct CustomAttribute {
    pub contents: String,
    // The span of the entire attribute, including leading `#[` and trailing `]`
    pub span: Span,
    // The span for the attribute contents (what's inside `#[...]`)
    pub contents_span: Span,
}

impl CustomAttribute {
    fn name(&self) -> Option<String> {
        let mut lexer = Lexer::new(&self.contents);
        let token = lexer.next()?.ok()?;
        if let Token::Ident(ident) = token.into_token() {
            Some(ident)
        } else {
            None
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
    Bool,
    Break,
    CallData,
    Char,
    Comptime,
    Constrain,
    Continue,
    Contract,
    Crate,
    CtString,
    Dep,
    Else,
    Expr,
    Field,
    Fn,
    For,
    FormatString,
    FunctionDefinition,
    Global,
    If,
    Impl,
    In,
    Let,
    Mod,
    Module,
    Mut,
    Pub,
    Quoted,
    Return,
    ReturnData,
    String,
    Struct,
    StructDefinition,
    Super,
    TopLevelItem,
    Trait,
    TraitConstraint,
    TraitDefinition,
    TraitImpl,
    Type,
    TypedExpr,
    TypeType,
    Unchecked,
    Unconstrained,
    UnresolvedType,
    Unsafe,
    Use,
    Where,
    While,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Keyword::As => write!(f, "as"),
            Keyword::Assert => write!(f, "assert"),
            Keyword::AssertEq => write!(f, "assert_eq"),
            Keyword::Bool => write!(f, "bool"),
            Keyword::Break => write!(f, "break"),
            Keyword::Char => write!(f, "char"),
            Keyword::CallData => write!(f, "call_data"),
            Keyword::Comptime => write!(f, "comptime"),
            Keyword::Constrain => write!(f, "constrain"),
            Keyword::Continue => write!(f, "continue"),
            Keyword::Contract => write!(f, "contract"),
            Keyword::Crate => write!(f, "crate"),
            Keyword::CtString => write!(f, "CtString"),
            Keyword::Dep => write!(f, "dep"),
            Keyword::Else => write!(f, "else"),
            Keyword::Expr => write!(f, "Expr"),
            Keyword::Field => write!(f, "Field"),
            Keyword::Fn => write!(f, "fn"),
            Keyword::For => write!(f, "for"),
            Keyword::FormatString => write!(f, "fmtstr"),
            Keyword::FunctionDefinition => write!(f, "FunctionDefinition"),
            Keyword::Global => write!(f, "global"),
            Keyword::If => write!(f, "if"),
            Keyword::Impl => write!(f, "impl"),
            Keyword::In => write!(f, "in"),
            Keyword::Let => write!(f, "let"),
            Keyword::Mod => write!(f, "mod"),
            Keyword::Module => write!(f, "Module"),
            Keyword::Mut => write!(f, "mut"),
            Keyword::Pub => write!(f, "pub"),
            Keyword::Quoted => write!(f, "Quoted"),
            Keyword::Return => write!(f, "return"),
            Keyword::ReturnData => write!(f, "return_data"),
            Keyword::String => write!(f, "str"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::StructDefinition => write!(f, "StructDefinition"),
            Keyword::Super => write!(f, "super"),
            Keyword::TopLevelItem => write!(f, "TopLevelItem"),
            Keyword::Trait => write!(f, "trait"),
            Keyword::TraitConstraint => write!(f, "TraitConstraint"),
            Keyword::TraitDefinition => write!(f, "TraitDefinition"),
            Keyword::TraitImpl => write!(f, "TraitImpl"),
            Keyword::Type => write!(f, "type"),
            Keyword::TypedExpr => write!(f, "TypedExpr"),
            Keyword::TypeType => write!(f, "Type"),
            Keyword::Unchecked => write!(f, "unchecked"),
            Keyword::Unconstrained => write!(f, "unconstrained"),
            Keyword::UnresolvedType => write!(f, "UnresolvedType"),
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
            "bool" => Keyword::Bool,
            "break" => Keyword::Break,
            "call_data" => Keyword::CallData,
            "char" => Keyword::Char,
            "comptime" => Keyword::Comptime,
            "constrain" => Keyword::Constrain,
            "continue" => Keyword::Continue,
            "contract" => Keyword::Contract,
            "crate" => Keyword::Crate,
            "CtString" => Keyword::CtString,
            "dep" => Keyword::Dep,
            "else" => Keyword::Else,
            "Expr" => Keyword::Expr,
            "Field" => Keyword::Field,
            "fn" => Keyword::Fn,
            "for" => Keyword::For,
            "fmtstr" => Keyword::FormatString,
            "FunctionDefinition" => Keyword::FunctionDefinition,
            "global" => Keyword::Global,
            "if" => Keyword::If,
            "impl" => Keyword::Impl,
            "in" => Keyword::In,
            "let" => Keyword::Let,
            "mod" => Keyword::Mod,
            "Module" => Keyword::Module,
            "mut" => Keyword::Mut,
            "pub" => Keyword::Pub,
            "Quoted" => Keyword::Quoted,
            "return" => Keyword::Return,
            "return_data" => Keyword::ReturnData,
            "str" => Keyword::String,
            "struct" => Keyword::Struct,
            "super" => Keyword::Super,
            "TopLevelItem" => Keyword::TopLevelItem,
            "trait" => Keyword::Trait,
            "TraitConstraint" => Keyword::TraitConstraint,
            "TraitDefinition" => Keyword::TraitDefinition,
            "TraitImpl" => Keyword::TraitImpl,
            "type" => Keyword::Type,
            "Type" => Keyword::TypeType,
            "TypedExpr" => Keyword::TypedExpr,
            "StructDefinition" => Keyword::StructDefinition,
            "unchecked" => Keyword::Unchecked,
            "unconstrained" => Keyword::Unconstrained,
            "UnresolvedType" => Keyword::UnresolvedType,
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
pub struct Tokens(pub Vec<SpannedToken>);

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
