use crate::{
    ast::{GenericTypeArgs, Path, PathKind, TraitBound, UnresolvedTraitConstraint},
    parser::labels::ParsingRuleLabel,
    token::{Keyword, Token},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_where_clause(&mut self) -> Vec<UnresolvedTraitConstraint> {
        let mut constraints = Vec::new();

        if !self.eat_keyword(Keyword::Where) {
            return constraints;
        }

        let mut trailing_comma = false;
        loop {
            let start_span = self.current_token_span;
            let Some(typ) = self.parse_type() else {
                break;
            };

            if !trailing_comma && !constraints.is_empty() {
                self.expected_token_separating_items(",", "trait bounds", start_span);
            }

            self.eat_or_error(Token::Colon);

            let trait_bounds = self.parse_trait_bounds();
            for trait_bound in trait_bounds {
                constraints.push(UnresolvedTraitConstraint { typ: typ.clone(), trait_bound });
            }

            trailing_comma = self.eat_commas();
        }

        if constraints.is_empty() {
            // TODO: error? (`where` but no constrains)
        }

        constraints
    }

    pub(super) fn parse_trait_bounds(&mut self) -> Vec<TraitBound> {
        let mut bounds = Vec::new();

        let mut trailing_plus = false;
        loop {
            let start_span = self.current_token_span;
            let Some(bound) = self.parse_trait_bound() else {
                break;
            };

            if !trailing_plus && !bounds.is_empty() {
                self.expected_token_separating_items("+", "trait bounds", start_span);
            }

            bounds.push(bound);

            trailing_plus = self.eat_plus();
        }

        bounds
    }

    pub(crate) fn parse_trait_bound_or_error(&mut self) -> TraitBound {
        if let Some(trait_bound) = self.parse_trait_bound() {
            return trait_bound;
        }

        self.expected_label(ParsingRuleLabel::TraitBound);
        TraitBound {
            trait_path: Path {
                kind: PathKind::Plain,
                segments: Vec::new(),
                span: self.span_at_previous_token_end(),
            },
            trait_id: None,
            trait_generics: GenericTypeArgs::default(),
        }
    }

    pub(crate) fn parse_trait_bound(&mut self) -> Option<TraitBound> {
        let trait_path = self.parse_path_no_turbofish()?;
        let trait_generics = self.parse_generic_type_args();
        Some(TraitBound { trait_path, trait_generics, trait_id: None })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn parses_no_where_clause() {
        let src = "{";
        let mut parser = Parser::for_str(src);
        let constraints = parser.parse_where_clause();
        assert!(parser.errors.is_empty());
        assert!(constraints.is_empty());
    }

    #[test]
    fn parses_one_where_clause_with_two_constraints() {
        let src = "where Foo: Bar<T> + Baz";
        let mut parser = Parser::for_str(src);
        let mut constraints = parser.parse_where_clause();
        assert!(parser.errors.is_empty());
        assert_eq!(constraints.len(), 2);

        let constraint = constraints.remove(0);
        assert_eq!(constraint.typ.to_string(), "Foo");
        assert_eq!(constraint.trait_bound.trait_path.to_string(), "Bar");
        assert_eq!(constraint.trait_bound.trait_generics.ordered_args[0].to_string(), "T");

        let constraint = constraints.remove(0);
        assert_eq!(constraint.typ.to_string(), "Foo");
        assert_eq!(constraint.trait_bound.trait_path.to_string(), "Baz");
    }

    #[test]
    fn parses_two_where_clauses() {
        let src = "where Foo: Bar<T>, i32: Qux";
        let mut parser = Parser::for_str(src);
        let mut constraints = parser.parse_where_clause();
        assert!(parser.errors.is_empty());
        assert_eq!(constraints.len(), 2);

        let constraint = constraints.remove(0);
        assert_eq!(constraint.typ.to_string(), "Foo");
        assert_eq!(constraint.trait_bound.trait_path.to_string(), "Bar");
        assert_eq!(constraint.trait_bound.trait_generics.ordered_args[0].to_string(), "T");

        let constraint = constraints.remove(0);
        assert_eq!(constraint.typ.to_string(), "i32");
        assert_eq!(constraint.trait_bound.trait_path.to_string(), "Qux");
    }
}
