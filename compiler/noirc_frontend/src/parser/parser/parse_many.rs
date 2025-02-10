use crate::token::Token;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parses a list of items separated by a token, optionally ending when another token is found.
    /// The given function `f` must parse one item (eventually parsing many, if separators are found).
    /// If no item is parsed, `f` must report an error and return `None`.
    pub(super) fn parse_many<T, F>(
        &mut self,
        items: &'static str,
        separated_by: SeparatedBy,
        f: F,
    ) -> Vec<T>
    where
        F: FnMut(&mut Parser<'a>) -> Option<T>,
    {
        self.parse_many_return_trailing_separator_if_any(items, separated_by, f).0
    }

    /// parse_many, where the given function `f` may return multiple results
    pub(super) fn parse_many_to_many<T, F>(
        &mut self,
        items: &'static str,
        separated_by: SeparatedBy,
        f: F,
    ) -> Vec<T>
    where
        F: FnMut(&mut Parser<'a>) -> Vec<T>,
    {
        self.parse_many_to_many_return_trailing_separator_if_any(items, separated_by, f).0
    }

    /// Same as parse_many, but returns a bool indicating whether a trailing separator was found.
    pub(super) fn parse_many_return_trailing_separator_if_any<T, F>(
        &mut self,
        items: &'static str,
        separated_by: SeparatedBy,
        mut f: F,
    ) -> (Vec<T>, bool)
    where
        F: FnMut(&mut Parser<'a>) -> Option<T>,
    {
        let f = |x: &mut Parser<'a>| {
            if let Some(result) = f(x) {
                vec![result]
            } else {
                vec![]
            }
        };
        self.parse_many_to_many_return_trailing_separator_if_any(items, separated_by, f)
    }

    /// Same as parse_many, but returns a bool indicating whether a trailing separator was found.
    fn parse_many_to_many_return_trailing_separator_if_any<T, F>(
        &mut self,
        items: &'static str,
        separated_by: SeparatedBy,
        mut f: F,
    ) -> (Vec<T>, bool)
    where
        F: FnMut(&mut Parser<'a>) -> Vec<T>,
    {
        let mut elements: Vec<T> = Vec::new();
        let mut trailing_separator = false;
        loop {
            if let Some(end) = &separated_by.until {
                if self.eat(end.clone()) {
                    break;
                }
            }

            let start_span = self.current_token_span;
            let mut new_elements = f(self);
            if new_elements.is_empty() {
                if let Some(end) = &separated_by.until {
                    self.eat(end.clone());
                }
                break;
            }

            if let Some(separator) = &separated_by.token {
                if !trailing_separator && !elements.is_empty() {
                    self.expected_token_separating_items(separator.clone(), items, start_span);
                }
            }

            elements.append(&mut new_elements);

            trailing_separator = if let Some(separator) = &separated_by.token {
                self.eat(separator.clone())
            } else {
                true
            };

            if !trailing_separator && !separated_by.continue_if_separator_is_missing {
                if let Some(end) = &separated_by.until {
                    self.eat(end.clone());
                }
                break;
            }
        }

        (elements, trailing_separator)
    }
}

pub(super) struct SeparatedBy {
    pub(super) token: Option<Token>,
    pub(super) until: Option<Token>,
    pub(super) continue_if_separator_is_missing: bool,
}

impl SeparatedBy {
    pub(super) fn until(self, token: Token) -> SeparatedBy {
        SeparatedBy { until: Some(token), ..self }
    }

    pub(super) fn stop_if_separator_is_missing(self) -> SeparatedBy {
        SeparatedBy { continue_if_separator_is_missing: false, ..self }
    }
}

pub(super) fn separated_by(token: Token) -> SeparatedBy {
    SeparatedBy { token: Some(token), until: None, continue_if_separator_is_missing: true }
}

pub(super) fn separated_by_comma() -> SeparatedBy {
    separated_by(Token::Comma)
}

pub(super) fn separated_by_comma_until_right_paren() -> SeparatedBy {
    separated_by_comma().until(Token::RightParen)
}

pub(super) fn separated_by_comma_until_right_brace() -> SeparatedBy {
    separated_by_comma().until(Token::RightBrace)
}

pub(super) fn without_separator() -> SeparatedBy {
    SeparatedBy { token: None, until: None, continue_if_separator_is_missing: true }
}
