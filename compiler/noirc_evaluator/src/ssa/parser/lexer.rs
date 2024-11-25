use std::str::{CharIndices, FromStr};

use acvm::{AcirField, FieldElement};
use noirc_errors::{Position, Span};
use noirc_frontend::token::IntType;
use num_bigint::BigInt;
use num_traits::{Num, One};
use thiserror::Error;

use super::token::{Keyword, SpannedToken, Token};

pub(crate) struct Lexer<'a> {
    chars: CharIndices<'a>,
    position: Position,
    done: bool,
    max_integer: BigInt,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.char_indices(),
            position: 0,
            done: false,
            max_integer: BigInt::from_biguint(num_bigint::Sign::Plus, FieldElement::modulus())
                - BigInt::one(),
        }
    }

    pub(crate) fn next_token(&mut self) -> SpannedTokenResult {
        match self.next_char() {
            Some(char) if char.is_ascii_whitespace() => {
                while let Some(char) = self.peek_char() {
                    if char.is_ascii_whitespace() {
                        self.next_char();
                    } else {
                        break;
                    }
                }
                self.next_token()
            }
            Some('/') if self.peek_char() == Some('/') => {
                while let Some(char) = self.next_char() {
                    if char == '\n' {
                        break;
                    }
                }
                self.next_token()
            }
            Some('=') if self.peek_char() == Some('=') => self.double_char_token(Token::Equal),
            Some('=') => self.single_char_token(Token::Assign),
            Some(',') => self.single_char_token(Token::Comma),
            Some(':') => self.single_char_token(Token::Colon),
            Some(';') => self.single_char_token(Token::Semicolon),
            Some('(') => self.single_char_token(Token::LeftParen),
            Some(')') => self.single_char_token(Token::RightParen),
            Some('{') => self.single_char_token(Token::LeftBrace),
            Some('}') => self.single_char_token(Token::RightBrace),
            Some('[') => self.single_char_token(Token::LeftBracket),
            Some(']') => self.single_char_token(Token::RightBracket),
            Some('&') => self.single_char_token(Token::Ampersand),
            Some('-') if self.peek_char() == Some('>') => self.double_char_token(Token::Arrow),
            Some('-') => self.single_char_token(Token::Dash),
            Some('"') => self.eat_string_literal(),
            Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' => self.eat_alpha_numeric(ch),
            Some(char) => Err(LexerError::UnexpectedCharacter {
                char,
                span: Span::single_char(self.position),
            }),
            None => {
                self.done = true;
                Ok(Token::Eof.into_single_span(self.position))
            }
        }
    }

    fn eat_alpha_numeric(&mut self, initial_char: char) -> SpannedTokenResult {
        match initial_char {
            'A'..='Z' | 'a'..='z' | '_' => Ok(self.eat_word(initial_char)?),
            '0'..='9' => self.eat_digit(initial_char),
            _ => Err(LexerError::UnexpectedCharacter {
                char: initial_char,
                span: Span::single_char(self.position),
            }),
        }
    }

    fn eat_word(&mut self, initial_char: char) -> SpannedTokenResult {
        let (start, word, end) = self.lex_word(initial_char);
        self.lookup_word_token(word, start, end)
    }

    fn lex_word(&mut self, initial_char: char) -> (Position, String, Position) {
        let start = self.position;
        let word = self.eat_while(Some(initial_char), |ch| {
            ch.is_ascii_alphabetic() || ch.is_numeric() || ch == '_'
        });
        (start, word, self.position)
    }

    fn lookup_word_token(
        &self,
        word: String,
        start: Position,
        end: Position,
    ) -> SpannedTokenResult {
        // Check if word either an identifier or a keyword
        if let Some(keyword_token) = Keyword::lookup_keyword(&word) {
            return Ok(keyword_token.into_span(start, end));
        }

        // Check if word an int type
        // if no error occurred, then it is either a valid integer type or it is not an int type
        let parsed_token = IntType::lookup_int_type(&word);

        // Check if it is an int type
        if let Some(int_type) = parsed_token {
            return Ok(Token::IntType(int_type).into_span(start, end));
        }

        // Else it is just an identifier
        let ident_token = Token::Ident(word);
        Ok(ident_token.into_span(start, end))
    }

    fn eat_digit(&mut self, initial_char: char) -> SpannedTokenResult {
        let start = self.position;

        let integer_str = self.eat_while(Some(initial_char), |ch| {
            ch.is_ascii_digit() | ch.is_ascii_hexdigit() | (ch == 'x') | (ch == '_')
        });

        let end = self.position;

        // We want to enforce some simple rules about usage of underscores:
        // 1. Underscores cannot appear at the end of a integer literal. e.g. 0x123_.
        // 2. There cannot be more than one underscore consecutively, e.g. 0x5__5, 5__5.
        //
        // We're not concerned with an underscore at the beginning of a decimal literal
        // such as `_5` as this would be lexed into an ident rather than an integer literal.
        let invalid_underscore_location = integer_str.ends_with('_');
        let consecutive_underscores = integer_str.contains("__");
        if invalid_underscore_location || consecutive_underscores {
            return Err(LexerError::InvalidIntegerLiteral {
                span: Span::inclusive(start, end),
                found: integer_str,
            });
        }

        // Underscores needs to be stripped out before the literal can be converted to a `FieldElement.
        let integer_str = integer_str.replace('_', "");

        let bigint_result = match integer_str.strip_prefix("0x") {
            Some(integer_str) => BigInt::from_str_radix(integer_str, 16),
            None => BigInt::from_str(&integer_str),
        };

        let integer = match bigint_result {
            Ok(bigint) => {
                if bigint > self.max_integer {
                    return Err(LexerError::IntegerLiteralTooLarge {
                        span: Span::inclusive(start, end),
                        limit: self.max_integer.to_string(),
                    });
                }
                let big_uint = bigint.magnitude();
                FieldElement::from_be_bytes_reduce(&big_uint.to_bytes_be())
            }
            Err(_) => {
                return Err(LexerError::InvalidIntegerLiteral {
                    span: Span::inclusive(start, end),
                    found: integer_str,
                })
            }
        };

        let integer_token = Token::Int(integer);
        Ok(integer_token.into_span(start, end))
    }

    fn eat_string_literal(&mut self) -> SpannedTokenResult {
        let start = self.position;
        let mut string = String::new();

        while let Some(next) = self.next_char() {
            let char = match next {
                '"' => break,
                '\\' => match self.next_char() {
                    Some('r') => '\r',
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('0') => '\0',
                    Some('"') => '"',
                    Some('\\') => '\\',
                    Some(escaped) => {
                        let span = Span::inclusive(start, self.position);
                        return Err(LexerError::InvalidEscape { escaped, span });
                    }
                    None => {
                        let span = Span::inclusive(start, self.position);
                        return Err(LexerError::UnterminatedStringLiteral { span });
                    }
                },
                other => other,
            };

            string.push(char);
        }

        let str_literal_token = Token::Str(string);

        let end = self.position;
        Ok(str_literal_token.into_span(start, end))
    }

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

    fn single_char_token(&self, token: Token) -> SpannedTokenResult {
        Ok(token.into_single_span(self.position))
    }

    fn double_char_token(&mut self, token: Token) -> SpannedTokenResult {
        let start_position = self.position;
        self.next_char();
        Ok(token.into_span(start_position, self.position))
    }

    fn next_char(&mut self) -> Option<char> {
        let (position, ch) = self.chars.next()?;
        self.position = position as u32;
        Some(ch)
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, ch)| ch)
    }

    fn is_code_whitespace(c: char) -> bool {
        c.is_ascii_whitespace()
    }

    pub(crate) fn newline_follows(&self) -> bool {
        let chars = self.chars.clone();
        chars.take_while(|(_, char)| char.is_ascii_whitespace()).any(|(_, char)| char == '\n')
    }
}

type SpannedTokenResult = Result<SpannedToken, LexerError>;

#[derive(Debug, Error)]
pub(crate) enum LexerError {
    #[error("Unexpected character: {char:?}")]
    UnexpectedCharacter { char: char, span: Span },
    #[error("Invalid integer literal")]
    InvalidIntegerLiteral { span: Span, found: String },
    #[error("Integer literal too large")]
    IntegerLiteralTooLarge { span: Span, limit: String },
    #[error("Unterminated string literal")]
    UnterminatedStringLiteral { span: Span },
    #[error(
        "'\\{escaped}' is not a valid escape sequence. Use '\\' for a literal backslash character."
    )]
    InvalidEscape { escaped: char, span: Span },
}

impl LexerError {
    pub(crate) fn span(&self) -> Span {
        match self {
            LexerError::UnexpectedCharacter { span, .. }
            | LexerError::InvalidIntegerLiteral { span, .. }
            | LexerError::IntegerLiteralTooLarge { span, .. }
            | LexerError::UnterminatedStringLiteral { span }
            | LexerError::InvalidEscape { span, .. } => *span,
        }
    }
}
