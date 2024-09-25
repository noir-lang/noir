use std::collections::HashMap;

use lsp_types::TextEdit;
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{NoirTraitImpl, TraitImplItemKind, UnresolvedTypeData},
    node_interner::ReferenceId,
};

use crate::{byte_span_to_range, trait_impl_method_stub_generator::TraitImplMethodStubGenerator};

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn implement_missing_members(
        &mut self,
        noir_trait_impl: &NoirTraitImpl,
        span: Span,
    ) {
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

        // Also get all associated types
        let mut associated_types = HashMap::new();
        for associated_type in &trait_.associated_types {
            associated_types.insert(associated_type.name.as_ref(), associated_type);
        }

        // Remove the ones that already are implemented
        for item in &noir_trait_impl.items {
            match &item.item.kind {
                TraitImplItemKind::Function(noir_function) => {
                    method_ids.remove(noir_function.name());
                }
                TraitImplItemKind::Constant(..) => (),
                TraitImplItemKind::Type { name, alias } => {
                    if let UnresolvedTypeData::Unspecified = alias.typ {
                        continue;
                    }
                    associated_types.remove(&name.0.contents);
                }
            }
        }

        // Also remove default methods
        for trait_function in &trait_.methods {
            if trait_function.default_impl.is_some() {
                method_ids.remove(&trait_function.name.0.contents);
            }
        }

        if method_ids.is_empty() && associated_types.is_empty() {
            return;
        }

        let bytes = self.source.as_bytes();
        let right_brace_index = span.end() as usize - 1;

        // Let's find out the indent
        let mut cursor = right_brace_index - 1;
        while cursor > 0 {
            let c = bytes[cursor] as char;
            if c == '\n' {
                break;
            }
            if !c.is_whitespace() {
                break;
            }
            cursor -= 1;
        }
        let cursor_char = bytes[cursor] as char;

        let indent = if cursor_char == '\n' { right_brace_index - cursor - 1 } else { 0 };
        let indent_string = " ".repeat(indent + 4);

        let index = cursor + 1;

        let Some(range) = byte_span_to_range(self.files, self.file, index..index) else {
            return;
        };

        let mut method_ids: Vec<_> = method_ids.iter().collect();
        method_ids.sort_by_key(|(name, _)| *name);

        let mut stubs = Vec::new();

        for (name, _) in associated_types {
            stubs.push(format!("{}type {};\n", indent_string, name));
        }

        for (name, func_id) in method_ids {
            let func_meta = self.interner.function_meta(func_id);
            let modifiers = self.interner.function_modifiers(func_id);

            let mut generator = TraitImplMethodStubGenerator::new(
                name,
                func_meta,
                modifiers,
                trait_,
                noir_trait_impl,
                self.interner,
                self.def_maps,
                self.module_id,
                indent + 4,
            );
            generator.set_body(format!("panic(f\"Implement {}\")", name));

            let stub = generator.generate();
            stubs.push(stub);
        }

        let mut new_text = stubs.join("\n");
        if cursor_char != '\n' {
            new_text.insert(0, '\n');
        }

        let title = "Implement missing members".to_string();
        let text_edit = TextEdit { range, new_text };
        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(code_action);
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
    fn bar() {}
}

struct Foo {}

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
trait Trait {
    fn foo(x: i32) -> i32;
    fn bar() {}
}

struct Foo {}

impl Trait for Foo {
    fn foo(x: i32) -> i32 {
        panic(f"Implement foo")
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
        panic(f"Implement bar")
    }

    fn foo(x: i32) -> i32 {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_qualify_type() {
        let title = "Implement missing members";

        let src = r#"
mod moo {
    struct Moo {}

    trait Trait {
        fn foo(x: Moo);
    }
}

struct Foo {}

use moo::Trait;

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
mod moo {
    struct Moo {}

    trait Trait {
        fn foo(x: Moo);
    }
}

struct Foo {}

use moo::Trait;

impl Trait for Foo {
    fn foo(x: moo::Moo) {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_no_need_to_qualify_type() {
        let title = "Implement missing members";

        let src = r#"
mod moo {
    struct Moo {}

    trait Trait {
        fn foo(x: Moo);
    }
}

struct Foo {}

use moo::Trait;
use moo::Moo;

impl Tra>|<it for Foo {
}"#;

        let expected = r#"
mod moo {
    struct Moo {}

    trait Trait {
        fn foo(x: Moo);
    }
}

struct Foo {}

use moo::Trait;
use moo::Moo;

impl Trait for Foo {
    fn foo(x: Moo) {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_generics() {
        let title = "Implement missing members";

        let src = r#"
trait Bar {}

trait Trait<T> {
    fn foo<let N: u32, M>(x: T) -> [T; N] where M: Bar;
}

struct Foo {}

impl <U> Tra>|<it<[U]> for Foo {
}"#;

        let expected = r#"
trait Bar {}

trait Trait<T> {
    fn foo<let N: u32, M>(x: T) -> [T; N] where M: Bar;
}

struct Foo {}

impl <U> Trait<[U]> for Foo {
    fn foo<let N: u32, M>(x: [U]) -> [[U]; N] where M: Bar {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_associated_types() {
        let title = "Implement missing members";

        let src = r#"
trait Trait {
    type Elem;

    fn foo(x: Self::Elem) -> [Self::Elem];
}

struct Foo {}

impl Trait>|< for Foo {
}"#;

        let expected = r#"
trait Trait {
    type Elem;

    fn foo(x: Self::Elem) -> [Self::Elem];
}

struct Foo {}

impl Trait for Foo {
    type Elem;

    fn foo(x: Self::Elem) -> [Self::Elem] {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_nested() {
        let title = "Implement missing members";

        let src = r#"
mod moo {
    trait Trait {
        fn foo();
        fn bar();
    }

    struct Foo {}

    impl Tra>|<it for Foo {
    }
}"#;

        let expected = r#"
mod moo {
    trait Trait {
        fn foo();
        fn bar();
    }

    struct Foo {}

    impl Trait for Foo {
        fn bar() {
            panic(f"Implement bar")
        }

        fn foo() {
            panic(f"Implement foo")
        }
    }
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_add_missing_impl_members_inline() {
        let title = "Implement missing members";

        let src = r#"
trait Trait {
    fn foo();
    fn bar();
}

struct Foo {}

impl Tra>|<it for Foo {}"#;

        let expected = r#"
trait Trait {
    fn foo();
    fn bar();
}

struct Foo {}

impl Trait for Foo {
    fn bar() {
        panic(f"Implement bar")
    }

    fn foo() {
        panic(f"Implement foo")
    }
}"#;

        assert_code_action(title, src, expected).await;
    }
}
