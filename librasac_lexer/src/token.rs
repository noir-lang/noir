use std::fmt;

// XXX(low): Need to Add functionality to parse all types of numbers including hex. This would be in the lexer
// XXX(low): Add positional information
// XXX(low): Add a Comment Token to force users to have documentation on public functions
// XXX(med) : Modify Int to use rasa_field, so it will be Int(FieldElement)

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
/// All possible tokens allowed in the target language
pub enum Token {
    Ident(String),
    Int(i128),
    Bool(bool),
    Str(String),
    Keyword(Keyword),
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
    // .
    Dot,
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
            Token::Keyword(k) => write!(f, "{}", k),
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
            Token::Dot => write!(f, "."),
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
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match *self {
            Token::Ident(_) => TokenKind::Ident,
            Token::Int(_) | Token::Bool(_) | Token::Str(_) | Token::Keyword(Keyword::Fn) => {
                TokenKind::Literal
            }
            Token::Keyword(_) => TokenKind::Keyword,
            ref tok => TokenKind::Token(tok.clone()),
        }
    }
    // Does not work for Keyword or whatever is inside of variant
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
}


#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
// Special Keywords allowed in the target language
pub enum Keyword {
    Fn,
    Struct,
    If,
    Else,
    While,
    Continue,
    Break,
    Return,
    As,
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
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Keyword::Fn => write!(f, "fn"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::If => write!(f, "if"),
            Keyword::Else => write!(f, "else"),
            Keyword::While => write!(f, "while"),
            Keyword::Continue => write!(f, "continue"),
            Keyword::Break => write!(f, "break"),
            Keyword::Constrain => write!(f, "constrain"),
            Keyword::Let => write!(f, "let"),
            Keyword::Return => write!(f, "return"),
            Keyword::As => write!(f, "as"),
            Keyword::Pub => write!(f, "pub"),
            Keyword::Public => write!(f, "Public"),
            Keyword::Private => write!(f, "priv"),
            Keyword::Witness => write!(f, "Witness"),
            Keyword::Constant => write!(f, "Constant"),
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
            "else" => Some(Token::Keyword(Keyword::Else)),
            "while" => Some(Token::Keyword(Keyword::While)),
            "continue" => Some(Token::Keyword(Keyword::Continue)),
            "break" => Some(Token::Keyword(Keyword::Break)),
            "constrain" => Some(Token::Keyword(Keyword::Constrain)),
            "let" => Some(Token::Keyword(Keyword::Let)),
            "return" => Some(Token::Keyword(Keyword::Return)),
            "as" => Some(Token::Keyword(Keyword::As)),
            "true" => Some(Token::Bool(true)),
            "false" => Some(Token::Bool(false)),
            
            "priv" => Some(Token::Keyword(Keyword::Private)),
            "pub" => Some(Token::Keyword(Keyword::Pub)),
            "const" => Some(Token::Keyword(Keyword::Const)),
            // Native Types
            "Witness" => Some(Token::Keyword(Keyword::Witness)),
            "Public" => Some(Token::Keyword(Keyword::Public)),
            "Constant" => Some(Token::Keyword(Keyword::Constant)),
            _ => None,
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
