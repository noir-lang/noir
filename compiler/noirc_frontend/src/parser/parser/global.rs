use noirc_errors::Location;

use crate::{
    ast::{Expression, ExpressionKind, Ident, LetStatement, Pattern},
    parser::ParserErrorReason,
    token::Attribute,
};

use super::Parser;

impl Parser<'_> {
    /// Global = 'global' identifier OptionalTypeAnnotation '=' Expression ';'
    pub(crate) fn parse_global(
        &mut self,
        attributes: Vec<(Attribute, Location)>,
        comptime: bool,
        mutable: bool,
    ) -> LetStatement {
        // Only comptime globals are allowed to be mutable, but we always parse the `mut`
        // and throw the error in name resolution.

        let attributes = self.validate_secondary_attributes(attributes);
        let is_global_let = true;

        let Some(ident) = self.eat_ident() else {
            self.eat_semicolon();
            let location = self.location_at_previous_token_end();
            let ident = self.unknown_ident_at_previous_token_end();
            return LetStatement {
                pattern: ident_to_pattern(ident, mutable),
                r#type: None,
                expression: Expression { kind: ExpressionKind::Error, location },
                attributes,
                comptime,
                is_global_let,
            };
        };

        let pattern = ident_to_pattern(ident, mutable);

        let typ = self.parse_optional_type_annotation();

        let expression = if self.eat_assign() {
            self.parse_expression_or_error()
        } else {
            self.push_error(ParserErrorReason::GlobalWithoutValue, pattern.location());
            let location = self.location_at_previous_token_end();
            Expression { kind: ExpressionKind::Error, location }
        };

        self.eat_semicolon_or_error();

        LetStatement { pattern, r#type: typ, expression, attributes, comptime, is_global_let }
    }
}

fn ident_to_pattern(ident: Ident, mutable: bool) -> Pattern {
    if mutable {
        let location = ident.location();
        Pattern::Mutable(Box::new(Pattern::Identifier(ident)), location, false)
    } else {
        Pattern::Identifier(ident)
    }
}

#[cfg(test)]
mod tests {
    use acvm::FieldElement;
    use insta::assert_snapshot;

    use crate::{
        ast::{ExpressionKind, ItemVisibility, LetStatement, Literal, Pattern},
        parse_program_with_dummy_file,
        parser::{
            ItemKind, ParserErrorReason,
            parser::tests::{
                expect_no_errors, get_single_error, get_single_error_reason,
                get_source_with_error_span,
            },
        },
    };

    fn parse_global_no_errors(src: &str) -> (LetStatement, ItemVisibility) {
        let (mut module, errors) = parse_program_with_dummy_file(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Global(let_statement, visibility) = item.kind else {
            panic!("Expected global");
        };
        (let_statement, visibility)
    }

    #[test]
    fn parse_global_no_type_annotation() {
        let src = "global foo = 1;";
        let (let_statement, visibility) = parse_global_no_errors(src);
        let Pattern::Identifier(name) = &let_statement.pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!("foo", name.to_string());
        assert!(let_statement.r#type.is_none());
        assert!(!let_statement.comptime);
        assert!(let_statement.is_global_let);
        assert_eq!(visibility, ItemVisibility::Private);
    }

    #[test]
    fn parse_global_with_type_annotation() {
        let src = "global foo: i32 = 1;";
        let (let_statement, _visibility) = parse_global_no_errors(src);
        let Pattern::Identifier(name) = &let_statement.pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!("foo", name.to_string());
        assert_eq!(let_statement.r#type.unwrap().to_string(), "i32");
    }

    #[test]
    fn parse_comptime_global() {
        let src = "comptime global foo: i32 = 1;";
        let (let_statement, _visibility) = parse_global_no_errors(src);
        assert!(let_statement.comptime);
    }

    #[test]
    fn parse_mutable_global() {
        let src = "mut global foo: i32 = 1;";
        let (let_statement, _visibility) = parse_global_no_errors(src);
        let Pattern::Mutable(pattern, _, _) = &let_statement.pattern else {
            panic!("Expected mutable pattern");
        };
        let pattern: &Pattern = pattern;
        let Pattern::Identifier(name) = pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!("foo", name.to_string());
        assert_eq!(pattern.span().start(), 11);
        assert_eq!(pattern.span().end(), 14);
    }

    #[test]
    fn parse_global_no_value() {
        let src = "
        global foo;
               ^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_, errors) = parse_program_with_dummy_file(&src);
        let reason = get_single_error_reason(&errors, span);
        assert!(matches!(reason, ParserErrorReason::GlobalWithoutValue));
    }

    #[test]
    fn parse_global_no_semicolon() {
        let src = "
        global foo = 1 
                      ^ 
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_, errors) = parse_program_with_dummy_file(&src);
        let error = get_single_error(&errors, span);
        assert_snapshot!(error.to_string(), @"Expected a ';' but found end of input");
    }

    #[test]
    fn parse_negative_field_global() {
        let src = "
        global foo: Field = -17;
        ";
        let (let_statement, _visibility) = parse_global_no_errors(src);
        let Pattern::Identifier(name) = &let_statement.pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!("foo", name.to_string());
        assert_eq!(let_statement.pattern.span().start(), 16);
        assert_eq!(let_statement.pattern.span().end(), 19);

        let ExpressionKind::Literal(Literal::Integer(value, _)) = let_statement.expression.kind
        else {
            panic!("Expected integer literal expression, got {:?}", let_statement.expression.kind);
        };

        assert!(value.is_negative());
        assert_eq!(value.absolute_value(), FieldElement::from(17u128));
    }
}
