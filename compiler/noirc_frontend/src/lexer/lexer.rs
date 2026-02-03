use crate::token::DocStyle;

use super::{
    errors::LexerErrorKind,
    token::{
        FmtStrFragment, IntegerTypeSuffix, Keyword, LocatedToken, SpannedToken, Token, Tokens,
    },
};
use acvm::{AcirField, FieldElement};
use fm::FileId;
use noirc_errors::{Location, Position, Span};
use num_bigint::BigInt;
use num_traits::{Num, One};
use std::str::{CharIndices, FromStr};

/// The job of the lexer is to transform an iterator of characters (`char_iter`)
/// into an iterator of `SpannedToken`. Each `Token` corresponds roughly to 1 word or operator.
/// Tokens are tagged with their location in the source file (a `Span`) for use in error reporting.
pub struct Lexer<'a> {
    file_id: FileId,
    chars: CharIndices<'a>,
    position: Position,
    done: bool,
    skip_comments: bool,
    skip_whitespaces: bool,
    max_integer: BigInt,
}

pub type SpannedTokenResult = Result<SpannedToken, LexerErrorKind>;

pub type LocatedTokenResult = Result<LocatedToken, LexerErrorKind>;

impl<'a> Lexer<'a> {
    /// Given a source file of noir code, return all the tokens in the file
    /// in order, along with any lexing errors that occurred.
    pub fn lex(source: &'a str, file_id: FileId) -> (Tokens, Vec<LexerErrorKind>) {
        let lexer = Lexer::new(source, file_id);
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

    pub fn new(source: &'a str, file_id: FileId) -> Self {
        Lexer {
            file_id,
            chars: source.char_indices(),
            position: 0,
            done: false,
            skip_comments: true,
            skip_whitespaces: true,
            max_integer: BigInt::from_biguint(num_bigint::Sign::Plus, FieldElement::modulus()) // cSpell:disable-line
                - BigInt::one(),
        }
    }

    pub fn new_with_dummy_file(source: &'a str) -> Self {
        Self::new(source, FileId::dummy())
    }

    pub fn skip_comments(mut self, flag: bool) -> Self {
        self.skip_comments = flag;
        self
    }

    pub fn skip_whitespaces(mut self, flag: bool) -> Self {
        self.skip_whitespaces = flag;
        self
    }

    pub fn set_skip_whitespaces_flag(&mut self, flag: bool) {
        self.skip_whitespaces = flag;
    }

    /// Iterates the cursor and returns the char at the new cursor position
    fn next_char(&mut self) -> Option<char> {
        let (position, ch) = self.chars.next()?;
        self.position = position as u32;
        Some(ch)
    }

    /// Peeks at the next char. Does not iterate the cursor
    fn peek_char(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, ch)| ch)
    }

    /// Peeks at the character two positions ahead. Does not iterate the cursor
    fn peek2_char(&self) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().map(|(_, ch)| ch)
    }

    /// Peeks at the next char and returns true if it is equal to the char argument
    fn peek_char_is(&self, ch: char) -> bool {
        self.peek_char() == Some(ch)
    }

    /// Peeks at the character two positions ahead and returns true if it is equal to the char argument
    fn peek2_char_is(&self, ch: char) -> bool {
        self.peek2_char() == Some(ch)
    }

    fn ampersand(&mut self) -> SpannedTokenResult {
        if self.peek_char_is('&') {
            let start = self.position;
            self.next_char();
            Ok(Token::LogicalAnd.into_span(start, start + 1))
        } else if self.peek_char_is('[') {
            self.single_char_token(Token::DeprecatedVectorStart)
        } else {
            self.single_char_token(Token::Ampersand)
        }
    }

    fn next_token(&mut self) -> LocatedTokenResult {
        self.next_spanned_token().map(|token| {
            let span = token.span();
            LocatedToken::new(token.into_token(), Location::new(span, self.file_id))
        })
    }

    fn next_spanned_token(&mut self) -> SpannedTokenResult {
        if !self.skip_comments {
            return self.next_spanned_token_without_checking_comments();
        }

        // Read tokens and skip comments. This is done like this to avoid recursion
        // and hitting stack overflow when there are many comments in a row.
        loop {
            let token = self.next_spanned_token_without_checking_comments()?;
            if matches!(token.token(), Token::LineComment(_, None) | Token::BlockComment(_, None)) {
                continue;
            }
            return Ok(token);
        }
    }

    /// Reads the next token, which might be a comment token (these aren't skipped in this method)
    fn next_spanned_token_without_checking_comments(&mut self) -> SpannedTokenResult {
        match self.next_char() {
            Some(x) if Self::is_code_whitespace(x) => {
                let spanned = self.eat_whitespace(x);
                if self.skip_whitespaces {
                    self.next_spanned_token_without_checking_comments()
                } else {
                    Ok(spanned)
                }
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
            Some('$') => self.single_char_token(Token::DollarSign),
            Some('@') => self.single_char_token(Token::At),
            Some('\\') => self.single_char_token(Token::Backslash),
            Some('"') => self.eat_string_literal(),
            Some('f') => self.eat_format_string_or_alpha_numeric(),
            Some('r') => self.eat_raw_string_or_alpha_numeric(),
            Some('q') => self.eat_quote_or_alpha_numeric(),
            Some('#') => self.eat_attribute_start(),
            // cSpell:disable
            Some(ch)
                if ch.is_whitespace()
                    // These aren't unicode whitespace but look like '' so they are also misleading
                    || ch == '\u{180E}'
                    || ch == '\u{200B}'
                    || ch == '\u{200C}'
                    || ch == '\u{200D}'
                    || ch == '\u{2060}'
                    || ch == '\u{FEFF}' =>
            {
                // cSpell:enable
                let span = Span::from(self.position..self.position + 1);
                let location = Location::new(span, self.file_id);
                self.next_char();
                Err(LexerErrorKind::UnicodeCharacterLooksLikeSpaceButIsItNot { char: ch, location })
            }
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

    /// If `single` is followed by `character` then extend it as `double`.
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

    /// Given that some tokens can contain two characters, such as <= , !=, >=, or even three like ..=
    /// Glue will take the first character of the token and check if it can be glued onto the next character(s)
    /// forming a double or triple token
    ///
    /// Returns an error if called with a token which cannot be extended with anything.
    fn glue(&mut self, prev_token: Token) -> SpannedTokenResult {
        match prev_token {
            Token::Dot => {
                if self.peek_char_is('.') && self.peek2_char_is('=') {
                    let start = self.position;
                    self.next_char();
                    self.next_char();
                    Ok(Token::DoubleDotEqual.into_span(start, start + 2))
                } else {
                    self.single_double_peek_token('.', prev_token, Token::DoubleDot)
                }
            }
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
            Token::Assign => {
                let start = self.position;
                if self.peek_char_is('=') {
                    self.next_char();
                    Ok(Token::Equal.into_span(start, start + 1))
                } else if self.peek_char_is('>') {
                    self.next_char();
                    Ok(Token::FatArrow.into_span(start, start + 1))
                } else {
                    Ok(prev_token.into_single_span(start))
                }
            }
            Token::Bang => self.single_double_peek_token('=', prev_token, Token::NotEqual),
            Token::Minus => self.single_double_peek_token('>', prev_token, Token::Arrow),
            Token::Colon => self.single_double_peek_token(':', prev_token, Token::DoubleColon),
            Token::Slash => {
                let start = self.position;

                if self.peek_char_is('/') {
                    self.next_char();
                    return self.parse_comment(start);
                } else if self.peek_char_is('*') {
                    self.next_char();
                    return self.parse_block_comment(start);
                }

                Ok(prev_token.into_single_span(start))
            }
            _ => Err(LexerErrorKind::NotADoubleChar {
                location: self.location(Span::single_char(self.position)),
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
            '0'..='9' => self.eat_digits(initial_char),
            _ => Err(LexerErrorKind::UnexpectedCharacter {
                location: self.location(Span::single_char(self.position)),
                found: initial_char.into(),
                expected: "an alpha numeric character".to_owned(),
            }),
        }
    }

    fn eat_attribute_start(&mut self) -> SpannedTokenResult {
        let start = self.position;

        let is_inner = if self.peek_char_is('!') {
            self.next_char();
            true
        } else {
            false
        };

        if !self.peek_char_is('[') {
            return Err(LexerErrorKind::UnexpectedCharacter {
                location: self.location(Span::single_char(self.position)),
                found: self.next_char(),
                expected: "[".to_owned(),
            });
        }
        self.next_char();

        let is_tag = self.peek_char_is('\'');
        if is_tag {
            self.next_char();
        }

        let end = self.position;

        Ok(Token::AttributeStart { is_inner, is_tag }.into_span(start, end))
    }

    //XXX(low): Can increase performance if we use iterator semantic and utilize some of the methods on String. See below
    // https://doc.rust-lang.org/stable/std/primitive.str.html#method.rsplit
    fn eat_word(&mut self, initial_char: char) -> SpannedTokenResult {
        let (start, word, end) = self.lex_word(initial_char);
        self.lookup_word_token(word, start, end)
    }

    /// Lex the next word in the input stream. Returns (start position, word, end position)
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

    fn eat_digits(&mut self, initial_char: char) -> SpannedTokenResult {
        let start = self.position;

        let original_str = self.eat_while(Some(initial_char), |ch| {
            // We eat any alphanumeric character. Even though we're only expecting
            // integers, we don't want to allow things like `1234abc` to be lexed
            // as an integer followed by an ident. We'd rather an invalid integer error here.
            // This also lets us parse integer type suffixes more easily.
            ch.is_ascii_alphanumeric() | (ch == '_')
        });

        let end = self.position;

        // Underscores needs to be stripped out before the literal can be converted to a `FieldElement.
        let mut integer_str = original_str.replace('_', "");
        let type_suffix = Self::check_for_integer_type_suffix(&mut integer_str);

        let bigint_result = match integer_str.strip_prefix("0x") {
            Some(integer_str) => BigInt::from_str_radix(integer_str, 16),
            None => BigInt::from_str(&integer_str),
        };

        let integer = match bigint_result {
            Ok(bigint) => {
                if bigint > self.max_integer {
                    return Err(LexerErrorKind::IntegerLiteralTooLarge {
                        location: self.location(Span::inclusive(start, end)),
                        limit: self.max_integer.to_string(),
                    });
                }
                let big_uint = bigint.magnitude();
                FieldElement::from_be_bytes_reduce(&big_uint.to_bytes_be())
            }
            Err(_) => {
                return Err(LexerErrorKind::InvalidIntegerLiteral {
                    location: self.location(Span::inclusive(start, end)),
                    found: original_str,
                });
            }
        };

        let integer_token = Token::Int(integer, type_suffix);
        Ok(integer_token.into_span(start, end))
    }

    /// Check for and return the type suffix on the integer string if it exists.
    /// If there is a type suffix, it is also stripped from the string
    fn check_for_integer_type_suffix(integer_string: &mut String) -> Option<IntegerTypeSuffix> {
        let cases = [
            ("i8", IntegerTypeSuffix::I8),
            ("i16", IntegerTypeSuffix::I16),
            ("i32", IntegerTypeSuffix::I32),
            ("i64", IntegerTypeSuffix::I64),
            ("u1", IntegerTypeSuffix::U1),
            ("u8", IntegerTypeSuffix::U8),
            ("u16", IntegerTypeSuffix::U16),
            ("u32", IntegerTypeSuffix::U32),
            ("u64", IntegerTypeSuffix::U64),
            ("u128", IntegerTypeSuffix::U128),
            ("Field", IntegerTypeSuffix::Field),
        ];

        let len = integer_string.len();
        for (suffix_string, suffix_value) in cases {
            if integer_string.ends_with(suffix_string) {
                integer_string.truncate(len - suffix_string.len());
                return Some(suffix_value);
            }
        }

        None
    }

    fn eat_string_literal(&mut self) -> SpannedTokenResult {
        let start = self.position;
        let mut string = String::new();

        loop {
            if let Some(next) = self.next_char() {
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
                            return Err(LexerErrorKind::InvalidEscape {
                                escaped,
                                location: self.location(span),
                            });
                        }
                        None => {
                            let span = Span::inclusive(start, self.position);
                            return Err(LexerErrorKind::UnterminatedStringLiteral {
                                location: self.location(span),
                            });
                        }
                    },
                    other => other,
                };

                string.push(char);
            } else {
                let span = Span::inclusive(start, self.position);
                return Err(LexerErrorKind::UnterminatedStringLiteral {
                    location: self.location(span),
                });
            }
        }

        let str_literal_token = Token::Str(string);
        let end = self.position;
        Ok(str_literal_token.into_span(start, end))
    }

    fn eat_fmt_string(&mut self) -> SpannedTokenResult {
        let start = self.position;
        self.next_char();

        let mut fragments = Vec::new();
        let mut length = 0;

        loop {
            // String fragment until '{' or '"'
            let mut string = String::new();
            let mut found_curly = false;

            loop {
                if let Some(next) = self.next_char() {
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
                                return Err(LexerErrorKind::InvalidEscape {
                                    escaped,
                                    location: self.location(span),
                                });
                            }
                            None => {
                                let span = Span::inclusive(start, self.position);
                                return Err(LexerErrorKind::UnterminatedStringLiteral {
                                    location: self.location(span),
                                });
                            }
                        },
                        '{' if self.peek_char_is('{') => {
                            self.next_char();
                            '{'
                        }
                        '}' if self.peek_char_is('}') => {
                            self.next_char();
                            '}'
                        }
                        '}' => {
                            let error_position = self.position;

                            // Keep consuming chars until we find the closing double quote
                            self.skip_until_string_end();

                            let span = Span::inclusive(error_position, error_position);
                            return Err(LexerErrorKind::InvalidFormatString {
                                found: '}',
                                location: self.location(span),
                            });
                        }
                        '{' => {
                            found_curly = true;
                            break;
                        }
                        other => other,
                    };

                    string.push(char);
                    length += 1;

                    if char == '{' || char == '}' {
                        // This might look a bit strange, but if there's `{{` or `}}` in the format string
                        // then it will be `{` and `}` in the string fragment respectively, but on the codegen
                        // phase it will be translated back to `{{` and `}}` to avoid executing an interpolation,
                        // thus the actual length of the codegen'd string will be one more than what we get here.
                        //
                        // We could just make the fragment include the double curly braces, but then the interpreter
                        // would need to undo the curly braces, so it's simpler to add them during codegen.
                        length += 1;
                    }
                } else {
                    let span = Span::inclusive(start, self.position);
                    return Err(LexerErrorKind::UnterminatedStringLiteral {
                        location: self.location(span),
                    });
                }
            }

            if !string.is_empty() {
                fragments.push(FmtStrFragment::String(string));
            }

            if !found_curly {
                break;
            }

            length += 1; // for the curly brace

            // Interpolation fragment until '}' or '"'
            let mut string = String::new();
            let interpolation_start = self.position + 1; // + 1 because we are at '{'
            let mut first_char = true;
            while let Some(next) = self.next_char() {
                let char = match next {
                    '}' => {
                        if string.is_empty() {
                            let error_position = self.position;

                            // Keep consuming chars until we find the closing double quote
                            self.skip_until_string_end();

                            let span = Span::inclusive(error_position, error_position);
                            return Err(LexerErrorKind::EmptyFormatStringInterpolation {
                                location: self.location(span),
                            });
                        }

                        break;
                    }
                    other => {
                        let is_valid_char = if first_char {
                            other.is_ascii_alphabetic() || other == '_'
                        } else {
                            other.is_ascii_alphanumeric() || other == '_'
                        };
                        if !is_valid_char {
                            let error_position = self.position;

                            // Keep consuming chars until we find the closing double quote
                            // (unless we bumped into a double quote now, in which case we are done)
                            if other != '"' {
                                self.skip_until_string_end();
                            }

                            let span = Span::inclusive(error_position, error_position);
                            return Err(LexerErrorKind::InvalidFormatString {
                                found: other,
                                location: self.location(span),
                            });
                        }
                        first_char = false;
                        other
                    }
                };
                length += 1;
                string.push(char);
            }

            length += 1; // for the closing curly brace

            let span = if interpolation_start <= self.position {
                Span::from(interpolation_start..self.position)
            } else {
                // This can happen if the interpolation ends abruptly on EOF
                Span::single_char(interpolation_start)
            };
            let location = Location::new(span, self.file_id);
            fragments.push(FmtStrFragment::Interpolation(string, location));
        }

        let token = Token::FmtStr(fragments, length);
        let end = self.position;
        Ok(token.into_span(start, end))
    }

    fn skip_until_string_end(&mut self) {
        while let Some(next) = self.next_char() {
            if next == '\'' && self.peek_char_is('"') {
                self.next_char();
            } else if next == '"' {
                break;
            }
        }
    }

    fn eat_format_string_or_alpha_numeric(&mut self) -> SpannedTokenResult {
        if self.peek_char_is('"') { self.eat_fmt_string() } else { self.eat_alpha_numeric('f') }
    }

    fn eat_raw_string(&mut self) -> SpannedTokenResult {
        let start = self.position;

        let beginning_hashes = self.eat_while(None, |ch| ch == '#');
        let beginning_hashes_count = beginning_hashes.chars().count();
        if beginning_hashes_count > 255 {
            // too many hashes (unlikely in practice)
            // also, Rust disallows 256+ hashes as well
            return Err(LexerErrorKind::UnexpectedCharacter {
                location: self.location(Span::single_char(start + 255)),
                found: Some('#'),
                expected: "\"".to_owned(),
            });
        }

        if !self.peek_char_is('"') {
            return Err(LexerErrorKind::UnexpectedCharacter {
                location: self.location(Span::single_char(self.position)),
                found: self.next_char(),
                expected: "\"".to_owned(),
            });
        }
        self.next_char();

        let mut str_literal = String::new();
        loop {
            let chars = self.eat_while(None, |ch| ch != '"');
            str_literal.push_str(&chars[..]);
            if !self.peek_char_is('"') {
                return Err(LexerErrorKind::UnexpectedCharacter {
                    location: self.location(Span::single_char(self.position)),
                    found: self.next_char(),
                    expected: "\"".to_owned(),
                });
            }
            self.next_char();
            let mut ending_hashes_count = 0;
            while let Some('#') = self.peek_char() {
                if ending_hashes_count == beginning_hashes_count {
                    break;
                }
                self.next_char();
                ending_hashes_count += 1;
            }
            if ending_hashes_count == beginning_hashes_count {
                break;
            } else {
                str_literal.push('"');
                for _ in 0..ending_hashes_count {
                    str_literal.push('#');
                }
            }
        }

        let str_literal_token = Token::RawStr(str_literal, beginning_hashes_count as u8);

        let end = self.position;
        Ok(str_literal_token.into_span(start, end))
    }

    fn eat_raw_string_or_alpha_numeric(&mut self) -> SpannedTokenResult {
        // Problem: we commit to eating raw strings once we see one or two characters.
        // This is unclean, but likely ok in all practical cases, and works with existing
        // `Lexer` methods.
        let peek1 = self.peek_char().unwrap_or('X');
        let peek2 = self.peek2_char().unwrap_or('X');
        match (peek1, peek2) {
            ('#', '#') | ('#', '"') | ('"', _) => self.eat_raw_string(),
            _ => self.eat_alpha_numeric('r'),
        }
    }

    fn eat_quote_or_alpha_numeric(&mut self) -> SpannedTokenResult {
        let (start, word, end) = self.lex_word('q');
        if word != "quote" {
            return self.lookup_word_token(word, start, end);
        }

        let mut delimiter = self.next_token()?;
        while let Token::Whitespace(_) = delimiter.token() {
            delimiter = self.next_token()?;
        }

        let (start_delim, end_delim) = match delimiter.token() {
            Token::LeftBrace => (Token::LeftBrace, Token::RightBrace),
            Token::LeftBracket => (Token::LeftBracket, Token::RightBracket),
            Token::LeftParen => (Token::LeftParen, Token::RightParen),
            _ => return Err(LexerErrorKind::InvalidQuoteDelimiter { delimiter }),
        };

        let mut tokens = Vec::new();

        // Keep track of each nested delimiter we need to close.
        let mut nested_delimiters = vec![delimiter];

        while !nested_delimiters.is_empty() {
            let token = self.next_token()?;

            if *token.token() == start_delim {
                nested_delimiters.push(token.clone());
            } else if *token.token() == end_delim {
                nested_delimiters.pop();
            } else if *token.token() == Token::EOF {
                let start_delim =
                    nested_delimiters.pop().expect("If this were empty, we wouldn't be looping");
                return Err(LexerErrorKind::UnclosedQuote { start_delim, end_delim });
            }

            tokens.push(token);
        }

        // Pop the closing delimiter from the token stream
        if !tokens.is_empty() {
            tokens.pop();
        }

        let end = self.position;
        Ok(Token::Quote(Tokens(tokens)).into_span(start, end))
    }

    fn parse_comment(&mut self, start: u32) -> SpannedTokenResult {
        let doc_style = match self.peek_char() {
            Some('!') => {
                self.next_char();
                Some(DocStyle::Inner)
            }
            Some('/') if self.peek2_char() != '/'.into() => {
                self.next_char();
                Some(DocStyle::Outer)
            }
            _ => None,
        };
        let comment = self.eat_while(None, |ch| ch != '\n');

        if !comment.is_ascii() {
            let span = Span::from(start..self.position);
            return Err(LexerErrorKind::NonAsciiComment { location: self.location(span) });
        }

        Ok(Token::LineComment(comment, doc_style).into_span(start, self.position))
    }

    fn parse_block_comment(&mut self, start: u32) -> SpannedTokenResult {
        let doc_style = match self.peek_char() {
            Some('!') => {
                self.next_char();
                Some(DocStyle::Inner)
            }
            Some('*') if !matches!(self.peek2_char(), Some('*' | '/')) => {
                self.next_char();
                Some(DocStyle::Outer)
            }
            _ => None,
        };

        let mut depth = 1usize;

        let mut content = String::new();
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
                ch => content.push(ch),
            }
        }
        if depth == 0 {
            if !content.is_ascii() {
                let span = Span::from(start..self.position);
                return Err(LexerErrorKind::NonAsciiComment { location: self.location(span) });
            }

            Ok(Token::BlockComment(content, doc_style).into_span(start, self.position))
        } else {
            let span = Span::inclusive(start, self.position);
            Err(LexerErrorKind::UnterminatedBlockComment { location: self.location(span) })
        }
    }

    fn is_code_whitespace(c: char) -> bool {
        c == '\t' || c == '\n' || c == '\r' || c == ' '
    }

    /// Skips white space. They are not significant in the source language
    fn eat_whitespace(&mut self, initial_char: char) -> SpannedToken {
        let start = self.position;
        let whitespace = self.eat_while(initial_char.into(), Self::is_code_whitespace);
        SpannedToken::new(Token::Whitespace(whitespace), Span::inclusive(start, self.position))
    }

    fn location(&self, span: Span) -> Location {
        Location::new(span, self.file_id)
    }
}

