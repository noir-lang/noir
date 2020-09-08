use crate::token::{Attribute, IntType, Keyword, Token};
use std::iter::Peekable;
use std::str::Chars;

// XXX(low) : We could probably use Bytes, but I cannot see the advantage yet. I don't think Unicode will be implemented
// XXX(low) : Currently the Lexer does not return Result. It would be more idiomatic to do this, instead of returning Token::Error
// XXX(low) : We may need to implement a TokenStream struct which wraps the lexer. This is then passed to the Parser
// XXX(low) : Possibly use &str instead of String when applicable
// XXX(med) : Add line numbers to the token information

pub struct Lexer<'a> {
    char_iter: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            char_iter: source.chars().peekable(),
        }
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
        self.char_iter.next()
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
        return false;
    }

    pub fn next_token(&mut self) -> Token {
        match self.next_char() {
            Some(x) if { x.is_whitespace() } => {
                self.eat_whitespace();
                return self.next_token();
            }
            Some('<') => self.glue(Token::Less),
            Some('>') => self.glue(Token::Greater),
            Some('=') => self.glue(Token::Assign),
            Some('#') => self.eat_attribute(),
            Some('/') => Token::Slash,
            Some('%') => Token::Percent,
            Some('&') => Token::Ampersand,
            Some('^') => Token::Caret,
            Some('.') => Token::Dot,
            Some(';') => Token::Semicolon,
            Some(':') => self.glue(Token::Colon),
            Some('*') => Token::Star,
            Some('(') => Token::LeftParen,
            Some(')') => Token::RightParen,
            Some(',') => Token::Comma,
            Some('+') => Token::Plus,
            Some('!') => self.glue(Token::Bang),
            Some('-') => self.glue(Token::Minus),
            Some('{') => Token::LeftBrace,
            Some('|') => Token::Pipe,
            Some('}') => Token::RightBrace,
            Some('[') => Token::LeftBracket,
            Some(']') => Token::RightBracket,
            Some('"') => self.eat_string_literal(),
            Some(ch) if ch.is_ascii_alphanumeric() => self.eat_alpha_numeric(ch),
            Some(ch) => Token::Error(format!("cannot parse character \"{}\" ", ch)),
            None => Token::EOF,
        }
    }

    /// Given that some tokens can contain two characters, such as <= , !=, >=
    /// Glue will take the first character of the token and check if it can be glued onto the next character
    /// forming a double token
    fn glue(&mut self, prev_token: Token) -> Token {
        match prev_token {
            Token::Less => {
                if self.peek_char_is('=') {
                    self.next_char();
                    return Token::LessEqual;
                }
                prev_token
            }
            Token::Greater => {
                if self.peek_char_is('=') {
                    self.next_char();
                    return Token::GreaterEqual;
                }
                prev_token
            }
            Token::Bang => {
                if self.peek_char_is('=') {
                    self.next_char();
                    return Token::NotEqual;
                }
                prev_token
            }
            Token::Assign => {
                if self.peek_char_is('=') {
                    self.next_char();
                    return Token::Equal;
                }
                prev_token
            }
            Token::Minus => {
                if self.peek_char_is('>') {
                    self.next_char();
                    return Token::Arrow;
                }
                prev_token
            }
            Token::Colon => {
                if self.peek_char_is(':') {
                    self.next_char();
                    return Token::DoubleColon;
                }
                prev_token
            }
            _ => Token::Error(format!("{} is not a double char token", prev_token)),
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
        match initial_char {
            Some(init_char) => word.push(init_char),
            _ => {}
        };

        // Keep checking that we are not at the EOF
        while self.peek_char().is_some() {
            let peek_char = self.peek_char().unwrap();
            // Then check for the predicate, if predicate matches append char and increment the cursor
            // If not, return word. The next character will be analysed on the next iteration of next_token,
            // Which will increment the cursor
            match predicate(*peek_char) {
                true => word.push(*peek_char),
                false => return word,
            }
            // If we arrive at this point, then the word has been added to the vector and we should increment the cursor
            self.next_char();
        }
        word
    }

    fn eat_alpha_numeric(&mut self, initial_char: char) -> Token {
        match initial_char {
            'A'..='Z' | 'a'..='z' => {
                return self.eat_word(initial_char);
            }
            '0'..='9' => {
                return self.eat_digit(initial_char);
            }
            _ => Token::Error(format!(
                "{} is not an alpha numeric character",
                initial_char
            )),
        }
    }

    fn eat_attribute(&mut self) -> Token {
        if !self.peek_char_is('[') {
            let err_msg = format!(
                "Lexer expected a '[' character after the '#' character, instead got {}",
                self.next_char().unwrap()
            );
            return Token::Error(err_msg.to_string());
        }
        self.next_char();

        let word = self.eat_while(None, |ch| {
            (ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_') && (ch != ']')
        });

        if !self.peek_char_is(']') {
            let err_msg = format!(
                "Lexer expected a trailing ']' character instead got {}",
                self.next_char().unwrap()
            );
            return Token::Error(err_msg.to_string());
        }
        self.next_char();

        Attribute::lookup_attribute(&word)
    }

    //XXX(low): Can increase performance if we use iterator semantic and utilise some of the methods on String. See below
    // https://doc.rust-lang.org/stable/std/primitive.str.html#method.rsplit
    fn eat_word(&mut self, initial_char: char) -> Token {
        let word = self.eat_while(Some(initial_char), |ch| {
            ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_'
        });

        // Check if word either an identifier or a keyword
        if let Some(keyword_token) = Keyword::lookup_keyword(&word.to_string()) {
            return keyword_token;
        }
        // Check if word an int type
        if let Some(int_type_token) = IntType::lookup_int_type(&word.to_string()) {
            return int_type_token;
        }
        return Token::Ident(word);
    }
    fn eat_digit(&mut self, initial_char: char) -> Token {
        let integer_str = self.eat_while(Some(initial_char), |ch| ch.is_numeric());
        let integer: i128 = integer_str.parse().unwrap();
        return Token::Int(integer);
    }
    fn eat_string_literal(&mut self) -> Token {
        let str_literal = self.eat_while(None, |ch| !(ch == '"'));
        return Token::Str(str_literal);
    }
    /// Skips white space. They are not significant in the source language
    fn eat_whitespace(&mut self) {
        self.eat_while(None, |ch| ch.is_whitespace());
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        Some(self.next_token())
    }
}

