use std::fmt;
use noirc_errors::{Position, Spanned, Span};


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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken(Spanned<Token>);

impl Into<Token> for SpannedToken {
    fn into(self) -> Token {
        self.0.contents
    }
}

impl SpannedToken {
    pub fn into_span(&self) -> Span{
        self.0.span()
    }
    pub fn token(&self) -> &Token{
        &self.0.contents
    }
    pub fn kind(&self) -> TokenKind {
        self.token().kind()
    }
    pub fn is_variant(&self, tok: &Token) -> bool {
       self.token().is_variant(tok)
    }
    pub fn is_comment(&self) -> bool {
        self.token().is_comment()
    }
    pub fn can_start_declaration(&self) -> bool {
        self.token().can_start_declaration()
    }
}

// XXX(low): Need to Add functionality to parse all types of numbers including hex. This would be in the lexer
// XXX(low): Add positional information
// XXX(low): Add a Comment Token to force users to have documentation on public functions
// XXX(med) : Modify Int to use noir_field, so it will be Int(FieldElement)

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
/// All possible tokens allowed in the target language
pub enum Token {
    Ident(String),
    Int(i128),
    Bool(bool),
    Str(String),
    Keyword(Keyword),
    IntType(IntType),
    Comment(String),
    Attribute(Attribute),
    // <
    Less,
    // <=
    LessEqual,
    // >
    Greater,
    // >=
    GreaterEqual,
    // ==
    Equal,
    // !=
    NotEqual,
    // +
    Plus,
    // -
    Minus,
    // *
    Star,
    // /
    Slash,
    // %
    Percent,
    // &
    Ampersand,
    // ^
    Caret,
    // .
    Dot,
    // ..
    DoubleDot,
    // (
    LeftParen,
    // )
    RightParen,
    // {
    LeftBrace,
    // }
    RightBrace,
    // [
    LeftBracket,
    // ]
    RightBracket,
    // ->
    Arrow,
    // |
    Pipe,
    // #
    Pound,
    // ,
    Comma,
    // :
    Colon,
    // ::
    DoubleColon,
    // ;
    Semicolon,
    // !
    Bang,
    // _
    Underscore,
    // =
    Assign,
    Error(String),
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Ident(ref s) => write!(f, "{}", s),
            Token::Int(n) => write!(f, "{}", n),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Str(ref b) => write!(f, "{}", b),
            Token::Comment(ref b) => write!(f, "{}", b),
            Token::Keyword(k) => write!(f, "{}", k),
            Token::Attribute(ref a) => write!(f, "{}", a),
            Token::IntType(ref i) => write!(f, "{}", i),
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
            Token::Underscore => write!(f, "_"),
            Token::Error(ref err) => write!(f, "Error: {}", err),
            Token::EOF => write!(f, ""),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
/// The different kinds of tokens that are possible in the target language
pub enum TokenKind {
    Token(Token),
    Ident,
    Literal,
    Keyword,
    Attribute,
}

impl Token {

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }

    pub fn kind(&self) -> TokenKind {
        match *self {
            Token::Ident(_) => TokenKind::Ident,
            Token::Int(_)
            | Token::Bool(_)
            | Token::Str(_)
            | Token::Keyword(Keyword::Fn)
            | Token::IntType(_) => TokenKind::Literal,
            Token::Keyword(_) => TokenKind::Keyword,
            ref tok => TokenKind::Token(tok.clone()),
        }
    }
    // Does not work for Keyword or whatever is inside of variant
    // XXX: Review the special case of Keyword
    pub fn is_variant(&self, tok: &Token) -> bool {
        let got_variant = core::mem::discriminant(self);
        let expected_variant = core::mem::discriminant(tok);
        let same_token_variant = got_variant == expected_variant;
        if !same_token_variant {
            return false;
        }
        // Check if the keywords are the same
        // Two tokens can be the same variant, but have different inner values
        // We especially care about the Keyword value however
        match (&self, tok) {
            (Token::Keyword(x), Token::Keyword(y)) => return x == y,
            (_, _) => {}
        };

        // If we arrive here, then the Token variants are the same and they are not the Keyword type
        return same_token_variant;
    }
    pub fn is_comment(&self) -> bool {
        match self {
            Token::Comment(_) => true,
            _=> false
        }
    }

