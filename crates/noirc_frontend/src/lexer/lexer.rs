use super::{
    errors::LexerErrorKind,
    token::{Attribute, IntType, Keyword, SpannedToken, Token, Tokens},
};
use acvm::FieldElement;
use noirc_errors::{Position, Span};
use std::str::Chars;
use std::{
    iter::{Peekable, Zip},
    ops::RangeFrom,
};

/// The job of the lexer is to transform an iterator of characters (`char_iter`)
/// into an iterator of `SpannedToken`. Each `Token` corresponds roughly to 1 word or operator.
/// Tokens are tagged with their location in the source file (a `Span`) for use in error reporting.
pub struct Lexer<'a> {
    char_iter: Peekable<Zip<Chars<'a>, RangeFrom<u32>>>,
    position: Position,
    done: bool,
}

pub type SpannedTokenResult = Result<SpannedToken, LexerErrorKind>;

impl<'a> Lexer<'a> {
    /// Given a source file of noir code, return all the tokens in the file
    /// in order, along with any lexing errors that occurred.
    pub fn lex(source: &'a str) -> (Tokens, Vec<LexerErrorKind>) {
        let lexer = Lexer::new(source);
        let mut tokens = vec![];
        let mut errors = vec![];
        for result in lexer {
            match result {
                Ok(token) => tokens.push(token),
                Err(error) => errors.push(error),
            }
        }
        (Tokens(tokens), errors)
    }

    fn new(source: &'a str) -> Self {
        Lexer {
            // We zip with the character index here to ensure the first char has index 0
            char_iter: source.chars().zip(0..).peekable(),
            position: 0,
            done: false,
        }
    }

    /// Iterates the cursor and returns the char at the new cursor position
    fn next_char(&mut self) -> Option<char> {
        let (c, index) = self.char_iter.next()?;
        self.position = index;
        Some(c)
    }

    /// Peeks at the next char. Does not iterate the cursor
    fn peek_char(&mut self) -> Option<char> {
        self.char_iter.peek().map(|(c, _)| *c)
    }

    /// Peeks at the next char and returns true if it is equal to the char argument
    fn peek_char_is(&mut self, ch: char) -> bool {
        self.peek_char() == Some(ch)
    }

    fn ampersand(&mut self) -> SpannedTokenResult {
        if self.peek_char_is('&') {
            // When we issue this error the first '&' will already be consumed
            // and the next token issued will be the next '&'.
            let span = Span::inclusive(self.position, self.position + 1);
            Err(LexerErrorKind::LogicalAnd { span })
        } else {
            self.single_char_token(Token::Ampersand)
        }
    }

    fn next_token(&mut self) -> SpannedTokenResult {
        match self.next_char() {
            Some(x) if { x.is_whitespace() } => {
                self.eat_whitespace();
                self.next_token()
            }
            Some('<') => self.glue(Token::Less),
            Some('>') => self.glue(Token::Greater),
            Some('=') => self.glue(Token::Assign),
            Some('/') => self.glue(Token::Slash),
            Some('.') => self.glue(Token::Dot),
            Some(':') => self.glue(Token::Colon),
            Some('!') => self.glue(Token::Bang),
            Some('-') => self.glue(Token::Minus),
            Some('&') => self.ampersand(),
            Some('|') => self.single_char_token(Token::Pipe),
            Some('%') => self.single_char_token(Token::Percent),
            Some('^') => self.single_char_token(Token::Caret),
            Some(';') => self.single_char_token(Token::Semicolon),
            Some('*') => self.single_char_token(Token::Star),
            Some('(') => self.single_char_token(Token::LeftParen),
            Some(')') => self.single_char_token(Token::RightParen),
            Some(',') => self.single_char_token(Token::Comma),
            Some('+') => self.single_char_token(Token::Plus),
            Some('{') => self.single_char_token(Token::LeftBrace),
            Some('}') => self.single_char_token(Token::RightBrace),
            Some('[') => self.single_char_token(Token::LeftBracket),
            Some(']') => self.single_char_token(Token::RightBracket),
            Some('"') => self.eat_string_literal(),
            Some('f') => self.eat_format_string_or_alpha_numeric(),
            Some('#') => self.eat_attribute(),
            Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' => self.eat_alpha_numeric(ch),
            Some(ch) => {
                // We don't report invalid tokens in the source as errors until parsing to
                // avoid reporting the error twice. See the note on Token::Invalid's documentation for details.
                Ok(Token::Invalid(ch).into_single_span(self.position))
            }
            None => {
                self.done = true;
                Ok(Token::EOF.into_single_span(self.position))
            }
        }
    }

