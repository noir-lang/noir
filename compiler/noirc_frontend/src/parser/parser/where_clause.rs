use crate::{
    ast::{TraitBound, UnresolvedTraitConstraint},
    parser::ParserErrorReason,
    token::Keyword,
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
            let typ = self.parse_type();
            if self.current_token_span == start_span {
                break;
            }

            if !trailing_comma && !constraints.is_empty() {
                self.push_error(ParserErrorReason::MissingCommaSeparatingTraitBounds, start_span);
            }

            if self.eat_colon() {
                let trait_bounds = self.parse_trait_bounds();
                for trait_bound in trait_bounds {
                    constraints.push(UnresolvedTraitConstraint { typ: typ.clone(), trait_bound });
                }
            } else {
                // TODO: error
            }

            trailing_comma = self.eat_commas();
        }

        if constraints.is_empty() {
            // TODO: error
        }

        constraints
    }

    pub(super) fn parse_trait_bounds(&mut self) -> Vec<TraitBound> {
        let mut bounds = Vec::new();

        let mut trailing_plus = false;
        loop {
            let start_span = self.current_token_span;
            let bound = self.parse_trait_bound();
            if self.current_token_span == start_span {
                break;
            }

            if !trailing_plus && !bounds.is_empty() {
                self.push_error(ParserErrorReason::MissingPlusSeparatingTraitBounds, start_span);
            }

            bounds.push(bound);

            trailing_plus = self.eat_plus();
        }

        bounds
    }

    pub(crate) fn parse_trait_bound(&mut self) -> TraitBound {
        let trait_path = self.parse_path_no_turbofish();
        let trait_generics = self.parse_generic_type_args();
        TraitBound { trait_path, trait_generics, trait_id: None }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn parses_no_where_clause() {
        let src = "{";
        let constraints = Parser::for_str(src).parse_where_clause();
        assert!(constraints.is_empty());
    }

    #[test]
    fn parses_one_where_clause_with_two_constraints() {
        let src = "where Foo: Bar<T> + Baz";
        let mut constraints = Parser::for_str(src).parse_where_clause();
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
        let mut constraints = Parser::for_str(src).parse_where_clause();
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