    pub(super) fn into_single_span(self,position: Position) -> SpannedToken {
      self.into_span(position, position)
    }
    pub(super) fn into_span(self,start: Position,end: Position) -> SpannedToken {
        SpannedToken(Spanned::from(start, end, self))
    }

}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum IntType {
    Unsigned(u32), // u32 = Unsigned(32)
    Signed(u32),   // i64 = Signed(64)
}

impl fmt::Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IntType::Unsigned(num) => write!(f, "u{}", num),
            IntType::Signed(num) => write!(f, "i{}", num),
        }
    }
}

impl IntType {
    pub(crate) fn lookup_int_type(word: &str) -> Option<Token> {
        // Check if the first string is a 'u' or 'i'

        let is_signed = if word.starts_with("i") {
            true
        } else if word.starts_with("u") {
            false
        } else {
            return None;
        };

        // Word start with 'u' or 'i'. Check if the latter is an integer
        let str_as_u32 = match word[1..].parse::<u32>() {
            Ok(str_as_u32) => str_as_u32,
            Err(_) => return None,
        };

        let max_bits = noir_field::FieldElement::max_num_bits();

        if str_as_u32 > max_bits {
            panic!(
                "The maximum number of bits need to represent a field is {}, {} is too much ",
                max_bits, str_as_u32
            );
        }
        if (str_as_u32 % 2 == 1) && (str_as_u32 > 1) {
            panic!("Barretenberg currently panics on odd integered bit widths such as u3, u5. u1 works as it is a type alias for bool, so we can use a bool gate for it");
        }

        if is_signed {
            return Some(Token::IntType(IntType::Signed(str_as_u32)));
        } else {
            return Some(Token::IntType(IntType::Unsigned(str_as_u32)));
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
// Attributes are special language markers in the target language
// An example of one is `#[sha256]` . Currently only Foreign attributes are supported
// Calls to functions which have the foreign attribute are executed in the host language
pub enum Attribute {
    Foreign(String),
    Builtin(String),
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Attribute::Foreign(ref k) => write!(f, "#[foreign({})]", k),
            Attribute::Builtin(ref k) => write!(f, "#[builtin({})]", k),
        }
    }
}

impl Attribute {
    /// If the string is a fixed attribute return that, else
    /// return the custom attribute
    pub(crate) fn lookup_attribute(word: &str) -> Token {

        let word_segments : Vec<&str> = word
        .split(|c| c == '(' || c == ')')
        .filter(|string_segment| !string_segment.is_empty())
        .collect();

        if word_segments.len() != 2 {
            panic!("Malformed function attribute. An example of an attribute is #[foreign(sha256)]")
        }

        let attribute_type = word_segments[0];
        let attribute_name = word_segments[1];


        match attribute_type {
            "foreign" => Token::Attribute(Attribute::Foreign(attribute_name.to_string())),
            "builtin" => Token::Attribute(Attribute::Builtin(attribute_name.to_string())),
            _=> panic!("unknown attribute type")
        }
    }
    pub fn is_foreign(&self) -> bool{
        match self 
        {
            Attribute::Foreign(_) => true,
            _ => false
        }
    }
    pub fn is_low_level(&self) -> bool{
        match self 
        {
            Attribute::Foreign(_) => true,
            Attribute::Builtin(_) => true,
            _ => false
        }
    }

