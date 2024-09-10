use lsp_types::TextEdit;
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{NoirTraitImpl, TraitImplItem},
    hir_def::{function::FuncMeta, stmt::HirPattern},
    macros_api::NodeInterner,
    node_interner::ReferenceId,
    Type,
};

use crate::byte_span_to_range;

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn add_missing_impl_members(&mut self, noir_trait_impl: &NoirTraitImpl, span: Span) {
        if !self.includes_span(span) {
            return;
        }

        let location = Location::new(noir_trait_impl.trait_name.span(), self.file);
        let Some(ReferenceId::Trait(trait_id)) = self.interner.find_referenced(location) else {
            return;
        };

        let trait_ = self.interner.get_trait(trait_id);

        // Get all methods
        let mut method_ids = trait_.method_ids.clone();

        // Remove the ones that already are implemented
        for item in &noir_trait_impl.items {
            if let TraitImplItem::Function(noir_function) = &item.item {
                method_ids.remove(noir_function.name());
            }
        }

        if method_ids.is_empty() {
            return;
        }

        // let bytes = self.source.as_bytes();
        let right_brace_index = span.end() as usize - 1;
        let index = right_brace_index;

        let Some(range) = byte_span_to_range(self.files, self.file, index..index) else {
            return;
        };

        let mut method_ids: Vec<_> = method_ids.iter().collect();
        method_ids.sort_by_key(|(name, _)| *name);

        let method_stubs: Vec<_> = method_ids
            .iter()
            .map(|(name, func_id)| {
                let func_meta = self.interner.function_meta(&func_id);
                method_stub(&name, func_meta, self.interner)
            })
            .collect();

        let new_text = method_stubs.join("\n");

        let title = "Implement missing members".to_string();
        let text_edit = TextEdit { range, new_text };
        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(code_action);
    }
}

fn method_stub(name: &str, func_meta: &FuncMeta, interner: &NodeInterner) -> String {
    let mut string = String::new();
    let indent = "    ";

    string.push_str(indent);
    string.push_str("fn ");
    string.push_str(name);
    string.push('(');
    for (index, (pattern, typ, _visibility)) in func_meta.parameters.iter().enumerate() {
        if index > 0 {
            string.push_str(", ");
        }
        if append_pattern(pattern, &mut string, interner) {
            string.push_str(": ");
            string.push_str(&typ.to_string());
        }
    }
    string.push(')');

    let return_type = func_meta.return_type();
    if return_type != &Type::Unit {
        string.push_str(" -> ");
        string.push_str(&return_type.to_string());
    }

    string.push_str(" {\n");
    string.push_str(indent);
    string.push_str(indent);
    string.push_str("panic(f\"Implement ");
    string.push_str(name);
    string.push_str("\");\n");
    string.push_str(indent);
    string.push_str("}\n");
    string
}

/// Appends a pattern and returns true if this was not the self type
fn append_pattern(pattern: &HirPattern, string: &mut String, interner: &NodeInterner) -> bool {
    match pattern {
        HirPattern::Identifier(hir_ident) => {
            let definition = interner.definition(hir_ident.id);
            string.push_str(&definition.name);
            &definition.name != "self"
        }
        HirPattern::Mutable(pattern, _) => {
            string.push_str("mut ");
            append_pattern(pattern, string, interner)
        }
        HirPattern::Tuple(patterns, _) => {
            string.push('(');
            for (index, pattern) in patterns.iter().enumerate() {
                if index > 0 {
                    string.push_str(", ");
                }
                append_pattern(pattern, string, interner);
            }
            string.push(')');
            true
        }
        HirPattern::Struct(_, _, _) => {
            // TODO
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use crate::requests::code_action::tests::assert_code_action;

    #[test]
    async fn test_add_missing_impl_members_simple() {
        let title = "Implement missing members";

        let src = r#"
trait Trait {
    fn foo(x: i32) -> i32;
}

struct Foo {}

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
trait Trait {
    fn foo(x: i32) -> i32;
}

struct Foo {}

impl Trait for Foo {
    fn foo(x: i32) -> i32 {
        panic(f"Implement foo");
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_multiple_with_self_type() {
        let title = "Implement missing members";

        let src = r#"
trait Trait {
    fn bar(self) -> Self;
    fn foo(x: i32) -> i32;
}

struct Foo {}

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
trait Trait {
    fn bar(self) -> Self;
    fn foo(x: i32) -> i32;
}

struct Foo {}

impl Trait for Foo {
    fn bar(self) -> Self {
        panic(f"Implement bar");
    }

    fn foo(x: i32) -> i32 {
        panic(f"Implement foo");
    }
}"#;

        assert_code_action(title, src, expected).await;
    }
}
