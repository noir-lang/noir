use crate::{
    ast::Pattern,
    token::{Keyword, Token, TokenKind},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        let start_span = self.current_token_span;

        let mutable = self.eat_keyword(Keyword::Mut);
        let pattern = self.parse_pattern_no_mut();
        if mutable {
            Pattern::Mutable(
                Box::new(pattern),
                self.span_since(start_span),
                false, // is synthesized
            )
        } else {
            pattern
        }
    }

    fn parse_pattern_no_mut(&mut self) -> Pattern {
        if let Some(ident) = self.eat_ident() {
            return Pattern::Identifier(ident);
        }

        if let Some(token) = self.eat_kind(TokenKind::InternedPattern) {
            match token.into_token() {
                Token::InternedPattern(pattern) => {
                    return Pattern::Interned(pattern, self.previous_token_span)
                }
                _ => unreachable!(),
            }
        }

        if self.eat_left_paren() {
            let start_span = self.current_token_span;
            let mut patterns = Vec::new();
            loop {
                if self.eat_right_paren() {
                    break;
                }

                patterns.push(self.parse_pattern());

                self.eat_commas();
            }

            return Pattern::Tuple(patterns, self.span_since(start_span));
        }

        // TODO: parse other patterns
        todo!("Parser")
    }
}

#[cfg(test)]
mod tests {

    use crate::{ast::Pattern, parser::Parser};

    #[test]
    fn parses_identifier_pattern() {
        let src = "foo";
        let typ = Parser::for_str(src).parse_pattern();
        let Pattern::Identifier(ident) = typ else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_mutable_pattern() {
        let src = "mut foo";
        let typ = Parser::for_str(src).parse_pattern();
        let Pattern::Mutable(pattern, _, _) = typ else { panic!("Expected a mutable pattern") };
        let pattern: &Pattern = &pattern;
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_tuple_pattern() {
        let src = "(foo, bar)";
        let typ = Parser::for_str(src).parse_pattern();
        let Pattern::Tuple(mut patterns, _) = typ else { panic!("Expected a tuple pattern") };
        assert_eq!(patterns.len(), 2);

        let pattern = patterns.remove(0);
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");

        let pattern = patterns.remove(0);
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "bar");
    }
}
