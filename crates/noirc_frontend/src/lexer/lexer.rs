use super::{
    errors::LexerErrorKind,
    token::{Attribute, IntType, Keyword, SpannedToken, Token},
};
use fm::File;
use noir_field::FieldElement;
use noirc_errors::{Position, Span};
use std::iter::Peekable;
use std::str::Chars;
// XXX(low) : We could probably use Bytes, but I cannot see the advantage yet. I don't think Unicode will be implemented
// XXX(low) : We may need to implement a TokenStream struct which wraps the lexer. This is then passed to the Parser
// XXX(low) : Possibly use &str instead of String when applicable

pub type SpannedTokenResult = Result<SpannedToken, LexerErrorKind>;

pub struct Lexer<'a> {
    char_iter: Peekable<Chars<'a>>,
    position: Position,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            char_iter: source.chars().peekable(),
            position: Position::default(),
        }
    }
    pub fn from_file(source: File<'a>) -> Self {
        let source_file = source.get_source();
        Lexer::new(source_file)
    }

    // This method uses size_hint and therefore should not be trusted 100%
    pub fn approx_len(&self) -> usize {
        let mut size_hint: usize;
        let (lower, _upper) = self.char_iter.size_hint();
        size_hint = lower; // This is better than nothing, if we do not have an upper bound

        if let Some(upper) = _upper {
            size_hint = upper;
        }

        size_hint
    }
    /// Iterates the cursor and returns the char at the new cursor position
    fn next_char(&mut self) -> Option<char> {
        let next_char = self.char_iter.next();
        match &next_char {
            Some('\n') => self.position.new_line(),
            Some(_) => self.position.right_shift(),
            _ => return None,
        };

        next_char
    }
    /// Peeks at the next char. Does not iterate the cursor
    fn peek_char(&mut self) -> Option<&char> {
        self.char_iter.peek()
    }
    /// Peeks at the next char and returns true if it is equal to the char argument
    fn peek_char_is(&mut self, ch: char) -> bool {
        if let Some(peeked_ch) = self.peek_char() {
            return *peeked_ch == ch;
        };
        false
    }

    pub fn next_token(&mut self) -> SpannedTokenResult {
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
            Some('%') => self.single_char_token(Token::Percent),
            Some('&') => self.single_char_token(Token::Ampersand),
            Some('^') => self.single_char_token(Token::Caret),
            Some(';') => self.single_char_token(Token::Semicolon),
            Some('*') => self.single_char_token(Token::Star),
            Some('(') => self.single_char_token(Token::LeftParen),
            Some(')') => self.single_char_token(Token::RightParen),
            Some(',') => self.single_char_token(Token::Comma),
            Some('+') => self.single_char_token(Token::Plus),
            Some('{') => self.single_char_token(Token::LeftBrace),
            Some('|') => self.single_char_token(Token::Pipe),
            Some('}') => self.single_char_token(Token::RightBrace),
            Some('[') => self.single_char_token(Token::LeftBracket),
            Some(']') => self.single_char_token(Token::RightBracket),
            Some('"') => Ok(self.eat_string_literal()),
            Some('#') => self.eat_attribute(),
            Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' => self.eat_alpha_numeric(ch),
            Some(ch) => {
                let span = self.position.mark().into_span();
                Err(LexerErrorKind::CharacterNotInLanguage { span, found: ch })
            }
            None => Ok(Token::EOF.into_single_span(self.position.mark())),
        }
    }

    fn single_char_token(&self, token: Token) -> SpannedTokenResult {
        Ok(token.into_single_span(self.position.mark()))
    }

    fn single_double_peek_token(
        &mut self,
        character: char,
        single: Token,
        double: Token,
    ) -> SpannedTokenResult {
        let start = self.position.mark();

        match self.peek_char_is(character) {
            false => Ok(single.into_single_span(start)),
            true => {
                self.next_char();
                Ok(double.into_span(start, start.forward()))
            }
        }
    }

    /// Given that some tokens can contain two characters, such as <= , !=, >=
    /// Glue will take the first character of the token and check if it can be glued onto the next character
    /// forming a double token
    fn glue(&mut self, prev_token: Token) -> SpannedTokenResult {
        let spanned_prev_token = prev_token.clone().into_single_span(self.position.mark());
        match prev_token {
            Token::Dot => self.single_double_peek_token('.', prev_token, Token::DoubleDot),
            Token::Less => self.single_double_peek_token('=', prev_token, Token::LessEqual),
            Token::Greater => self.single_double_peek_token('=', prev_token, Token::GreaterEqual),
            Token::Bang => self.single_double_peek_token('=', prev_token, Token::NotEqual),
            Token::Assign => self.single_double_peek_token('=', prev_token, Token::Equal),
            Token::Minus => self.single_double_peek_token('>', prev_token, Token::Arrow),
            Token::Colon => self.single_double_peek_token(':', prev_token, Token::DoubleColon),
            Token::Slash => {
                if self.peek_char_is('/') {
                    self.next_char();
                    return Ok(self.parse_comment());
                }
                Ok(spanned_prev_token)
            }
            Token::Underscore => {
                let next_char = self.peek_char();
                let peeked_char = match next_char {
                    Some(peek_char) => peek_char,
                    None => return Ok(spanned_prev_token),
                };

                if peeked_char.is_ascii_alphabetic() {
                    // Okay to unwrap here because we already peeked to
                    // see that we have a character
                    let curr_char = self.next_char().unwrap();
                    return self.eat_word(curr_char);
                }

                Ok(spanned_prev_token)
            }
            _ => {
                let span = self.position.mark().into_span();
                Err(LexerErrorKind::NotADoubleChar {
                    span,
                    found: prev_token,
                })
            }
        }
    }

    /// Keeps consuming tokens as long as the predicate is satisfied
    fn eat_while<F: Fn(char) -> bool>(
        &mut self,
        initial_char: Option<char>,
        predicate: F,
    ) -> (String, Position, Position) {
        let start_span = self.position.mark();

        // This function is only called when we want to continue consuming a character of the same type.
        // For example, we see a digit and we want to consume the whole integer
        // Therefore, the current character which triggered this function will need to be appended
        let mut word = String::new();
        if let Some(init_char) = initial_char {
            word.push(init_char)
        }

        // Keep checking that we are not at the EOF
        while let Some(peek_char) = self.peek_char() {
            // Then check for the predicate, if predicate matches append char and increment the cursor
            // If not, return word. The next character will be analysed on the next iteration of next_token,
            // Which will increment the cursor
            if !predicate(*peek_char) {
                return (word, start_span, self.position.mark());
            }
            word.push(*peek_char);

            // If we arrive at this point, then the char has been added to the word and we should increment the cursor
            self.next_char();
        }
        let end_span = self.position.mark();

        (word, start_span, end_span)
    }

    fn eat_alpha_numeric(&mut self, initial_char: char) -> SpannedTokenResult {
        match initial_char {
            'A'..='Z' | 'a'..='z' | '_' => Ok(self.eat_word(initial_char)?),
            '0'..='9' => self.eat_digit(initial_char),
            _ => {
                let span = self.position.mark().into_span();
                Err(LexerErrorKind::UnexpectedCharacter {
                    span,
                    found: initial_char,
                    expected: "an alpha numeric character".to_owned(),
                })
            }
        }
    }

    fn eat_attribute(&mut self) -> SpannedTokenResult {
        if !self.peek_char_is('[') {
            let start = self.position.mark();
            let end = start;

            return Err(LexerErrorKind::UnexpectedCharacter {
                span: Span { start, end },
                found: self.next_char().unwrap(),
                expected: "[".to_owned(),
            });
        }
        self.next_char();

        let (word, start_span, end_span) = self.eat_while(None, |ch| {
            (ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_' || ch == '(' || ch == ')')
                && (ch != ']')
        });

        if !self.peek_char_is(']') {
            let start = self.position.mark();
            let end = start;
            return Err(LexerErrorKind::UnexpectedCharacter {
                span: Span { start, end },
                expected: "]".to_owned(),
                found: self.next_char().unwrap(),
            });
        }
        self.next_char();

        let attribute = Attribute::lookup_attribute(
            &word,
            Span {
                start: start_span,
                end: end_span,
            },
        )?;

        // Move start position backwards to cover the left bracket
        // Move end position forwards to cover the right bracket
        Ok(attribute.into_span(start_span.backward(), end_span.forward()))
    }

    //XXX(low): Can increase performance if we use iterator semantic and utilise some of the methods on String. See below
    // https://doc.rust-lang.org/stable/std/primitive.str.html#method.rsplit
    fn eat_word(&mut self, initial_char: char) -> SpannedTokenResult {
        let (word, start_span, end_span) = self.eat_while(Some(initial_char), |ch| {
            ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_'
        });

        // Check if word either an identifier or a keyword
        if let Some(keyword_token) = Keyword::lookup_keyword(&word) {
            return Ok(keyword_token.into_span(start_span, end_span));
        }

        // Check if word an int type
        // if no error occurred, then it is either a valid integer type or it is not an int type
        let parsed_token = IntType::lookup_int_type(
            &word,
            Span {
                start: start_span,
                end: end_span,
            },
        )?;

        // Check if it is an int type
        if let Some(int_type_token) = parsed_token {
            return Ok(int_type_token.into_span(start_span, end_span));
        }

        // Else it is just an identifier
        let ident_token = Token::Ident(word);
        Ok(ident_token.into_span(start_span, end_span))
    }
    fn eat_digit(&mut self, initial_char: char) -> SpannedTokenResult {
        let (integer_str, start_span, end_span) = self.eat_while(Some(initial_char), |ch| {
            ch.is_digit(10) | ch.is_digit(16) | (ch == 'x')
        });

        let integer = match FieldElement::try_from_str(&integer_str) {
            None => {
                return Err(LexerErrorKind::InvalidIntegerLiteral {
                    span: Span {
                        start: start_span,
                        end: end_span,
                    },
                    found: integer_str,
                })
            }
            Some(integer) => integer,
        };

        let integer_token = Token::Int(integer);
        Ok(integer_token.into_span(start_span, end_span))
    }
    fn eat_string_literal(&mut self) -> SpannedToken {
        let (str_literal, start_span, end_span) = self.eat_while(None, |ch| ch != '"');
        let str_literal_token = Token::Str(str_literal);
        str_literal_token.into_span(start_span, end_span)
    }
    fn parse_comment(&mut self) -> SpannedToken {
        let (comment_literal, start_span, end_span) = self.eat_while(None, |ch| ch != '\n');
        let comment_literal_token = Token::Comment(comment_literal);
        comment_literal_token.into_span(start_span, end_span)
    }
    /// Skips white space. They are not significant in the source language
    fn eat_whitespace(&mut self) {
        self.eat_while(None, |ch| ch.is_whitespace());
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = SpannedTokenResult;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_token())
    }
}