    fn single_char_token(&self, token: Token) -> SpannedTokenResult {
        Ok(token.into_single_span(self.position))
    }

    fn single_double_peek_token(
        &mut self,
        character: char,
        single: Token,
        double: Token,
    ) -> SpannedTokenResult {
        let start = self.position;

        match self.peek_char_is(character) {
            false => Ok(single.into_single_span(start)),
            true => {
                self.next_char();
                Ok(double.into_span(start, start + 1))
            }
        }
    }

    /// Given that some tokens can contain two characters, such as <= , !=, >=
    /// Glue will take the first character of the token and check if it can be glued onto the next character
    /// forming a double token
    fn glue(&mut self, prev_token: Token) -> SpannedTokenResult {
        let spanned_prev_token = prev_token.clone().into_single_span(self.position);
        match prev_token {
            Token::Dot => self.single_double_peek_token('.', prev_token, Token::DoubleDot),
            Token::Less => {
                let start = self.position;
                if self.peek_char_is('=') {
                    self.next_char();
                    Ok(Token::LessEqual.into_span(start, start + 1))
                } else if self.peek_char_is('<') {
                    self.next_char();
                    Ok(Token::ShiftLeft.into_span(start, start + 1))
                } else {
                    Ok(prev_token.into_single_span(start))
                }
            }
            Token::Greater => {
                let start = self.position;
                if self.peek_char_is('=') {
                    self.next_char();
                    Ok(Token::GreaterEqual.into_span(start, start + 1))
                    // Note: There is deliberately no case for RightShift. We always lex >> as
                    // two separate Greater tokens to help the parser parse nested generic types.
                } else {
                    Ok(prev_token.into_single_span(start))
                }
            }
            Token::Bang => self.single_double_peek_token('=', prev_token, Token::NotEqual),
            Token::Assign => self.single_double_peek_token('=', prev_token, Token::Equal),
            Token::Minus => self.single_double_peek_token('>', prev_token, Token::Arrow),
            Token::Colon => self.single_double_peek_token(':', prev_token, Token::DoubleColon),
            Token::Slash => {
                if self.peek_char_is('/') {
                    self.next_char();
                    return self.parse_comment();
                } else if self.peek_char_is('*') {
                    self.next_char();
                    return self.parse_block_comment();
                }
                Ok(spanned_prev_token)
            }
            _ => Err(LexerErrorKind::NotADoubleChar {
                span: Span::single_char(self.position),
                found: prev_token,
            }),
        }
    }

    /// Keeps consuming tokens as long as the predicate is satisfied
    fn eat_while<F: Fn(char) -> bool>(
        &mut self,
        initial_char: Option<char>,
        predicate: F,
    ) -> String {
        // This function is only called when we want to continue consuming a character of the same type.
        // For example, we see a digit and we want to consume the whole integer
        // Therefore, the current character which triggered this function will need to be appended
        let mut word = String::new();
        if let Some(init_char) = initial_char {
            word.push(init_char);
        }

        // Keep checking that we are not at the EOF
        while let Some(peek_char) = self.peek_char() {
            // Then check for the predicate, if predicate matches append char and increment the cursor
            // If not, return word. The next character will be analyzed on the next iteration of next_token,
            // Which will increment the cursor
            if !predicate(peek_char) {
                return word;
            }
            word.push(peek_char);

            // If we arrive at this point, then the char has been added to the word and we should increment the cursor
            self.next_char();
        }

        word
    }

    fn eat_alpha_numeric(&mut self, initial_char: char) -> SpannedTokenResult {
        match initial_char {
            'A'..='Z' | 'a'..='z' | '_' => Ok(self.eat_word(initial_char)?),
            '0'..='9' => self.eat_digit(initial_char),
            _ => Err(LexerErrorKind::UnexpectedCharacter {
                span: Span::single_char(self.position),
                found: initial_char.into(),
                expected: "an alpha numeric character".to_owned(),
            }),
        }
    }

