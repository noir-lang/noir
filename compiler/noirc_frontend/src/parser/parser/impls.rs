use noirc_errors::Span;

use crate::ast::TypeImpl;

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_impl(&mut self, start_span: Span) -> TypeImpl {
        let generics = self.parse_generics();

        let type_span_start = self.current_token_span;
        let object_type = self.parse_type();
        let type_span = self.span_since(type_span_start);

        let where_clause = self.parse_where_clause();

        // TODO: methods
        let methods = Vec::new();

        if self.eat_left_brace() {
            self.eat_right_brace();
        }

        TypeImpl { object_type, type_span, generics, where_clause, methods }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{parser::parse_program, ItemKind};

    #[test]
    fn parse_empty_impl() {
        let src = "impl Foo {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Impl(type_impl) = &item.kind else {
            panic!("Expected type impl");
        };
        assert_eq!(type_impl.object_type.to_string(), "Foo");
        assert!(type_impl.generics.is_empty());
        assert!(type_impl.methods.is_empty());
    }

    #[test]
    fn parse_empty_impl_with_generics() {
        let src = "impl <A, B> Foo {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Impl(type_impl) = &item.kind else {
            panic!("Expected type impl");
        };
        assert_eq!(type_impl.object_type.to_string(), "Foo");
        assert_eq!(type_impl.generics.len(), 2);
        assert!(type_impl.methods.is_empty());
    }
}
