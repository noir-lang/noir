use acvm::{acir::AcirField, FieldElement};
use noirc_errors::{Position, Span, Spanned};
use std::{fmt, iter::Map, vec::IntoIter};

use crate::lexer::errors::LexerErrorKind;

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
    Attribute(Attribute),
    LineComment(&'input str, Option<DocStyle>),
    BlockComment(&'input str, Option<DocStyle>),
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
    #[allow(clippy::upper_case_acronyms)]
    EOF,

    Whitespace(&'input str),

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
    Attribute(Attribute),
    LineComment(String, Option<DocStyle>),
    BlockComment(String, Option<DocStyle>),
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
    #[allow(clippy::upper_case_acronyms)]
    EOF,

    Whitespace(String),

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
        Token::Attribute(ref a) => BorrowedToken::Attribute(a.clone()),
        Token::LineComment(ref s, _style) => BorrowedToken::LineComment(s, *_style),
        Token::BlockComment(ref s, _style) => BorrowedToken::BlockComment(s, *_style),
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
        Token::EOF => BorrowedToken::EOF,
        Token::Invalid(c) => BorrowedToken::Invalid(*c),
        Token::Whitespace(ref s) => BorrowedToken::Whitespace(s),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum DocStyle {
    Outer,
    Inner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
            Token::Int(n) => write!(f, "{}", n.to_u128()),
            Token::Bool(b) => write!(f, "{b}"),
            Token::Str(ref b) => write!(f, "{b}"),
            Token::FmtStr(ref b) => write!(f, "f{b}"),
            Token::RawStr(ref b, hashes) => {
                let h: String = std::iter::once('#').cycle().take(hashes as usize).collect();
                write!(f, "r{h}\"{b}\"{h}")
            }
            Token::Keyword(k) => write!(f, "{k}"),
            Token::Attribute(ref a) => write!(f, "{a}"),
            Token::LineComment(ref s, _style) => write!(f, "//{s}"),
            Token::BlockComment(ref s, _style) => write!(f, "/*{s}*/"),
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
            Token::EOF => write!(f, "end of input"),
            Token::Invalid(c) => write!(f, "{c}"),
            Token::Whitespace(ref s) => write!(f, "{s}"),
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
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Token(ref tok) => write!(f, "{tok}"),
            TokenKind::Ident => write!(f, "identifier"),
            TokenKind::Literal => write!(f, "literal"),
            TokenKind::Keyword => write!(f, "keyword"),
            TokenKind::Attribute => write!(f, "attribute"),
        }
    }
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match *self {
            Token::Ident(_) => TokenKind::Ident,
            Token::Int(_)
            | Token::Bool(_)
            | Token::Str(_)
            | Token::RawStr(..)
            | Token::FmtStr(_) => TokenKind::Literal,
            Token::Keyword(_) => TokenKind::Keyword,
            Token::Attribute(_) => TokenKind::Attribute,
            ref tok => TokenKind::Token(tok.clone()),
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
    // Is not the best API. We could split this into two functions. One that checks if the the
    // word is a integer, which only returns an Option
    pub(crate) fn lookup_int_type(word: &str) -> Result<Option<Token>, LexerErrorKind> {
        // Check if the first string is a 'u' or 'i'

        let is_signed = if word.starts_with('i') {
            true
        } else if word.starts_with('u') {
            false
        } else {
            return Ok(None);
        };

        // Word start with 'u' or 'i'. Check if the latter is an integer

        let str_as_u32 = match word[1..].parse::<u32>() {
            Ok(str_as_u32) => str_as_u32,
            Err(_) => return Ok(None),
        };

        if is_signed {
            Ok(Some(Token::IntType(IntType::Signed(str_as_u32))))
        } else {
            Ok(Some(Token::IntType(IntType::Unsigned(str_as_u32))))
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

impl TestScope {
    fn lookup_str(string: &str) -> Option<TestScope> {
        match string.trim() {
            "should_fail" => Some(TestScope::ShouldFailWith { reason: None }),
            s if s.starts_with("should_fail_with") => {
                let parts: Vec<&str> = s.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let reason = parts[1].trim();
                    let reason = reason.trim_matches('"');
                    Some(TestScope::ShouldFailWith { reason: Some(reason.to_string()) })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl fmt::Display for TestScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TestScope::None => write!(f, ""),
            TestScope::ShouldFailWith { reason } => match reason {
                Some(failure_reason) => write!(f, "(should_fail_with = ({failure_reason}))"),
                None => write!(f, "should_fail"),
            },
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
// Attributes are special language markers in the target language
// An example of one is `#[SHA256]` . Currently only Foreign attributes are supported
// Calls to functions which have the foreign attribute are executed in the host language
pub struct Attributes {
    // Each function can have a single Primary Attribute
    pub function: Option<FunctionAttribute>,
    // Each function can have many Secondary Attributes
    pub secondary: Vec<SecondaryAttribute>,
}

impl Attributes {
    pub fn empty() -> Self {
        Self { function: None, secondary: Vec::new() }
    }

    /// Returns true if one of the secondary attributes is `contract_library_method`
    ///
    /// This is useful for finding out if we should compile a contract method
    /// as an entry point or not.
    pub fn has_contract_library_method(&self) -> bool {
        self.secondary
            .iter()
            .any(|attribute| attribute == &SecondaryAttribute::ContractLibraryMethod)
    }

    pub fn is_test_function(&self) -> bool {
        matches!(self.function, Some(FunctionAttribute::Test(_)))
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
        self.function.as_ref().map_or(false, |func_attribute| func_attribute.is_foldable())
    }

    pub fn is_no_predicates(&self) -> bool {
        self.function.as_ref().map_or(false, |func_attribute| func_attribute.is_no_predicates())
    }
}

/// An Attribute can be either a Primary Attribute or a Secondary Attribute
/// A Primary Attribute can alter the function type, thus there can only be one
/// A secondary attribute has no effect and is either consumed by a library or used as a notice for the developer
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
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

impl Attribute {
    /// If the string is a fixed attribute return that, else
    /// return the custom attribute
    pub(crate) fn lookup_attribute(word: &str, span: Span) -> Result<Token, LexerErrorKind> {
        let word_segments: Vec<&str> = word
            .split(|c| c == '(' || c == ')')
            .filter(|string_segment| !string_segment.is_empty())
            .collect();

        let validate = |slice: &str| {
            let is_valid = slice
                .chars()
                .all(|ch| {
                    ch.is_ascii_alphabetic()
                        || ch.is_numeric()
                        || ch.is_ascii_punctuation()
                        || ch == ' '
                })
                .then_some(());

            is_valid.ok_or(LexerErrorKind::MalformedFuncAttribute { span, found: word.to_owned() })
        };

        let attribute = match &word_segments[..] {
            // Primary Attributes
            ["foreign", name] => {
                validate(name)?;
                Attribute::Function(FunctionAttribute::Foreign(name.to_string()))
            }
            ["builtin", name] => {
                validate(name)?;
                Attribute::Function(FunctionAttribute::Builtin(name.to_string()))
            }
            ["oracle", name] => {
                validate(name)?;
                Attribute::Function(FunctionAttribute::Oracle(name.to_string()))
            }
            ["test"] => Attribute::Function(FunctionAttribute::Test(TestScope::None)),
            ["recursive"] => Attribute::Function(FunctionAttribute::Recursive),
            ["fold"] => Attribute::Function(FunctionAttribute::Fold),
            ["no_predicates"] => Attribute::Function(FunctionAttribute::NoPredicates),
            ["test", name] => {
                validate(name)?;
                let malformed_scope =
                    LexerErrorKind::MalformedFuncAttribute { span, found: word.to_owned() };
                match TestScope::lookup_str(name) {
                    Some(scope) => Attribute::Function(FunctionAttribute::Test(scope)),
                    None => return Err(malformed_scope),
                }
            }
            ["field", name] => {
                validate(name)?;
                Attribute::Secondary(SecondaryAttribute::Field(name.to_string()))
            }
            // Secondary attributes
            ["deprecated"] => Attribute::Secondary(SecondaryAttribute::Deprecated(None)),
            ["contract_library_method"] => {
                Attribute::Secondary(SecondaryAttribute::ContractLibraryMethod)
            }
            ["abi", tag] => Attribute::Secondary(SecondaryAttribute::Abi(tag.to_string())),
            ["export"] => Attribute::Secondary(SecondaryAttribute::Export),
            ["deprecated", name] => {
                if !name.starts_with('"') && !name.ends_with('"') {
                    return Err(LexerErrorKind::MalformedFuncAttribute {
                        span,
                        found: word.to_owned(),
                    });
                }

                Attribute::Secondary(SecondaryAttribute::Deprecated(
                    name.trim_matches('"').to_string().into(),
                ))
            }
            tokens => {
                tokens.iter().try_for_each(|token| validate(token))?;
                Attribute::Secondary(SecondaryAttribute::Custom(word.to_owned()))
            }
        };

        Ok(Token::Attribute(attribute))
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
    Recursive,
    Fold,
    NoPredicates,
}

impl FunctionAttribute {
    pub fn builtin(self) -> Option<String> {
        match self {
            FunctionAttribute::Builtin(name) => Some(name),
            _ => None,
        }
    }

    pub fn foreign(self) -> Option<String> {
        match self {
            FunctionAttribute::Foreign(name) => Some(name),
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
}

impl fmt::Display for FunctionAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionAttribute::Test(scope) => write!(f, "#[test{scope}]"),
            FunctionAttribute::Foreign(ref k) => write!(f, "#[foreign({k})]"),
            FunctionAttribute::Builtin(ref k) => write!(f, "#[builtin({k})]"),
            FunctionAttribute::Oracle(ref k) => write!(f, "#[oracle({k})]"),
            FunctionAttribute::Recursive => write!(f, "#[recursive]"),
            FunctionAttribute::Fold => write!(f, "#[fold]"),
            FunctionAttribute::NoPredicates => write!(f, "#[no_predicates]"),
        }
    }
}

/// Secondary attributes are those which a function can have many of.
/// They are not able to change the `FunctionKind` and thus do not have direct impact on the IR output
/// They are often consumed by libraries or used as notices for the developer
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum SecondaryAttribute {
    Deprecated(Option<String>),
    // This is an attribute to specify that a function
    // is a helper method for a contract and should not be seen as
    // the entry point.
    ContractLibraryMethod,
    Export,
    Field(String),
    Custom(String),
    Abi(String),
}

impl fmt::Display for SecondaryAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecondaryAttribute::Deprecated(None) => write!(f, "#[deprecated]"),
            SecondaryAttribute::Deprecated(Some(ref note)) => {
                write!(f, r#"#[deprecated("{note}")]"#)
            }
            SecondaryAttribute::Custom(ref k) => write!(f, "#[{k}]"),
            SecondaryAttribute::ContractLibraryMethod => write!(f, "#[contract_library_method]"),
            SecondaryAttribute::Export => write!(f, "#[export]"),
            SecondaryAttribute::Field(ref k) => write!(f, "#[field({k})]"),
            SecondaryAttribute::Abi(ref k) => write!(f, "#[abi({k})]"),
        }
    }
}

impl AsRef<str> for FunctionAttribute {
    fn as_ref(&self) -> &str {
        match self {
            FunctionAttribute::Foreign(string) => string,
            FunctionAttribute::Builtin(string) => string,
            FunctionAttribute::Oracle(string) => string,
            FunctionAttribute::Test { .. } => "",
            FunctionAttribute::Recursive => "",
            FunctionAttribute::Fold => "",
            FunctionAttribute::NoPredicates => "",
        }
    }
}

impl AsRef<str> for SecondaryAttribute {
    fn as_ref(&self) -> &str {
        match self {
            SecondaryAttribute::Deprecated(Some(string)) => string,
            SecondaryAttribute::Deprecated(None) => "",
            SecondaryAttribute::Custom(string)
            | SecondaryAttribute::Field(string)
            | SecondaryAttribute::Abi(string) => string,
            SecondaryAttribute::ContractLibraryMethod => "",
            SecondaryAttribute::Export => "",
        }
    }
}

/// Note that `self` is not present - it is a contextual keyword rather than a true one as it is
/// only special within `impl`s. Otherwise `self` functions as a normal identifier.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, PartialOrd, Ord)]
#[cfg_attr(test, derive(strum_macros::EnumIter))]
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
    Dep,
    Distinct,
    Else,
    Field,
    Fn,
    For,
    FormatString,
    Global,
    If,
    Impl,
    In,
    Let,
    Mod,
    Mut,
    Pub,
    Quote,
    Return,
    ReturnData,
    String,
    Struct,
    Super,
    Trait,
    Type,
    Unchecked,
    Unconstrained,
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
            Keyword::Dep => write!(f, "dep"),
            Keyword::Distinct => write!(f, "distinct"),
            Keyword::Else => write!(f, "else"),
            Keyword::Field => write!(f, "Field"),
            Keyword::Fn => write!(f, "fn"),
            Keyword::For => write!(f, "for"),
            Keyword::FormatString => write!(f, "fmtstr"),
            Keyword::Global => write!(f, "global"),
            Keyword::If => write!(f, "if"),
            Keyword::Impl => write!(f, "impl"),
            Keyword::In => write!(f, "in"),
            Keyword::Let => write!(f, "let"),
            Keyword::Mod => write!(f, "mod"),
            Keyword::Mut => write!(f, "mut"),
            Keyword::Pub => write!(f, "pub"),
            Keyword::Quote => write!(f, "quote"),
            Keyword::Return => write!(f, "return"),
            Keyword::ReturnData => write!(f, "return_data"),
            Keyword::String => write!(f, "str"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::Super => write!(f, "super"),
            Keyword::Trait => write!(f, "trait"),
            Keyword::Type => write!(f, "type"),
            Keyword::Unchecked => write!(f, "unchecked"),
            Keyword::Unconstrained => write!(f, "unconstrained"),
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
            "dep" => Keyword::Dep,
            "distinct" => Keyword::Distinct,
            "else" => Keyword::Else,
            "Field" => Keyword::Field,
            "fn" => Keyword::Fn,
            "for" => Keyword::For,
            "fmtstr" => Keyword::FormatString,
            "global" => Keyword::Global,
            "if" => Keyword::If,
            "impl" => Keyword::Impl,
            "in" => Keyword::In,
            "let" => Keyword::Let,
            "mod" => Keyword::Mod,
            "mut" => Keyword::Mut,
            "pub" => Keyword::Pub,
            "quote" => Keyword::Quote,
            "return" => Keyword::Return,
            "return_data" => Keyword::ReturnData,
            "str" => Keyword::String,
            "struct" => Keyword::Struct,
            "super" => Keyword::Super,
            "trait" => Keyword::Trait,
            "type" => Keyword::Type,
            "unchecked" => Keyword::Unchecked,
            "unconstrained" => Keyword::Unconstrained,
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

pub struct Tokens(pub Vec<SpannedToken>);

type TokenMapIter = Map<IntoIter<SpannedToken>, fn(SpannedToken) -> (Token, Span)>;

impl<'a> From<Tokens> for chumsky::Stream<'a, Token, Span, TokenMapIter> {
    fn from(tokens: Tokens) -> Self {
        let end_of_input = match tokens.0.last() {
            Some(spanned_token) => spanned_token.to_span(),
            None => Span::single_char(0),
        };

        fn get_span(token: SpannedToken) -> (Token, Span) {
            let span = token.to_span();
            (token.into_token(), span)
        }

        let iter = tokens.0.into_iter().map(get_span as fn(_) -> _);
        chumsky::Stream::from_iter(end_of_input, iter)
    }
}

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
