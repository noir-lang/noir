use std::str::{CharIndices, FromStr};

use acir_field::{AcirField, FieldElement};

use noirc_span::{Position, Span};
use num_bigint::BigInt;
use num_traits::One;
use thiserror::Error;

use crate::parser::token::Keyword;

use super::token::{SpannedToken, Token};

pub(super) struct Lexer<'a> {
    chars: CharIndices<'a>,
    position: Position,
    done: bool,
    max_integer: BigInt,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.char_indices(),
            position: 0,
            done: false,
            max_integer: BigInt::from_biguint(num_bigint::Sign::Plus, FieldElement::modulus()) // cSpell:disable-line
                - BigInt::one(),
        }
    }

    pub(super) fn next_token(&mut self) -> SpannedTokenResult {
        let ch = match self.next_char() {
            Some(ch) => ch,
            None => {
                self.done = true;
                return Ok(Token::Eof.into_single_span(self.position));
            }
        };

        match ch {
            ch if ch.is_ascii_whitespace() => {
                while let Some(char) = self.peek_char() {
                    if char.is_ascii_whitespace() {
                        self.next_char();
                    } else {
                        break;
                    }
                }
                self.next_token()
            }
            '/' if self.peek_char() == Some('/') => {
                while let Some(char) = self.next_char() {
                    if char == '\n' {
                        break;
                    }
                }
                self.next_token()
            }
            '(' => self.single_char_token(Token::LeftParen),
            ')' => self.single_char_token(Token::RightParen),
            '[' => self.single_char_token(Token::LeftBracket),
            ']' => self.single_char_token(Token::RightBracket),
            ',' => self.single_char_token(Token::Comma),
            ':' => self.single_char_token(Token::Colon),
            ';' => self.single_char_token(Token::Semicolon),
            '+' => self.single_char_token(Token::Plus),
            '-' if self.peek_char().is_none_or(|char| !char.is_ascii_digit()) => {
                self.single_char_token(Token::Minus)
            }
            '*' => self.single_char_token(Token::Star),
            '=' => self.single_char_token(Token::Equal),
            'b' | 'w' if self.peek_char().is_some_and(|char| char.is_ascii_digit()) => {
                let start = self.position;

                // Witness token format is 'w' followed by digits.
                // Block token format is 'b' followed by digits.
                let digits = self.eat_while(None, |ch| ch.is_ascii_digit());
                let end = self.position;

                // Parse digits into u32
                match digits.parse::<u32>() {
                    Ok(value) => {
                        let token =
                            if ch == 'w' { Token::Witness(value) } else { Token::Block(value) };
                        Ok(token.into_span(start, end))
                    }
                    Err(_) => Err(LexerError::InvalidIntegerLiteral {
                        span: Span::inclusive(start, end),
                        found: digits,
                    }),
                }
            }
            '-' | '0'..='9' => self.eat_integer(ch),
            ch if ch.is_ascii_alphabetic() => self.eat_word(ch),
            ch => Err(LexerError::UnexpectedCharacter {
                char: ch,
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

        // Else it is just an identifier
        let ident_token = Token::Ident(word);
        Ok(ident_token.into_span(start, end))
    }

    fn eat_integer(&mut self, first_char: char) -> SpannedTokenResult {
        let start = self.position;
        let mut number_str = String::new();

        let is_negative = if first_char == '-' {
            // Peek ahead that '-' must be followed by a digit
            match self.peek_char() {
                Some(ch) if ch.is_ascii_digit() => {
                    // Consume the digit we just peeked
                    self.next_char();
                    number_str.push('-');
                    number_str.push(ch);
                }
                _ => {
                    return Err(LexerError::UnexpectedCharacter {
                        char: '-',
                        span: Span::single_char(start),
                    });
                }
            }
            true
        } else {
            number_str.push(first_char);
            false
        };

        number_str += &self.eat_while(None, |ch| ch.is_ascii_digit());

        let end = self.position;

        let bigint_result = BigInt::from_str(&number_str);
        let integer = match bigint_result {
            Ok(bigint) => {
                if bigint > self.max_integer {
                    return Err(LexerError::IntegerLiteralTooLarge {
                        span: Span::inclusive(start, end),
                        limit: self.max_integer.to_string(),
                    });
                }
                let big_uint = bigint.magnitude();
                let field = FieldElement::from_be_bytes_reduce(&big_uint.to_bytes_be());
                if is_negative { -field } else { field }
            }
            Err(_) => {
                return Err(LexerError::InvalidIntegerLiteral {
                    span: Span::inclusive(start, end),
                    found: number_str,
                });
            }
        };

        Ok(Token::Int(integer).into_span(start, end))
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

    fn next_char(&mut self) -> Option<char> {
        let (position, ch) = self.chars.next()?;
        self.position = position as u32;
        Some(ch)
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, ch)| ch)
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
}

impl LexerError {
    pub(super) fn span(&self) -> Span {
        use LexerError::*;
        match self {
            UnexpectedCharacter { span, .. } => *span,
            InvalidIntegerLiteral { span, .. } => *span,
            IntegerLiteralTooLarge { span, .. } => *span,
        }
    }
}
