use super::{errors::LexerErrorKind, token::{Attribute, IntType, Keyword, Token, SpannedToken}};
use std::iter::Peekable;
use std::str::Chars;
use noir_field::FieldElement;
use noirc_errors::Position;
use fm::File;
use super::errors::LexerError;
// XXX(low) : We could probably use Bytes, but I cannot see the advantage yet. I don't think Unicode will be implemented
// XXX(low) : Currently the Lexer does not return Result. It would be more idiomatic to do this, instead of returning Token::Error
// XXX(low) : We may need to implement a TokenStream struct which wraps the lexer. This is then passed to the Parser
// XXX(low) : Possibly use &str instead of String when applicable

pub type SpannedTokenResult = Result<SpannedToken, LexerError>;

pub struct Lexer<'a> {
    char_iter: Peekable<Chars<'a>>,
    position  : Position,
    pub file_id : usize, 
}

impl<'a> Lexer<'a> {
    pub fn new(file_id : usize, source: &'a str) -> Self {
        Lexer {
            char_iter: source.chars().peekable(),
            position : Position::default_from(file_id),
            file_id,
        }
    }
    pub fn from_file(file_id : usize, source: File<'a>) -> Self {
        let source_file = source.get_source();
        Lexer::new(file_id, source_file)
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
            _=> return None
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
        return false;
    }

    pub fn next_token(&mut self) -> SpannedTokenResult {
        match self.next_char() {
            Some(x) if { x.is_whitespace() } => {
                self.eat_whitespace();
                return self.next_token();
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
            Some('#') => Ok(self.eat_attribute()),
            Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' => self.eat_alpha_numeric(ch),
            Some(ch) => {
                let span = self.position.mark().into_span();
                Err(LexerErrorKind::CharacterNotInLanguage{span, found : ch }.into_err(self.file_id))
            },
            None => Ok(Token::EOF.into_single_span(self.position.mark())),
        }
    }

    fn single_char_token(&self, token : Token) -> SpannedTokenResult {
        Ok(token.into_single_span(self.position.mark()))
    }

    fn single_double_peek_token(&mut self, character : char, single : Token, double : Token) -> SpannedTokenResult {
        let start = self.position.mark();

        match self.peek_char_is(character) {
            false => return Ok(single.into_single_span(start)), 
            true => {
                self.next_char();
                return Ok(double.into_span(start, start.forward()))
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
                    return Ok(self.parse_comment())
                }
                Ok(spanned_prev_token)
            }
            Token::Underscore => {
        
                let next_char = self.peek_char();
                let peeked_char = match next_char {
                    Some(peek_char) => peek_char,
                    None => return Ok(spanned_prev_token)
                };
        
                if peeked_char.is_ascii_alphabetic() {
                    // Okay to unwrap here because we already peeked to 
                    // see that we have a character
                    let curr_char = self.next_char().unwrap();
                    return Ok(self.eat_word(curr_char))
                }
                
                Ok(spanned_prev_token)
            }
            _ => {
                let span = self.position.mark().into_span();
                Err(LexerErrorKind::NotADoubleChar{span, found : prev_token}.into_err(self.file_id))
            },
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
            'A'..='Z' | 'a'..='z' | '_' => {
                Ok(self.eat_word(initial_char))
            }
            '0'..='9' => {
                Ok(self.eat_digit(initial_char))
            }
            _ =>  {
                let span = self.position.mark().into_span();
                Err(LexerErrorKind::UnexpectedCharacter{span, found : initial_char}.into_err(self.file_id))
            },
        }
    }

    fn eat_attribute(&mut self) -> SpannedToken {
        if !self.peek_char_is('[') {
            let err_msg = format!(
                "Lexer expected a '[' character after the '#' character, instead got {}",
                self.next_char().unwrap()
            );
            let err = Token::Error(err_msg.to_string());
            return err.into_single_span(self.position.mark())
        }
        self.next_char();
        
        let (word, start_span, end_span) = self.eat_while(None, |ch| {
            (ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_' || ch == '(' || ch == ')') && (ch != ']')
        });
        
        if !self.peek_char_is(']') {
            let err_msg = format!(
                "Lexer expected a trailing ']' character instead got {}",
                self.next_char().unwrap()
            );
            return Token::Error(err_msg.to_string()).into_single_span(self.position.mark());
        }
        self.next_char();

        let attribute = Attribute::lookup_attribute(&word);

        // Move start position backwards to cover the left bracket
        // Move end position forwards to cover the right bracket
        attribute.into_span(start_span.backward(),end_span.forward())
    }

    //XXX(low): Can increase performance if we use iterator semantic and utilise some of the methods on String. See below
    // https://doc.rust-lang.org/stable/std/primitive.str.html#method.rsplit
    fn eat_word(&mut self, initial_char: char) -> SpannedToken {
        let (word, start_span, end_span) = self.eat_while(Some(initial_char), |ch| {
            ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_'
        });

        // Check if word either an identifier or a keyword
        if let Some(keyword_token) = Keyword::lookup_keyword(&word.to_string()) {
            return keyword_token.into_span(start_span, end_span);
        }
        // Check if word an int type
        if let Some(int_type_token) = IntType::lookup_int_type(&word.to_string()) {
            return int_type_token.into_span(start_span, end_span);
        }
        let ident_token = Token::Ident(word);
        return ident_token.into_span(start_span, end_span);
    }
    fn eat_digit(&mut self, initial_char: char) -> SpannedToken {
        let (integer_str, start_span, end_span) = self.eat_while(Some(initial_char), |ch| ch.is_numeric());
        let integer = match FieldElement::from_str(&integer_str) {
            None => panic!("Expected an integer in base10. Hex is not supported currently, coming soon."),
            Some(integer) => integer 
        };
        let integer_token = Token::Int(integer.into());
        integer_token.into_span(start_span, end_span,)
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

    let mut lexer = Lexer::new(0,input);

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

    let mut lexer = Lexer::new(0,input);
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

    let mut lexer = Lexer::new(0,input);
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

    let mut lexer = Lexer::new(0,input);
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
    let mut lexer = Lexer::new(0,input);

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

    let expected = vec![
        let_token,
        ident_token,
        assign_token,
        int_token,
    ];
    let mut lexer = Lexer::new(0,input);

    for spanned_token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got.into_span(),spanned_token.into_span());
        assert_eq!(got,spanned_token);
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
        Token::Int(5.into()),
        Token::Semicolon,
        Token::Keyword(Keyword::Pub),
        Token::Ident("ten".to_string()),
        Token::Colon,
        Token::Keyword(Keyword::Witness),
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
    let mut lexer = Lexer::new(0,input);

    for token in expected.into_iter() {
        let got = lexer.next_token().unwrap();
        assert_eq!(got, token);
    }
}