impl Iterator for Lexer<'_> {
    type Item = LocatedTokenResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { None } else { Some(self.next_token()) }
    }
}

#[cfg(test)]
mod tests {
    use iter_extended::vecmap;

    use super::*;

    #[test]
    fn test_single_multi_char() {
        let input = "! != + ( ) { } [ ] | , ; : :: < <= > >= & - -> . .. ..= % / * = == << >>";

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
            Token::DoubleDotEqual,
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

        let mut lexer = Lexer::new_with_dummy_file(input);

        for token in expected.into_iter() {
            let got = lexer.next_token().unwrap();
            assert_eq!(got, token);
        }
    }

    #[test]
    fn invalid_attribute() {
        let input = "#";
        let mut lexer = Lexer::new_with_dummy_file(input);

        let token = lexer.next().unwrap();
        assert!(token.is_err());
    }

    #[test]
    fn test_attribute_start() {
        let input = r#"#[something]"#;
        let mut lexer = Lexer::new_with_dummy_file(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.token(), &Token::AttributeStart { is_inner: false, is_tag: false });
    }

    #[test]
    fn test_attribute_start_with_tag() {
        let input = r#"#['something]"#;
        let mut lexer = Lexer::new_with_dummy_file(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.token(), &Token::AttributeStart { is_inner: false, is_tag: true });
    }