    fn eat_attribute(&mut self) -> SpannedTokenResult {
        let start = self.position;

        if !self.peek_char_is('[') {
            return Err(LexerErrorKind::UnexpectedCharacter {
                span: Span::single_char(self.position),
                found: self.next_char(),
                expected: "[".to_owned(),
            });
        }
        self.next_char();

        let word = self.eat_while(None, |ch| ch != ']');

        if !self.peek_char_is(']') {
            return Err(LexerErrorKind::UnexpectedCharacter {
                span: Span::single_char(self.position),
                expected: "]".to_owned(),
                found: self.next_char(),
            });
        }
        self.next_char();

        let end = self.position;

        let attribute = Attribute::lookup_attribute(&word, Span::inclusive(start, end))?;

        Ok(attribute.into_span(start, end))
    }

    //XXX(low): Can increase performance if we use iterator semantic and utilize some of the methods on String. See below
    // https://doc.rust-lang.org/stable/std/primitive.str.html#method.rsplit
    fn eat_word(&mut self, initial_char: char) -> SpannedTokenResult {
        let start = self.position;

        let word = self.eat_while(Some(initial_char), |ch| {
            ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_'
        });

        let end = self.position;

        // Check if word either an identifier or a keyword
        if let Some(keyword_token) = Keyword::lookup_keyword(&word) {
            return Ok(keyword_token.into_span(start, end));
        }

        // Check if word an int type
        // if no error occurred, then it is either a valid integer type or it is not an int type
        let parsed_token = IntType::lookup_int_type(&word, Span::inclusive(start, end))?;

        // Check if it is an int type
        if let Some(int_type_token) = parsed_token {
            return Ok(int_type_token.into_span(start, end));
        }

        // Else it is just an identifier
        let ident_token = Token::Ident(word);
        Ok(ident_token.into_span(start, end))
    }

    fn eat_digit(&mut self, initial_char: char) -> SpannedTokenResult {
        let start = self.position;

        let integer_str = self.eat_while(Some(initial_char), |ch| {
            ch.is_ascii_digit() | ch.is_ascii_hexdigit() | (ch == 'x')
        });

        let end = self.position;

        let integer = match FieldElement::try_from_str(&integer_str) {
            None => {
                return Err(LexerErrorKind::InvalidIntegerLiteral {
                    span: Span::inclusive(start, end),
                    found: integer_str,
                })
            }
            Some(integer) => integer,
        };

        let integer_token = Token::Int(integer);
        Ok(integer_token.into_span(start, end))
    }

    fn eat_string_literal(&mut self) -> SpannedTokenResult {
        let start = self.position;

        let str_literal = self.eat_while(None, |ch| ch != '"');

        let str_literal_token = Token::Str(str_literal);

        self.next_char(); // Advance past the closing quote

        let end = self.position;
        Ok(str_literal_token.into_span(start, end))
    }

    // This differs from `eat_string_literal` in that we want the leading `f` to be captured in the Span
    fn eat_fmt_string(&mut self) -> SpannedTokenResult {
        let start = self.position;

        self.next_char();

        let str_literal = self.eat_while(None, |ch| ch != '"');

        let str_literal_token = Token::FmtStr(str_literal);

        self.next_char(); // Advance past the closing quote

        let end = self.position;
        Ok(str_literal_token.into_span(start, end))
    }

    fn eat_format_string_or_alpha_numeric(&mut self) -> SpannedTokenResult {
        if self.peek_char_is('"') {
            self.eat_fmt_string()
        } else {
            self.eat_alpha_numeric('f')
        }
    }

    fn parse_comment(&mut self) -> SpannedTokenResult {
        let _ = self.eat_while(None, |ch| ch != '\n');
        self.next_token()
    }

