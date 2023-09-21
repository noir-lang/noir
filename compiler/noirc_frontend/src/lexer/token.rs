use acvm::FieldElement;
use noirc_errors::{Position, Span, Spanned};
use std::{fmt, iter::Map, vec::IntoIter};

use crate::lexer::errors::LexerErrorKind;

/// Represents a token in noir's grammar - a word, number,
/// or symbol that can be used in noir's syntax. This is the
/// smallest unit of grammar. A parser may (will) decide to parse
/// items differently depending on the Tokens present but will
/// never parse the same ordering of identical tokens differently.
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub enum Token {
    Ident(String),
    Int(FieldElement),
    Bool(bool),
    Str(String),
    FmtStr(String),
    Keyword(Keyword),
    IntType(IntType),
    Attribute(Attribute),
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

    /// An invalid character is one that is not in noir's language or grammar.
    ///
    /// We don't report invalid tokens in the source as errors until parsing to
    /// avoid reporting the error twice (once while lexing, again when it is encountered
    /// during parsing). Reporting during lexing then removing these from the token stream
    /// would not be equivalent as it would change the resulting parse.
    Invalid(char),
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
            Token::Keyword(k) => write!(f, "{k}"),
            Token::Attribute(ref a) => write!(f, "{a}"),
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
            Token::Int(_) | Token::Bool(_) | Token::Str(_) | Token::FmtStr(_) => TokenKind::Literal,
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

    pub fn try_into_binary_op(self, span: Span) -> Option<Spanned<crate::BinaryOpKind>> {
        use crate::BinaryOpKind::*;
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
    pub(crate) fn lookup_int_type(word: &str, span: Span) -> Result<Option<Token>, LexerErrorKind> {
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

        let max_bits = FieldElement::max_num_bits();

        if str_as_u32 > max_bits {
            return Err(LexerErrorKind::TooManyBits { span, max: max_bits, got: str_as_u32 });
        }

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

    /// Returns note if a deprecated secondary attribute is found
    pub fn get_deprecated_note(&self) -> Option<Option<String>> {
        self.secondary.iter().find_map(|attr| match attr {
            SecondaryAttribute::Deprecated(note) => Some(note.clone()),
            _ => None,
        })
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
            Attribute::Function(attribute) => write!(f, "{}", attribute),
            Attribute::Secondary(attribute) => write!(f, "{}", attribute),
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
                        || ch == '_'
                        || ch == '('
                        || ch == ')'
                        || ch == '='
                        || ch == '"'
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
            ["test", name] => {
                validate(name)?;
                let malformed_scope =
                    LexerErrorKind::MalformedFuncAttribute { span, found: word.to_owned() };
                match TestScope::lookup_str(name) {
                    Some(scope) => Attribute::Function(FunctionAttribute::Test(scope)),
                    None => return Err(malformed_scope),
                }
            }
            // Secondary attributes
            ["deprecated"] => Attribute::Secondary(SecondaryAttribute::Deprecated(None)),
            ["contract_library_method"] => {
                Attribute::Secondary(SecondaryAttribute::ContractLibraryMethod)
            }
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

    pub fn is_low_level(&self) -> bool {
        matches!(self, FunctionAttribute::Foreign(_) | FunctionAttribute::Builtin(_))
    }
}

impl fmt::Display for FunctionAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionAttribute::Test(scope) => write!(f, "#[test{}]", scope),
            FunctionAttribute::Foreign(ref k) => write!(f, "#[foreign({k})]"),
            FunctionAttribute::Builtin(ref k) => write!(f, "#[builtin({k})]"),
            FunctionAttribute::Oracle(ref k) => write!(f, "#[oracle({k})]"),
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
    Custom(String),
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
        }
    }
}

impl AsRef<str> for SecondaryAttribute {
    fn as_ref(&self) -> &str {
        match self {
            SecondaryAttribute::Deprecated(Some(string)) => string,
            SecondaryAttribute::Deprecated(None) => "",
            SecondaryAttribute::Custom(string) => string,
            SecondaryAttribute::ContractLibraryMethod => "",
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
    Char,
    CompTime,
    Constrain,
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
    Internal,
    Let,
    Mod,
    Mut,
    Open,
    Pub,
    Return,
    String,
    Struct,
    Trait,
    Type,
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
            Keyword::Char => write!(f, "char"),
            Keyword::CompTime => write!(f, "comptime"),
            Keyword::Constrain => write!(f, "constrain"),
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
            Keyword::Internal => write!(f, "internal"),
            Keyword::Let => write!(f, "let"),
            Keyword::Mod => write!(f, "mod"),
            Keyword::Mut => write!(f, "mut"),
            Keyword::Open => write!(f, "open"),
            Keyword::Pub => write!(f, "pub"),
            Keyword::Return => write!(f, "return"),
            Keyword::String => write!(f, "str"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::Trait => write!(f, "trait"),
            Keyword::Type => write!(f, "type"),
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
            "char" => Keyword::Char,
            "comptime" => Keyword::CompTime,
            "constrain" => Keyword::Constrain,
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
            "internal" => Keyword::Internal,
            "let" => Keyword::Let,
            "mod" => Keyword::Mod,
            "mut" => Keyword::Mut,
            "open" => Keyword::Open,
            "pub" => Keyword::Pub,
            "return" => Keyword::Return,
            "str" => Keyword::String,
            "struct" => Keyword::Struct,
            "trait" => Keyword::Trait,
            "type" => Keyword::Type,
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

#[cfg(test)]
mod keywords {
    use strum::IntoEnumIterator;

    use super::{Keyword, Token};

    #[test]
    fn lookup_consistency() {
        for keyword in Keyword::iter() {
            let resolved_token =
                Keyword::lookup_keyword(&format!("{keyword}")).unwrap_or_else(|| {
                    panic!("Keyword::lookup_keyword couldn't find Keyword {}", keyword)
                });

            assert_eq!(
                resolved_token,
                Token::Keyword(keyword),
                "Keyword::lookup_keyword returns unexpected Keyword"
            );
        }
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
