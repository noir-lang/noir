use std::str::CharIndices;

use noirc_errors::{Position, Span};

use super::token::{Keyword, SpannedToken, Token};

pub(crate) struct Lexer<'a> {
    chars: CharIndices<'a>,
    position: Position,
    done: bool,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Lexer { chars: source.char_indices(), position: 0, done: false }
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
            Some(':') => self.single_char_token(Token::Colon),
            Some('(') => self.single_char_token(Token::LeftParen),
            Some(')') => self.single_char_token(Token::RightParen),
            Some('{') => self.single_char_token(Token::LeftBrace),
            Some('}') => self.single_char_token(Token::RightBrace),
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
            // '0'..='9' => self.eat_digit(initial_char),
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
        // let parsed_token = IntType::lookup_int_type(&word)?;

        // // Check if it is an int type
        // if let Some(int_type_token) = parsed_token {
        //     return Ok(int_type_token.into_span(start, end));
        // }

        // Else it is just an identifier
        let ident_token = Token::Ident(word);
        Ok(ident_token.into_span(start, end))
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

    fn peek_char(&mut self) -> Option<char> {
        self.chars.clone().next().map(|(_, ch)| ch)
    }

    fn is_code_whitespace(c: char) -> bool {
        c.is_ascii_whitespace()
    }
}

type SpannedTokenResult = Result<SpannedToken, LexerError>;

#[derive(Debug)]
pub(crate) enum LexerError {
    UnexpectedCharacter { char: char, span: Span },
}
