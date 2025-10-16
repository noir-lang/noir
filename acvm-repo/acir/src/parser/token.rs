use acir_field::FieldElement;
use noirc_span::{Position, Span, Spanned};

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
    /// Reserved identifiers such as `CONSTRAIN`.
    /// Most words in ACIR's human readable are expected to be keywords
    Keyword(Keyword),
    /// Witness index, like `w42`
    Witness(u32),
    /// Block index, like `b42`
    Block(u32),
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
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// =
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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(ident) => write!(f, "{ident}"),
            Token::Keyword(keyword) => write!(f, "{keyword}"),
            Token::Witness(index) => write!(f, "w{index}"),
            Token::Block(index) => write!(f, "b{index}"),
            Token::Int(int) => write!(f, "{int}"),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Equal => write!(f, "="),
            Token::Eof => write!(f, "(end of stream)"),
        }
    }
}

/// ACIR human readable text format keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Keyword {
    /// private
    Private,
    /// parameters
    Parameters,
    /// public
    Public,
    /// return
    Return,
    /// value
    Value,
    /// values
    Values,
    /// ASSERT
    Assert,
    /// BLACKBOX
    BlackBoxFuncCall,
    /// INIT
    MemoryInit,
    /// READ
    MemoryRead,
    /// WRITE
    MemoryWrite,
    /// BRILLIG
    Brillig,
    /// CALL
    Call,
    /// predicate
    Predicate,
    /// CALLDATA
    CallData,
    /// RETURNDATA
    ReturnData,
    /// func
    Function,
    /// input
    Input,
    /// inputs
    Inputs,
    /// output
    Output,
    /// outputs
    Outputs,
    /// bits
    Bits,
}

impl Keyword {
    pub(super) fn lookup_keyword(word: &str) -> Option<Token> {
        let keyword = match word {
            "private" => Keyword::Private,
            "parameters" => Keyword::Parameters,
            "public" => Keyword::Public,
            "return" => Keyword::Return,
            "value" => Keyword::Value,
            "values" => Keyword::Values,
            "ASSERT" => Keyword::Assert,
            "BLACKBOX" => Keyword::BlackBoxFuncCall,
            "INIT" => Keyword::MemoryInit,
            "READ" => Keyword::MemoryRead,
            "WRITE" => Keyword::MemoryWrite,
            "BRILLIG" => Keyword::Brillig,
            "CALL" => Keyword::Call,
            "predicate" => Keyword::Predicate,
            "CALLDATA" => Keyword::CallData,
            "RETURNDATA" => Keyword::ReturnData,
            "func" => Keyword::Function,
            "input" => Keyword::Input,
            "inputs" => Keyword::Inputs,
            "output" => Keyword::Output,
            "outputs" => Keyword::Outputs,
            "bits" => Keyword::Bits,
            _ => return None,
        };
        Some(Token::Keyword(keyword))
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Private => write!(f, "private"),
            Keyword::Parameters => write!(f, "parameters"),
            Keyword::Public => write!(f, "public"),
            Keyword::Return => write!(f, "return"),
            Keyword::Value => write!(f, "value"),
            Keyword::Values => write!(f, "values"),
            Keyword::Assert => write!(f, "ASSERT"),
            Keyword::BlackBoxFuncCall => write!(f, "BLACKBOX"),
            Keyword::MemoryInit => write!(f, "INIT"),
            Keyword::MemoryRead => write!(f, "READ"),
            Keyword::MemoryWrite => write!(f, "WRITE"),
            Keyword::Brillig => write!(f, "BRILLIG"),
            Keyword::Call => write!(f, "CALL"),
            Keyword::Predicate => write!(f, "predicate"),
            Keyword::CallData => write!(f, "CALLDATA"),
            Keyword::ReturnData => write!(f, "RETURNDATA"),
            Keyword::Function => write!(f, "func"),
            Keyword::Input => write!(f, "input"),
            Keyword::Inputs => write!(f, "inputs"),
            Keyword::Output => write!(f, "output"),
            Keyword::Outputs => write!(f, "outputs"),
            Keyword::Bits => write!(f, "bits"),
        }
    }
}
