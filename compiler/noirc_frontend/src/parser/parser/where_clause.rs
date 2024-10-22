use crate::{
    ast::{GenericTypeArgs, Path, PathKind, TraitBound, UnresolvedTraitConstraint, UnresolvedType},
    parser::labels::ParsingRuleLabel,
    token::{Keyword, Token},
};

use super::{
    parse_many::{separated_by, separated_by_comma},
    Parser,
};

impl<'a> Parser<'a> {
    /// WhereClause = 'where' WhereClauseItems?
    ///
    /// WhereClauseItems = WhereClauseItem ( ',' WhereClauseItem )* ','?
    ///
    /// WhereClauseItem = Type ':' TraitBounds
    pub(super) fn parse_where_clause(&mut self) -> Vec<UnresolvedTraitConstraint> {
        if !self.eat_keyword(Keyword::Where) {
            return Vec::new();
        }

        // Constraints might end up being empty, but that's accepted as valid syntax
        let constraints =
            self.parse_many("where clauses", separated_by_comma(), Self::parse_single_where_clause);

        constraints
            .into_iter()
            .flat_map(|(typ, trait_bounds)| {
                trait_bounds.into_iter().map(move |trait_bound| UnresolvedTraitConstraint {
                    typ: typ.clone(),
                    trait_bound,
                })
            })
            .collect()
    }

    fn parse_single_where_clause(&mut self) -> Option<(UnresolvedType, Vec<TraitBound>)> {
        let Some(typ) = self.parse_type() else {
            return None;
        };

        self.eat_or_error(Token::Colon);

        let trait_bounds = self.parse_trait_bounds();

        Some((typ, trait_bounds))
    }

    /// TraitBounds = TraitBound ( '+' TraitBound )? '+'?
    pub(super) fn parse_trait_bounds(&mut self) -> Vec<TraitBound> {
        self.parse_many(
            "trait bounds",
            separated_by(Token::Plus).stop_if_separator_is_missing(),
            Self::parse_trait_bound_in_list,
        )
    }

    fn parse_trait_bound_in_list(&mut self) -> Option<TraitBound> {
        if let Some(trait_bound) = self.parse_trait_bound() {
            Some(trait_bound)
        } else {
            self.expected_label(ParsingRuleLabel::TraitBound);
            None
        }
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

    /// TraitBound = PathNoTurbofish GenericTypeArgs
    pub(crate) fn parse_trait_bound(&mut self) -> Option<TraitBound> {
        let trait_path = self.parse_path_no_turbofish()?;
        let trait_generics = self.parse_generic_type_args();
        Some(TraitBound { trait_path, trait_generics, trait_id: None })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::UnresolvedTraitConstraint,
        parser::{
            parser::tests::{
                expect_no_errors, get_single_error_reason, get_source_with_error_span,
            },
            Parser, ParserErrorReason,
        },
        token::Token,
    };

    fn parse_where_clause_no_errors(src: &str) -> Vec<UnresolvedTraitConstraint> {
        let mut parser = Parser::for_str(src);
        let constraints = parser.parse_where_clause();
        expect_no_errors(&parser.errors);
        constraints
    }

    #[test]
    fn parses_no_where_clause() {
        let src = "{";
        let constraints = parse_where_clause_no_errors(src);
        assert!(constraints.is_empty());
    }

    #[test]
    fn parses_one_where_clause_with_two_constraints() {
        let src = "where Foo: Bar<T> + Baz";
        let mut constraints = parse_where_clause_no_errors(src);
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
        let src = "where Foo: Bar<T>, i32: Qux {";
        let mut constraints = parse_where_clause_no_errors(src);
        assert_eq!(constraints.len(), 2);

        let constraint = constraints.remove(0);
        assert_eq!(constraint.typ.to_string(), "Foo");
        assert_eq!(constraint.trait_bound.trait_path.to_string(), "Bar");
        assert_eq!(constraint.trait_bound.trait_generics.ordered_args[0].to_string(), "T");

        let constraint = constraints.remove(0);
        assert_eq!(constraint.typ.to_string(), "i32");
        assert_eq!(constraint.trait_bound.trait_path.to_string(), "Qux");
    }

    #[test]
    fn parses_two_where_clauses_missing_comma() {
        let src = "
        where Foo: Bar<T> i32: Qux {
                          ^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let mut constraints = parser.parse_where_clause();

        let reason = get_single_error_reason(&parser.errors, span);
        let ParserErrorReason::ExpectedTokenSeparatingTwoItems { token, items } = reason else {
            panic!("Expected a different error");
        };
        assert_eq!(token, &Token::Comma);
        assert_eq!(*items, "where clauses");

        assert_eq!(constraints.len(), 2);

        let constraint = constraints.remove(0);
        assert_eq!(constraint.typ.to_string(), "Foo");
        assert_eq!(constraint.trait_bound.trait_path.to_string(), "Bar");
        assert_eq!(constraint.trait_bound.trait_generics.ordered_args[0].to_string(), "T");

        let constraint = constraints.remove(0);
        assert_eq!(constraint.typ.to_string(), "i32");
        assert_eq!(constraint.trait_bound.trait_path.to_string(), "Qux");
    }

    #[test]
    fn parses_where_clause_missing_trait_bound() {
        let src = "where Foo: ";
        let mut parser = Parser::for_str(src);
        parser.parse_where_clause();
        assert!(!parser.errors.is_empty());
    }
}