    #[test]
    fn test_inner_attribute_start() {
        let input = r#"#![something]"#;
        let mut lexer = Lexer::new_with_dummy_file(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.token(), &Token::AttributeStart { is_inner: true, is_tag: false });
    }

    #[test]
    fn test_inner_attribute_start_with_tag() {
        let input = r#"#!['something]"#;
        let mut lexer = Lexer::new_with_dummy_file(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.token(), &Token::AttributeStart { is_inner: true, is_tag: true });
    }

    #[test]
    fn test_int_too_large() {
        let modulus = FieldElement::modulus();
        let input = modulus.to_string();

        let mut lexer = Lexer::new_with_dummy_file(&input);
        let token = lexer.next_token();
        assert!(
            matches!(token, Err(LexerErrorKind::IntegerLiteralTooLarge { .. })),
            "expected {input} to throw error"
        );
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

        let mut lexer = Lexer::new_with_dummy_file(input);
        for token in expected.into_iter() {
            let got = lexer.next_token().unwrap();
            assert_eq!(got, token);
        }
    }

    #[test]
    fn unterminated_block_comment() {
        let input = "/*/";

        let mut lexer = Lexer::new_with_dummy_file(input);
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
            Token::Int(FieldElement::from(5_i128), None),
        ];

        let mut lexer = Lexer::new_with_dummy_file(input);
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
            Token::Int(FieldElement::from(5_i128), None),
        ];

        let mut lexer = Lexer::new_with_dummy_file(input);
        for token in expected.into_iter() {
            let first_lexer_output = lexer.next_token().unwrap();
            assert_eq!(first_lexer_output, token);
        }
    }

    #[test]
    fn test_comments() {
        let input = "
            // comment
            /// comment
            //! comment
            /* comment */
            /** outer doc block */
            /*! inner doc block */
        ";
        let expected = [
            Token::LineComment(" comment".into(), None),
            Token::LineComment(" comment".into(), DocStyle::Outer.into()),
            Token::LineComment(" comment".into(), DocStyle::Inner.into()),
            Token::BlockComment(" comment ".into(), None),
            Token::BlockComment(" outer doc block ".into(), DocStyle::Outer.into()),
            Token::BlockComment(" inner doc block ".into(), DocStyle::Inner.into()),
        ];

        let mut lexer = Lexer::new_with_dummy_file(input).skip_comments(false);
        for token in expected {
            let first_lexer_output = lexer.next_token().unwrap();
            assert_eq!(token, first_lexer_output);
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
            Token::Int(FieldElement::from(5_i128), None),
        ];

        let mut lexer = Lexer::new_with_dummy_file(input);
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
        let mut lexer = Lexer::new_with_dummy_file(input);

        for token in expected.into_iter() {
            let got = lexer.next_token().unwrap();
            assert_eq!(got, token);
        }
    }

    #[test]
    fn test_eat_string_literal_with_escapes() {
        let input = "let _word = \"hello\\n\\t\"";

        let expected = vec![
            Token::Keyword(Keyword::Let),
            Token::Ident("_word".to_string()),
            Token::Assign,
            Token::Str("hello\n\t".to_string()),
        ];
        let mut lexer = Lexer::new_with_dummy_file(input);

        for token in expected.into_iter() {
            let got = lexer.next_token().unwrap();
            assert_eq!(got, token);
        }
    }

    #[test]
    fn test_eat_string_literal_missing_double_quote() {
        let input = "\"hello";
        let mut lexer = Lexer::new_with_dummy_file(input);
        assert!(matches!(
            lexer.next_token(),
            Err(LexerErrorKind::UnterminatedStringLiteral { .. })
        ));
    }

    #[test]
    fn test_eat_fmt_string_literal_without_interpolations() {
        let input = "let _word = f\"hello\"";

        let expected = vec![
            Token::Keyword(Keyword::Let),
            Token::Ident("_word".to_string()),
            Token::Assign,
            Token::FmtStr(vec![FmtStrFragment::String("hello".to_string())], 5),
        ];
        let mut lexer = Lexer::new_with_dummy_file(input);

        for token in expected.into_iter() {
            let got = lexer.next_token().unwrap();
            assert_eq!(got, token);
        }
    }

    #[test]
    fn test_eat_fmt_string_literal_with_escapes_without_interpolations() {
        let input = "let _word = f\"hello\\n\\t{{x}}\"";

        let expected = vec![
            Token::Keyword(Keyword::Let),
            Token::Ident("_word".to_string()),
            Token::Assign,
            Token::FmtStr(vec![FmtStrFragment::String("hello\n\t{x}".to_string())], 12),
        ];
        let mut lexer = Lexer::new_with_dummy_file(input);

        for token in expected.into_iter() {
            let got = lexer.next_token().unwrap();
            assert_eq!(got, token);
        }
    }

    #[test]
    fn test_eat_fmt_string_literal_with_interpolations() {
        let input = "let _word = f\"hello {world} and {_another} {vAr_123}\"";
        let file = FileId::dummy();

        let expected = vec![
            Token::Keyword(Keyword::Let),
            Token::Ident("_word".to_string()),
            Token::Assign,
            Token::FmtStr(
                vec![
                    FmtStrFragment::String("hello ".to_string()),
                    FmtStrFragment::Interpolation(
                        "world".to_string(),
                        Location::new(Span::from(21..26), file),
                    ),
                    FmtStrFragment::String(" and ".to_string()),
                    FmtStrFragment::Interpolation(
                        "_another".to_string(),
                        Location::new(Span::from(33..41), file),
                    ),
                    FmtStrFragment::String(" ".to_string()),
                    FmtStrFragment::Interpolation(
                        "vAr_123".to_string(),
                        Location::new(Span::from(44..51), file),
                    ),
                ],
                38,
            ),
        ];
        let mut lexer = Lexer::new_with_dummy_file(input);

        for token in expected.into_iter() {
            let got = lexer.next_token().unwrap().into_token();
            assert_eq!(got, token);
        }
    }

    #[test]
    fn test_eat_fmt_string_literal_missing_double_quote() {
        let input = "f\"hello";
        let mut lexer = Lexer::new_with_dummy_file(input);
        assert!(matches!(
            lexer.next_token(),
            Err(LexerErrorKind::UnterminatedStringLiteral { .. })
        ));
    }

    #[test]
    fn test_eat_fmt_string_literal_invalid_char_in_interpolation() {
        let input = "f\"hello {foo.bar}\" true";
        let mut lexer = Lexer::new_with_dummy_file(input);
        assert!(matches!(lexer.next_token(), Err(LexerErrorKind::InvalidFormatString { .. })));

        // Make sure the lexer went past the ending double quote for better recovery
        let token = lexer.next_token().unwrap().into_token();
        assert!(matches!(token, Token::Bool(true)));
    }

    #[test]
    fn test_eat_fmt_string_literal_double_quote_inside_interpolation() {
        let input = "f\"hello {world\" true";
        let mut lexer = Lexer::new_with_dummy_file(input);
        assert!(matches!(lexer.next_token(), Err(LexerErrorKind::InvalidFormatString { .. })));

        // Make sure the lexer stopped parsing the string literal when it found \" inside the interpolation
        let token = lexer.next_token().unwrap().into_token();
        assert!(matches!(token, Token::Bool(true)));
    }

    #[test]
    fn test_eat_fmt_string_literal_unmatched_closing_curly() {
        let input = "f\"hello }\" true";
        let mut lexer = Lexer::new_with_dummy_file(input);
        assert!(matches!(lexer.next_token(), Err(LexerErrorKind::InvalidFormatString { .. })));

        // Make sure the lexer went past the ending double quote for better recovery
        let token = lexer.next_token().unwrap().into_token();
        assert!(matches!(token, Token::Bool(true)));
    }

    #[test]
    fn test_eat_fmt_string_literal_empty_interpolation() {
        let input = "f\"{}\" true";
        let mut lexer = Lexer::new_with_dummy_file(input);
        assert!(matches!(
            lexer.next_token(),
            Err(LexerErrorKind::EmptyFormatStringInterpolation { .. })
        ));

        // Make sure the lexer went past the ending double quote for better recovery
        let token = lexer.next_token().unwrap().into_token();
        assert!(matches!(token, Token::Bool(true)));
    }

    #[test]
    fn test_eat_integer_literals() {
        let test_cases: Vec<(&str, Token)> = vec![
            ("0x05", Token::Int(5_i128.into(), None)),
            ("5", Token::Int(5_i128.into(), None)),
            ("0x1234_5678", Token::Int(0x1234_5678_u128.into(), None)),
            ("0x_01", Token::Int(0x1_u128.into(), None)),
            ("1_000_000", Token::Int(1_000_000_u128.into(), None)),
            ("1__0___Field", Token::Int(10_u32.into(), Some(IntegerTypeSuffix::Field))),
            ("0x1u1", Token::Int(1_u32.into(), Some(IntegerTypeSuffix::U1))),
            ("97_i64", Token::Int(97_u32.into(), Some(IntegerTypeSuffix::I64))),
            ("97_u128", Token::Int(97_u32.into(), Some(IntegerTypeSuffix::U128))),
        ];

        for (input, expected_token) in test_cases {
            let mut lexer = Lexer::new_with_dummy_file(input);
            let got = lexer.next_token().unwrap();
            assert_eq!(got.token(), &expected_token);
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
        let int_token = Token::Int(5_i128.into(), None).into_single_span(int_position);

        let expected = vec![let_token, ident_token, assign_token, int_token];
        let mut lexer = Lexer::new_with_dummy_file(input);

        for spanned_token in expected.into_iter() {
            let got = lexer.next_token().unwrap();
            assert_eq!(got.span(), spanned_token.span());
            assert_eq!(got.into_spanned_token(), spanned_token);
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
            Token::Int(5_i128.into(), None),
            Token::Semicolon,
            Token::Keyword(Keyword::Let),
            Token::Ident("ten".to_string()),
            Token::Colon,
            Token::Ident("Field".to_string()),
            Token::Assign,
            Token::Int(10_i128.into(), None),
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
            Token::Int(50_i128.into(), None),
            Token::Semicolon,
            Token::Keyword(Keyword::Assert),
            Token::LeftParen,
            Token::Ident("ten".to_string()),
            Token::Plus,
            Token::Ident("five".to_string()),
            Token::Equal,
            Token::Int(15_i128.into(), None),
            Token::RightParen,
            Token::Semicolon,
            Token::EOF,
        ];
        let mut lexer = Lexer::new_with_dummy_file(input);

        for token in expected.into_iter() {
            let got = lexer.next_token().unwrap();
            assert_eq!(got, token);
        }
    }

    // returns a vector of:
    //   (expected_token_discriminator, strings_to_lex)
    // expected_token_discriminator matches a given token when
    // std::mem::discriminant returns the same discriminant for both.
    fn big_list_base64_to_statements(base64_str: String) -> Vec<(Option<Token>, Vec<String>)> {
        use base64::engine::general_purpose;
        use std::borrow::Cow;
        use std::io::Cursor;
        use std::io::Read;

        let mut wrapped_reader = Cursor::new(base64_str);
        let mut decoder =
            base64::read::DecoderReader::new(&mut wrapped_reader, &general_purpose::STANDARD);
        let mut base64_decoded = Vec::new();
        decoder.read_to_end(&mut base64_decoded).unwrap();

        // NOTE: when successful, this is the same conversion method as used in
        // noirc_driver::stdlib::stdlib_paths_with_source, viz.
        //
        // let source = std::str::from_utf8(..).unwrap().to_string();
        let s: Cow<'_, str> = match std::str::from_utf8(&base64_decoded) {
            Ok(s) => Cow::Borrowed(s),
            Err(_err) => {
                // recover as much of the string as possible
                // when str::from_utf8 fails
                String::from_utf8_lossy(&base64_decoded)
            }
        };

        vec![
            // Token::Ident(_)
            (None, vec![format!("let \"{s}\" = ();")]),
            (Some(Token::Str("".to_string())), vec![format!("let s = \"{s}\";")]),
            (
                Some(Token::RawStr("".to_string(), 0)),
                vec![
                    // let s = r"Hello world";
                    format!("let s = r\"{s}\";"),
                    // let s = r#"Simon says "hello world""#;
                    format!("let s = r#\"{s}\"#;"),
                    // // Any number of hashes may be used (>= 1) as long as the string also terminates with the same number of hashes
                    // let s = r#####"One "#, Two "##, Three "###, Four "####, Five will end the string."#####;
                    format!("let s = r##\"{s}\"##;"),
                    format!("let s = r###\"{s}\"###;"),
                    format!("let s = r####\"{s}\"####; "),
                    format!("let s = r#####\"{s}\"#####;"),
                ],
            ),
            (Some(Token::FmtStr(vec![], 0)), vec![format!("assert(x == y, f\"{s}\");")]),
            // expected token not found
            // (Some(Token::LineComment("".to_string(), None)), vec![
            (None, vec![format!("//{s}"), format!("// {s}")]),
            // expected token not found
            // (Some(Token::BlockComment("".to_string(), None)), vec![
            (None, vec![format!("/*{s}*/"), format!("/* {s} */"), format!("/*\n{s}\n*/")]),
        ]
    }

    #[test]
    fn test_big_list_of_naughty_strings() {
        use std::mem::discriminant;

        let big_list_contents = include_str!("./blns/blns.base64.json"); // cSpell:disable-line
        let big_list_base64: Vec<String> =
            serde_json::from_str(big_list_contents).expect("BLNS json invalid"); // cSpell:disable-line
        for big_list_base64_str in big_list_base64 {
            let statements = big_list_base64_to_statements(big_list_base64_str);
            for (token_discriminator_opt, big_list_program_strings) in statements {
                for big_list_program_str in big_list_program_strings {
                    let mut expected_token_found = false;
                    let mut lexer = Lexer::new_with_dummy_file(&big_list_program_str);
                    let mut result_tokens = Vec::new();
                    loop {
                        match lexer.next_token() {
                            Ok(next_token) => {
                                result_tokens.push(next_token.clone());
                                expected_token_found |= token_discriminator_opt
                                    .as_ref()
                                    .map(|token_discriminator| {
                                        discriminant(token_discriminator)
                                            == discriminant(next_token.token())
                                    })
                                    .unwrap_or(true);

                                if next_token == Token::EOF {
                                    assert!(lexer.done, "lexer not done when EOF emitted!");
                                    break;
                                }
                            }

                            Err(LexerErrorKind::InvalidIntegerLiteral { .. })
                            | Err(LexerErrorKind::UnexpectedCharacter { .. })
                            | Err(LexerErrorKind::NonAsciiComment { .. })
                            | Err(LexerErrorKind::UnterminatedBlockComment { .. })
                            | Err(LexerErrorKind::UnterminatedStringLiteral { .. })
                            | Err(LexerErrorKind::InvalidFormatString { .. }) => {
                                expected_token_found = true;
                            }
                            Err(err) => {
                                panic!(
                                    "Unexpected lexer error found {err:?} for input string {big_list_program_str:?}"
                                )
                            }
                        }
                    }

                    assert!(
                        expected_token_found,
                        "expected token not found: {token_discriminator_opt:?}\noutput:\n{result_tokens:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn test_quote() {
        // cases is a vector of pairs of (test string, expected # of tokens in token stream)
        let cases = vec![
            ("quote {}", 0),
            ("quote { a.b }", 3),
            ("quote { ) ( }", 2), // invalid syntax is fine in a quote
            ("quote { { } }", 2), // Nested `{` and `}` shouldn't close the quote as long as they are matched.
            ("quote { 1 { 2 { 3 { 4 { 5 } 4 4 } 3 3 } 2 2 } 1 1 }", 21),
            ("quote [ } } ]", 2), // In addition to `{}`, `[]`, and `()` can also be used as delimiters.
            ("quote [ } foo[] } ]", 5),
            ("quote ( } () } )", 4),
        ];

        for (source, expected_stream_length) in cases {
            let mut tokens =
                vecmap(Lexer::new_with_dummy_file(source), |result| result.unwrap().into_token());

            // All examples should be a single TokenStream token followed by an EOF token.
            assert_eq!(tokens.len(), 2, "Unexpected token count: {tokens:?}");

            tokens.pop();
            match tokens.pop().unwrap() {
                Token::Quote(stream) => assert_eq!(stream.0.len(), expected_stream_length),
                other => panic!(
                    "test_quote test failure! Expected a single TokenStream token, got {other} for input `{source}`"
                ),
            }
        }
    }

    #[test]
    fn test_unclosed_quote() {
        let cases = vec!["quote {", "quote { {  }", "quote [ []", "quote (((((((())))"];

        for source in cases {
            // `quote` is not itself a keyword so if the token stream fails to
            // parse we don't expect any valid tokens from the quote construct
            for token in Lexer::new_with_dummy_file(source) {
                assert!(token.is_err(), "Expected Err, found {token:?}");
            }
        }
    }

    #[test]
    fn test_non_ascii_comments() {
        // cSpell:disable-next-line
        let cases = vec!["// 🙂", "// schön", "/* in the middle 🙂 of a comment */"];

        for source in cases {
            let mut lexer = Lexer::new_with_dummy_file(source);
            assert!(
                lexer.any(|token| matches!(token, Err(LexerErrorKind::NonAsciiComment { .. }))),
                "Expected NonAsciiComment error"
            );
        }
    }

    #[test]
    fn errors_on_non_unicode_whitespace() {
        let str = "\u{0085}";
        let mut lexer = Lexer::new_with_dummy_file(str);
        assert!(matches!(
            lexer.next_token(),
            Err(LexerErrorKind::UnicodeCharacterLooksLikeSpaceButIsItNot { .. })
        ));
    }

    #[test]
    fn does_not_crash_on_format_string_with_broken_interpolation() {
        let str = "f\"{";
        let mut lexer = Lexer::new_with_dummy_file(str);
        let _ = lexer.next_token();
    }
}
