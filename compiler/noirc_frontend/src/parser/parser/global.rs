use noirc_errors::Span;

use crate::{
    ast::{
        Expression, ExpressionKind, Ident, LetStatement, Pattern, UnresolvedType,
        UnresolvedTypeData,
    },
    parser::ParserErrorReason,
    token::Attribute,
};

use super::Parser;

impl<'a> Parser<'a> {
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

        if !self.eat_semicolon() {
            self.push_error(
                ParserErrorReason::ExpectedSemicolonAfterGlobal,
                self.current_token_span,
            );
        }

        LetStatement { pattern, r#type: typ, expression, attributes, comptime }
    }
}

fn ident_to_pattern(ident: Ident, mutable: bool) -> Pattern {
    if mutable {
        Pattern::Mutable(Box::new(Pattern::Identifier(ident)), Span::default(), false)
    } else {
        Pattern::Identifier(ident)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{IntegerBitSize, Pattern, Signedness, UnresolvedTypeData},
        parser::{
            parser::{
                parse_program,
                tests::{get_single_error, get_source_with_error_span},
            },
            ItemKind, ParserErrorReason,
        },
    };

    #[test]
    fn parse_global_no_type_annotation() {
        let src = "global foo = 1;";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Global(let_statement) = &item.kind else {
            panic!("Expected global");
        };
        let Pattern::Identifier(name) = &let_statement.pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!("foo", name.to_string());
        assert!(matches!(let_statement.r#type.typ, UnresolvedTypeData::Unspecified));
        assert!(!let_statement.comptime);
    }

    #[test]
    fn parse_global_with_type_annotation() {
        let src = "global foo: i32 = 1;";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Global(let_statement) = &item.kind else {
            panic!("Expected global");
        };
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
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Global(let_statement) = &item.kind else {
            panic!("Expected global");
        };
        assert!(let_statement.comptime);
    }

    #[test]
    fn parse_mutable_global() {
        let src = "mut global foo: i32 = 1;";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Global(let_statement) = &item.kind else {
            panic!("Expected global");
        };
        let Pattern::Mutable(pattern, _, _) = &let_statement.pattern else {
            panic!("Expected mutable pattern");
        };
        let pattern: &Pattern = pattern;
        let Pattern::Identifier(name) = pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!("foo", name.to_string());
    }

    #[test]
    fn parse_global_no_value() {
        let src = "
        global foo;
               ^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_, errors) = parse_program(&src);
        let reason = get_single_error(&errors, span);
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
        let reason = get_single_error(&errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedSemicolonAfterGlobal));
    }
}