#[test]
fn test_single_double_char() {
    let input = "! != + ( ) { } [ ] | , ; : :: < <= > >= & - -> . .. % / * = ==";

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
        Token::EOF,
    ];

    let mut lexer = Lexer::new(input);

    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
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
        Token::Int(5.into()),
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}
#[test]
fn test_comment() {
    let input = "// hello
        let x = 5
    ";

    let expected = vec![
        Token::Comment(" hello".to_string()),
        Token::Keyword(Keyword::Let),
        Token::Ident("x".to_string()),
        Token::Assign,
        Token::Int(FieldElement::from(5)),
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

    let expected = vec![Token::Int(5.into())];
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
    let start_position = Position::default().forward();
    let let_position = start_position.forward_by(2);
    let let_token = Token::Keyword(Keyword::Let).into_span(start_position, let_position);

    // Skip whitespace
    let whitespace_position = let_position.forward();

    // Identifier position
    let ident_position = whitespace_position.forward();
    let ident_token = Token::Ident("x".to_string()).into_single_span(ident_position);

    // Skip whitespace
    let whitespace_position = ident_position.forward();

    // Assign position
    let assign_position = whitespace_position.forward();
    let assign_token = Token::Assign.into_single_span(assign_position);

    // Skip whitespace
    let whitespace_position = assign_position.forward();

    // Int position
    let int_position = whitespace_position.forward();
    let int_token = Token::Int(5.into()).into_single_span(int_position);

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

    const five = 5;
    pub ten : Field = 10;
    let mul = fn(x, y) {
         x * y;
    };
    priv result = mul(five, ten);

    ";

    let expected = vec![
        Token::Keyword(Keyword::Const),
        Token::Ident("five".to_string()),
        Token::Assign,
        Token::Int(5.into()),
        Token::Semicolon,
        Token::Keyword(Keyword::Pub),
        Token::Ident("ten".to_string()),
        Token::Colon,
        Token::Keyword(Keyword::Field),
        Token::Assign,
        Token::Int(10.into()),
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
        Token::Keyword(Keyword::Priv),
        Token::Ident("result".to_string()),
        Token::Assign,
        Token::Ident("mul".to_string()),
        Token::LeftParen,
        Token::Ident("five".to_string()),
        Token::Comma,
        Token::Ident("ten".to_string()),
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
