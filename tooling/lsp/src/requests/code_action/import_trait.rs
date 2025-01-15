use noirc_errors::Location;
use noirc_frontend::{
    ast::MethodCallExpression,
    hir::{def_map::ModuleDefId, resolution::visibility::trait_member_is_visible},
    hir_def::traits::Trait,
    node_interner::ReferenceId,
};

use crate::{
    modules::relative_module_full_path,
    use_segment_positions::{
        use_completion_item_additional_text_edits, UseCompletionItemAdditionTextEditsRequest,
    },
};

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn import_trait_in_method_call(&mut self, method_call: &MethodCallExpression) {
        // First see if the method name already points to a function.
        let name_location = Location::new(method_call.method_name.span(), self.file);
        if let Some(ReferenceId::Function(func_id)) = self.interner.find_referenced(name_location) {
            // If yes, it could be that the compiler is issuing a warning because there's
            // only one possible trait that the method could be coming from, but it's not imported
            let func_meta = self.interner.function_meta(&func_id);
            let Some(trait_impl_id) = func_meta.trait_impl else {
                return;
            };

            let trait_impl = self.interner.get_trait_implementation(trait_impl_id);
            let trait_id = trait_impl.borrow().trait_id;
            let trait_ = self.interner.get_trait(trait_id);

            // Check if the trait is currently imported. If so, no need to suggest anything
            let module_data =
                &self.def_maps[&self.module_id.krate].modules()[self.module_id.local_id.0];
            if !module_data.scope().find_name(&trait_.name).is_none() {
                return;
            }

            self.push_import_trait_code_action(trait_);
            return;
        }

        // Find out the type of the object
        let object_location = Location::new(method_call.object.span, self.file);
        let Some(typ) = self.interner.type_at_location(object_location) else {
            return;
        };

        for (func_id, trait_id) in
            self.interner.lookup_trait_methods(&typ, &method_call.method_name.0.contents, true)
        {
            let visibility = self.interner.function_modifiers(&func_id).visibility;
            if !trait_member_is_visible(trait_id, visibility, self.module_id, self.def_maps) {
                continue;
            }

            let trait_ = self.interner.get_trait(trait_id);
            self.push_import_trait_code_action(trait_);
        }
    }

    fn push_import_trait_code_action(&mut self, trait_: &Trait) {
        let trait_id = trait_.id;

        let module_def_id = ModuleDefId::TraitId(trait_id);
        let current_module_parent_id = self.module_id.parent(self.def_maps);
        let Some(module_full_path) = relative_module_full_path(
            module_def_id,
            self.module_id,
            current_module_parent_id,
            self.interner,
        ) else {
            return;
        };
        let full_path = format!("{}::{}", module_full_path, trait_.name);

        let title = format!("Import {}", full_path);

        let text_edits = use_completion_item_additional_text_edits(
            UseCompletionItemAdditionTextEditsRequest {
                full_path: &full_path,
                files: self.files,
                file: self.file,
                lines: &self.lines,
                nesting: self.nesting,
                auto_import_line: self.auto_import_line,
            },
            &self.use_segment_positions,
        );

        let code_action = self.new_quick_fix_multiple_edits(title, text_edits);
        self.code_actions.push(code_action);
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use crate::requests::code_action::tests::assert_code_action;

    #[test]
    async fn test_import_trait_in_method_call_when_one_option_but_not_in_scope() {
        let title = "Import moo::Foo";

        let src = r#"mod moo {
    pub trait Foo {
        fn foobar(self);
    }

    impl Foo for Field {
        fn foobar(self) {}
    }
}

fn main() {
    let x: Field = 1;
    x.foo>|<bar();
}"#;

        let expected = r#"use moo::Foo;

mod moo {
    pub trait Foo {
        fn foobar(self);
    }

    impl Foo for Field {
        fn foobar(self) {}
    }
}

fn main() {
    let x: Field = 1;
    x.foobar();
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_import_trait_in_method_call_when_multiple_options_1() {
        let title = "Import moo::Foo";

        let src = r#"mod moo {
    pub trait Foo {
        fn foobar(self);
    }

    impl Foo for Field {
        fn foobar(self) {}
    }

    pub trait Bar {
        fn foobar(self);
    }

    impl Bar for Field {
        fn foobar(self) {}
    }
}

fn main() {
    let x: Field = 1;
    x.foo>|<bar();
}"#;

        let expected = r#"use moo::Foo;

mod moo {
    pub trait Foo {
        fn foobar(self);
    }

    impl Foo for Field {
        fn foobar(self) {}
    }

    pub trait Bar {
        fn foobar(self);
    }

    impl Bar for Field {
        fn foobar(self) {}
    }
}

fn main() {
    let x: Field = 1;
    x.foobar();
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_import_trait_in_method_call_when_multiple_options_2() {
        let title = "Import moo::Bar";

        let src = r#"mod moo {
    pub trait Foo {
        fn foobar(self);
    }

    impl Foo for Field {
        fn foobar(self) {}
    }

    pub trait Bar {
        fn foobar(self);
    }

    impl Bar for Field {
        fn foobar(self) {}
    }
}

fn main() {
    let x: Field = 1;
    x.foo>|<bar();
}"#;

        let expected = r#"use moo::Bar;

mod moo {
    pub trait Foo {
        fn foobar(self);
    }

    impl Foo for Field {
        fn foobar(self) {}
    }

    pub trait Bar {
        fn foobar(self);
    }

    impl Bar for Field {
        fn foobar(self) {}
    }
}

fn main() {
    let x: Field = 1;
    x.foobar();
}"#;

        assert_code_action(title, src, expected).await;
    }
}
