use noirc_errors::Span;

use crate::{
    ast::{
        Expression, ExpressionKind, Ident, LetStatement, Pattern, UnresolvedType,
        UnresolvedTypeData,
    },
    parser::ParserErrorReason,
    token::{Attribute, Token},
};

use super::Parser;

impl<'a> Parser<'a> {
    /// Global = 'global' identifier OptionalTypeAnnotation '=' Expression ';'
    pub(crate) fn parse_global(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        comptime: bool,
        mutable: bool,
    ) -> LetStatement {
        // Only comptime globals are allowed to be mutable, but we always parse the `mut`
        // and throw the error in name resolution.

        let attributes = self.validate_secondary_attributes(attributes);

        let Some(ident) = self.eat_ident() else {
            self.eat_semicolons();
            return LetStatement {
                pattern: ident_to_pattern(Ident::default(), mutable),
                r#type: UnresolvedType {
                    typ: UnresolvedTypeData::Unspecified,
                    span: Span::default(),
                },
                expression: Expression { kind: ExpressionKind::Error, span: Span::default() },
                attributes,
                comptime,
            };
        };

        let pattern = ident_to_pattern(ident, mutable);

        let typ = self.parse_optional_type_annotation();

        let expression = if self.eat_assign() {
            self.parse_expression_or_error()
        } else {
            self.push_error(ParserErrorReason::GlobalWithoutValue, pattern.span());
            Expression { kind: ExpressionKind::Error, span: Span::default() }
        };

        if !self.eat_semicolons() {
            self.expected_token(Token::Semicolon);
        }

        LetStatement { pattern, r#type: typ, expression, attributes, comptime }
    }
}

fn ident_to_pattern(ident: Ident, mutable: bool) -> Pattern {
    if mutable {
        let span = ident.span();
        Pattern::Mutable(Box::new(Pattern::Identifier(ident)), span, false)
    } else {
        Pattern::Identifier(ident)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            IntegerBitSize, ItemVisibility, LetStatement, Pattern, Signedness, UnresolvedTypeData,
        },
        parser::{
            parser::{
                parse_program,
                tests::{
                    expect_no_errors, get_single_error, get_single_error_reason,
                    get_source_with_error_span,
                },
            },
            ItemKind, ParserErrorReason,
        },
    };

    fn parse_global_no_errors(src: &str) -> (LetStatement, ItemVisibility) {
        let (mut module, errors) = parse_program(src);
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
        assert!(matches!(let_statement.r#type.typ, UnresolvedTypeData::Unspecified));
        assert!(!let_statement.comptime);
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
        assert!(matches!(
            let_statement.r#type.typ,
            UnresolvedTypeData::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo)
        ));
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
        let (_, errors) = parse_program(&src);
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
        let (_, errors) = parse_program(&src);
        let error = get_single_error(&errors, span);
        assert_eq!(error.to_string(), "Expected a ';' but found end of input");
    }
}