    pub fn declares_a_low_level_func(&self) -> bool {
        match self {
            Attribute::Builtin(_) => true,
            Attribute::Foreign(_) => true,
        }
    }
}

impl<'a> Into<&'a str> for &'a Attribute {
    fn into(self) -> &'a str {
        match self {
            Attribute::Foreign(string) => &string,
            Attribute::Builtin(string) => &string
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
// Special Keywords allowed in the target language
pub enum Keyword {
    Fn,
    Struct,
    If,
    Mod,
    Else,
    While,
    As,
    For,
    In,
    Use,
    Constrain,
    // You can declare a variable using pub which will give it the Public type
    Pub,
    Public,
    // You can declare a variable using private, which will give it the Witness type
    Private,
    Witness,
    // You can declare a variable using Const which will give it the Constant type
    Const,
    Constant,
    // Let declarations will be for Structures and possibly closures, if they are added
    Let,
    // Field type can only be used in Directive functions. They are explicitly for doing Field operations without applying constraints
    Field,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Keyword::Fn => write!(f, "fn"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::If => write!(f, "if"),
            Keyword::Mod => write!(f, "mod"),
            Keyword::For => write!(f, "for"),
            Keyword::In => write!(f, "in"),
            Keyword::Else => write!(f, "else"),
            Keyword::While => write!(f, "while"),
            Keyword::Constrain => write!(f, "constrain"),
            Keyword::Let => write!(f, "let"),
            Keyword::As => write!(f, "as"),
            Keyword::Use => write!(f, "use"),
            Keyword::Pub => write!(f, "pub"),
            Keyword::Public => write!(f, "Public"),
            Keyword::Private => write!(f, "priv"),
            Keyword::Witness => write!(f, "Witness"),
            Keyword::Constant => write!(f, "Constant"),
            Keyword::Field => write!(f, "Field"),
            Keyword::Const => write!(f, "const"),
        }
    }
}

impl Keyword {
    /// If the string is a keyword, return the associated token
    /// else return None
    pub(crate) fn lookup_keyword(word: &str) -> Option<Token> {
        match word {
            "fn" => Some(Token::Keyword(Keyword::Fn)),
            "struct" => Some(Token::Keyword(Keyword::Struct)),
            "if" => Some(Token::Keyword(Keyword::If)),
            "mod" => Some(Token::Keyword(Keyword::Mod)),
            "for" => Some(Token::Keyword(Keyword::For)),
            "in" => Some(Token::Keyword(Keyword::In)),
            "else" => Some(Token::Keyword(Keyword::Else)),
            "while" => Some(Token::Keyword(Keyword::While)),
            "constrain" => Some(Token::Keyword(Keyword::Constrain)),
            "let" => Some(Token::Keyword(Keyword::Let)),
            "as" => Some(Token::Keyword(Keyword::As)),
            "use" => Some(Token::Keyword(Keyword::Use)),
            "true" => Some(Token::Bool(true)),
            "false" => Some(Token::Bool(false)),

            "priv" => Some(Token::Keyword(Keyword::Private)),
            "pub" => Some(Token::Keyword(Keyword::Pub)),
            "const" => Some(Token::Keyword(Keyword::Const)),
            // Native Types
            "Witness" => Some(Token::Keyword(Keyword::Witness)),
            "Public" => Some(Token::Keyword(Keyword::Public)),
            "Constant" => Some(Token::Keyword(Keyword::Constant)),
            "Field" => Some(Token::Keyword(Keyword::Field)),

            "_" => Some(Token::Underscore),
            _ => None,
        }
    }
}

// The list of keyword tokens which can start "variable" declarations. "fn" is for function declarations
// XXX(low) : It might make sense to create a Keyword::Declarations Variant
const fn declaration_keywords() -> [Keyword; 4] {
    [Keyword::Let, Keyword::Const, Keyword::Pub, Keyword::Private]
}

impl Token {
    /// Converts Token into a declaration keyword
    /// Panics if the token cannot start a declaration
    pub fn to_declaration_keyword(&self) -> Keyword {
        assert!(self.can_start_declaration());
        match self {
            Token::Keyword(kw) => *kw,
            _ => panic!("All tokens which can start declarations, must be keyword"),
        }
    }
    // The set of keyword which can declare variables
    pub fn can_start_declaration(&self) -> bool {
        // First check it is a keyword
        let is_keyword = self.kind() == TokenKind::Keyword;
        if !is_keyword {
            return false;
        }

        match self {
            Token::Keyword(kw) => declaration_keywords().contains(kw),
            _ => false,
        }
    }
}

#[test]
fn test_variant_equality() {
    let tok = Token::Keyword(Keyword::Let);
    let tok2 = Token::Keyword(Keyword::Let);
    assert!(tok.is_variant(&tok2));

    let tok3 = Token::Keyword(Keyword::Const);
    assert!(!tok.is_variant(&tok3));

    let tok4 = Token::LeftBrace;
    assert!(!tok.is_variant(&tok4));
}
