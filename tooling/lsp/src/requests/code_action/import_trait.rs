use std::collections::HashSet;

use noirc_errors::Location;
use noirc_frontend::{
    ast::MethodCallExpression,
    hir::def_map::ModuleDefId,
    node_interner::{ReferenceId, TraitId},
};

use crate::{
    modules::{relative_module_full_path, relative_module_id_path},
    requests::TraitReexport,
    use_segment_positions::{
        use_completion_item_additional_text_edits, UseCompletionItemAdditionTextEditsRequest,
    },
    visibility::module_def_id_is_visible,
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
            self.import_trait(trait_id);
            return;
        }

        // Find out the type of the object
        let object_location = Location::new(method_call.object.span, self.file);
        let Some(typ) = self.interner.type_at_location(object_location) else {
            return;
        };

        let trait_methods =
            self.interner.lookup_trait_methods(&typ, &method_call.method_name.0.contents, true);
        let trait_ids: HashSet<_> = trait_methods.iter().map(|(_, trait_id)| *trait_id).collect();

        for trait_id in trait_ids {
            self.import_trait(trait_id);
        }
    }

    fn import_trait(&mut self, trait_id: TraitId) {
        // First check if the trait is visible
        let trait_ = self.interner.get_trait(trait_id);
        let visibility = trait_.visibility;
        let module_def_id = ModuleDefId::TraitId(trait_id);
        let mut trait_reexport = None;

        if !module_def_id_is_visible(
            module_def_id,
            self.module_id,
            visibility,
            None,
            self.interner,
            self.def_maps,
        ) {
            // If it's not, try to find a visible reexport of the trait
            // that is visible from the current module
            let Some((visible_module_id, name, _)) =
                self.interner.get_trait_reexports(trait_id).iter().find(
                    |(module_id, _, visibility)| {
                        module_def_id_is_visible(
                            module_def_id,
                            self.module_id,
                            *visibility,
                            Some(*module_id),
                            self.interner,
                            self.def_maps,
                        )
                    },
                )
            else {
                return;
            };
            trait_reexport = Some(TraitReexport { module_id: visible_module_id, name });
        }

        let trait_name = if let Some(trait_reexport) = &trait_reexport {
            trait_reexport.name
        } else {
            &trait_.name
        };

        // Check if the trait is currently imported. If yes, no need to suggest anything
        let module_data =
            &self.def_maps[&self.module_id.krate].modules()[self.module_id.local_id.0];
        if !module_data.scope().find_name(trait_name).is_none() {
            return;
        }

        let module_def_id = ModuleDefId::TraitId(trait_id);
        let current_module_parent_id = self.module_id.parent(self.def_maps);
        let module_full_path = if let Some(trait_reexport) = &trait_reexport {
            relative_module_id_path(
                *trait_reexport.module_id,
                &self.module_id,
                current_module_parent_id,
                self.interner,
            )
        } else {
            let Some(path) = relative_module_full_path(
                module_def_id,
                self.module_id,
                current_module_parent_id,
                self.interner,
            ) else {
                return;
            };
            path
        };

        let full_path = format!("{}::{}", module_full_path, trait_name);

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

    #[test]
    async fn test_import_trait_in_method_call_when_one_option_but_not_in_scope_via_reexport() {
        let title = "Import moo::Bar";

        let src = r#"mod moo {
    mod nested {
        pub trait Foo {
            fn foobar(self);
        }

        impl Foo for Field {
            fn foobar(self) {}
        }
    }

    pub use nested::Foo as Bar;
}

fn main() {
    let x: Field = 1;
    x.foo>|<bar();
}"#;

        let expected = r#"use moo::Bar;

mod moo {
    mod nested {
        pub trait Foo {
            fn foobar(self);
        }

        impl Foo for Field {
            fn foobar(self) {}
        }
    }

    pub use nested::Foo as Bar;
}

fn main() {
    let x: Field = 1;
    x.foobar();
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_import_trait_in_method_call_when_multiple_via_reexport() {
        let title = "Import moo::Baz";

        let src = r#"mod moo {
    mod nested {
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

    pub use nested::Foo as Baz;
    pub use nested::Foo as Qux;
}

fn main() {
    let x: Field = 1;
    x.foo>|<bar();
}"#;

        let expected = r#"use moo::Baz;

mod moo {
    mod nested {
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

    pub use nested::Foo as Baz;
    pub use nested::Foo as Qux;
}

fn main() {
    let x: Field = 1;
    x.foobar();
}"#;

        assert_code_action(title, src, expected).await;
    }
}
