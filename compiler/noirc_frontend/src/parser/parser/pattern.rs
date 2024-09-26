use crate::ast::Pattern;

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        if let Some(ident) = self.eat_ident() {
            return Pattern::Identifier(ident);
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
}