#[test]
fn test_single_double_char() {
    let input = "! != + ( ) { } [ ] | , ; : :: < <= > >= & - -> . % / * = ==";

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
        Token::Percent,
        Token::Slash,
        Token::Star,
        Token::Assign,
        Token::Equal,
        Token::EOF,
    ];

    let mut lexer = Lexer::new(input);

    for token in expected.into_iter() {
        let got = lexer.next_token();
        assert_eq!(got, token);
    }
}

#[test]
fn test_custom_gate_syntax() {
    let input = "#[sha256]#[directive]";

    let expected = vec![
        Token::Attribute(Attribute::Str("sha256".to_string())),
        Token::Attribute(Attribute::Directive),
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let got = lexer.next_token();
        assert_eq!(got, token);
    }
}
#[test]
fn test_int_type() {
    let input = "u16 i16 i107 u104.5";

    let expected = vec![
        Token::IntType(IntType::Unsigned(16)),
        Token::IntType(IntType::Signed(16)),
        Token::IntType(IntType::Signed(107)),
        Token::IntType(IntType::Unsigned(104)),
        Token::Dot,
        Token::Int(5),
    ];

    let mut lexer = Lexer::new(input);
    for token in expected.into_iter() {
        let got = lexer.next_token();
        assert_eq!(got, token);
    }
}
#[test]
fn test_eat_string_literal() {
    let input = "let word = \"hello\"";

    let expected = vec![
        Token::Keyword(Keyword::Let),
        Token::Ident("word".to_string()),
        Token::Assign,
        Token::Str("hello".to_string()),
    ];
    let mut lexer = Lexer::new(input);

    for token in expected.into_iter() {
        let got = lexer.next_token();
        assert_eq!(got, token);
    }
}

#[test]
fn test_basic_language_syntax() {
    let input = "
    
    const five = 5;
    pub ten : Witness = 10;
    let mul = fn(x, y) {
         x * y;
    };
    priv result = mul(five, ten);
    
    ";

    let expected = vec![
        Token::Keyword(Keyword::Const),
        Token::Ident("five".to_string()),
        Token::Assign,
        Token::Int(5),
        Token::Semicolon,
        Token::Keyword(Keyword::Pub),
        Token::Ident("ten".to_string()),
        Token::Colon,
        Token::Keyword(Keyword::Witness),
        Token::Assign,
        Token::Int(10),
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
        Token::Keyword(Keyword::Private),
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
        let got = lexer.next_token();
        assert_eq!(got, token);
    }
}
