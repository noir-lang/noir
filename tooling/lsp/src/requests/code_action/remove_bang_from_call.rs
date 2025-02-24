use lsp_types::TextEdit;
use noirc_errors::{Location, Span};
use noirc_frontend::{node_interner::ReferenceId, QuotedType, Type};

use crate::byte_span_to_range;

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn remove_bang_from_call(&mut self, span: Span) {
        // If we can't find the referenced function, there's nothing we can do
        let Some(ReferenceId::Function(func_id)) =
            self.interner.find_referenced(Location::new(span, self.file))
        else {
            return;
        };

        // If the return type is Quoted, all is good
        let func_meta = self.interner.function_meta(&func_id);
        if let Type::Quoted(QuotedType::Quoted) = func_meta.return_type() {
            return;
        }

        // The `!` comes right after the name
        let byte_span = span.end() as usize..span.end() as usize + 1;
        let Some(range) = byte_span_to_range(self.files, self.file, byte_span) else {
            return;
        };

        let title = "Remove `!` from call".to_string();
        let text_edit = TextEdit { range, new_text: "".to_string() };

        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(code_action);
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use crate::requests::code_action::tests::assert_code_action;

    #[test]
    async fn test_removes_bang_from_call() {
        let title = "Remove `!` from call";

        let src = r#"
        fn foo() {}

        fn main() {
            fo>|<o!();
        }
        "#;

        let expected = r#"
        fn foo() {}

        fn main() {
            foo();
        }
        "#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_removes_bang_from_method_call() {
        let title = "Remove `!` from call";

        let src = r#"
        struct Foo {}

        impl Foo {
          fn foo(self) {}
        }

        fn bar(foo: Foo) {
            foo.fo>|<o!();
        }
        "#;

        let expected = r#"
        struct Foo {}

        impl Foo {
          fn foo(self) {}
        }

        fn bar(foo: Foo) {
            foo.foo();
        }
        "#;

        assert_code_action(title, src, expected).await;
    }
}