    fn parse_block_comment(&mut self) -> SpannedTokenResult {
        let start = self.position;
        let mut depth = 1usize;

        while let Some(ch) = self.next_char() {
            match ch {
                '/' if self.peek_char_is('*') => {
                    self.next_char();
                    depth += 1;
                }
                '*' if self.peek_char_is('/') => {
                    self.next_char();
                    depth -= 1;

                    // This block comment is closed, so for a construction like "/* */ */"
                    // there will be a successfully parsed block comment "/* */"
                    // and " */" will be processed separately.
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth == 0 {
            self.next_token()
        } else {
            let span = Span::inclusive(start, self.position);
            Err(LexerErrorKind::UnterminatedBlockComment { span })
        }
    }

    /// Skips white space. They are not significant in the source language
    fn eat_whitespace(&mut self) {
        self.eat_while(None, |ch| ch.is_whitespace());
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = SpannedTokenResult;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            Some(self.next_token())
        }
    }
}

#[test]
fn test_single_double_char() {
    let input = "! != + ( ) { } [ ] | , ; : :: < <= > >= & - -> . .. % / * = == << >>";

    let expected = vec![
        Token::Bang,
        Token::NotEqual,
        Token::Plus,
        Token::LeftParen,
        Token::RightParen,
        Token::LeftBrace,
        Token::RightBrace,
        Token::LeftBracket,
        Token::RightBracket,
        Token::Pipe,
        Token::Comma,
        Token::Semicolon,
        Token::Colon,
        Token::DoubleColon,
        Token::Less,
        Token::LessEqual,
        Token::Greater,
        Token::GreaterEqual,
        Token::Ampersand,
        Token::Minus,
        Token::Arrow,
        Token::Dot,
        Token::DoubleDot,
        Token::Percent,
        Token::Slash,
        Token::Star,
        Token::Assign,
        Token::Equal,
        Token::ShiftLeft,
        Token::Greater,
        Token::Greater,
        Token::EOF,
    ];

    let mut lexer = Lexer::new(input);

    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}

#[test]
fn invalid_attribute() {
    let input = "#";
    let mut lexer = Lexer::new(input);

    let token = lexer.next().unwrap();
    assert!(token.is_err());
}

#[test]
fn deprecated_attribute() {
    let input = r#"#[deprecated]"#;
    let mut lexer = Lexer::new(input);

    let token = lexer.next().unwrap().unwrap();
    assert_eq!(token.token(), &Token::Attribute(Attribute::Deprecated(None)));
}

#[test]
fn deprecated_attribute_with_note() {
    let input = r#"#[deprecated("hello")]"#;
    let mut lexer = Lexer::new(input);

    let token = lexer.next().unwrap().unwrap();
    assert_eq!(token.token(), &Token::Attribute(Attribute::Deprecated("hello".to_string().into())));
}

#[test]
fn test_custom_gate_syntax() {
    let input = "#[foreign(sha256)]#[foreign(blake2s)]#[builtin(sum)]";

    let expected = vec![
        Token::Attribute(Attribute::Foreign("sha256".to_string())),
        Token::Attribute(Attribute::Foreign("blake2s".to_string())),
        Token::Attribute(Attribute::Builtin("sum".to_string())),
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}

#[test]
fn test_int_type() {
    let input = "u16 i16 i108 u104.5";

    let expected = vec![
        Token::IntType(IntType::Unsigned(16)),
        Token::IntType(IntType::Signed(16)),
        Token::IntType(IntType::Signed(108)),
        Token::IntType(IntType::Unsigned(104)),
        Token::Dot,
        Token::Int(5_i128.into()),
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}

#[test]
fn test_arithmetic_sugar() {
    let input = "+= -= *= /= %=";

    let expected = vec![
        Token::Plus,
        Token::Assign,
        Token::Minus,
        Token::Assign,
        Token::Star,
        Token::Assign,
        Token::Slash,
        Token::Assign,
        Token::Percent,
        Token::Assign,
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}

#[test]
fn unterminated_block_comment() {
    let input = "/*/";

    let mut lexer = Lexer::new(input);
    let token = lexer.next().unwrap();

    assert!(token.is_err());
}

#[test]
fn test_comment() {
    let input = "// hello
        let x = 5
    ";

    let expected = vec![
        Token::Keyword(Keyword::Let),
        Token::Ident("x".to_string()),
        Token::Assign,
        Token::Int(FieldElement::from(5_i128)),
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let first_lexer_output = lexer.next_token().unwrap();
        assert_eq!(first_lexer_output, token);
    }
}

#[test]
fn test_block_comment() {
    let input = "
    /* comment */
    let x = 5
    /* comment */
    ";

    let expected = vec![
        Token::Keyword(Keyword::Let),
        Token::Ident("x".to_string()),
        Token::Assign,
        Token::Int(FieldElement::from(5_i128)),
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let first_lexer_output = lexer.next_token().unwrap();
        assert_eq!(first_lexer_output, token);
    }
}

#[test]
fn test_nested_block_comments() {
    let input = "
    /*   /* */  /** */  /*! */  */
    let x = 5
    /*   /* */  /** */  /*! */  */
    ";

    let expected = vec![
        Token::Keyword(Keyword::Let),
        Token::Ident("x".to_string()),
        Token::Assign,
        Token::Int(FieldElement::from(5_i128)),
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let first_lexer_output = lexer.next_token().unwrap();
        assert_eq!(first_lexer_output, token);
    }
}
#[test]
fn test_eat_string_literal() {
    let input = "let _word = \"hello\"";

    let expected = vec![
        Token::Keyword(Keyword::Let),
        Token::Ident("_word".to_string()),
        Token::Assign,
        Token::Str("hello".to_string()),
    ];
    let mut lexer = Lexer::new(input);

    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}

#[test]
fn test_eat_hex_int() {
    let input = "0x05";

    let expected = vec![Token::Int(5_i128.into())];
    let mut lexer = Lexer::new(input);

    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}

#[test]
fn test_span() {
    let input = "let x = 5";

    // Let
    let start_position = Position::default();
    let let_position = start_position + 2;
    let let_token = Token::Keyword(Keyword::Let).into_span(start_position, let_position);

    // Skip whitespace
    let whitespace_position = let_position + 1;

    // Identifier position
    let ident_position = whitespace_position + 1;
    let ident_token = Token::Ident("x".to_string()).into_single_span(ident_position);

    // Skip whitespace
    let whitespace_position = ident_position + 1;

    // Assign position
    let assign_position = whitespace_position + 1;
    let assign_token = Token::Assign.into_single_span(assign_position);

    // Skip whitespace
    let whitespace_position = assign_position + 1;

    // Int position
    let int_position = whitespace_position + 1;
    let int_token = Token::Int(5_i128.into()).into_single_span(int_position);

    let expected = vec![let_token, ident_token, assign_token, int_token];
    let mut lexer = Lexer::new(input);

    for spanned_token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got.to_span(), spanned_token.to_span());
        assert_eq!(got, spanned_token);
    }
}

#[test]
fn test_basic_language_syntax() {
    let input = "
        let five = 5;
        let ten : Field = 10;
        let mul = fn(x, y) {
            x * y;
        };
        constrain mul(five, ten) == 50;
        assert(ten + five == 15);
    ";

    let expected = vec![
        Token::Keyword(Keyword::Let),
        Token::Ident("five".to_string()),
        Token::Assign,
        Token::Int(5_i128.into()),
        Token::Semicolon,
        Token::Keyword(Keyword::Let),
        Token::Ident("ten".to_string()),
        Token::Colon,
        Token::Keyword(Keyword::Field),
        Token::Assign,
        Token::Int(10_i128.into()),
        Token::Semicolon,
        Token::Keyword(Keyword::Let),
        Token::Ident("mul".to_string()),
        Token::Assign,
        Token::Keyword(Keyword::Fn),
        Token::LeftParen,
        Token::Ident("x".to_string()),
        Token::Comma,
        Token::Ident("y".to_string()),
        Token::RightParen,
        Token::LeftBrace,
        Token::Ident("x".to_string()),
        Token::Star,
        Token::Ident("y".to_string()),
        Token::Semicolon,
        Token::RightBrace,
        Token::Semicolon,
        Token::Keyword(Keyword::Constrain),
        Token::Ident("mul".to_string()),
        Token::LeftParen,
        Token::Ident("five".to_string()),
        Token::Comma,
        Token::Ident("ten".to_string()),
        Token::RightParen,
        Token::Equal,
        Token::Int(50_i128.into()),
        Token::Semicolon,
        Token::Keyword(Keyword::Assert),
        Token::LeftParen,
        Token::Ident("ten".to_string()),
        Token::Plus,
        Token::Ident("five".to_string()),
        Token::Equal,
        Token::Int(15_i128.into()),
        Token::RightParen,
        Token::Semicolon,
        Token::EOF,
    ];
    let mut lexer = Lexer::new(input);

    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}
