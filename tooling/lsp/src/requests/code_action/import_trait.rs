use std::collections::HashSet;

use noirc_errors::Location;
use noirc_frontend::{
    ast::MethodCallExpression,
    hir::def_map::ModuleDefId,
    node_interner::{ReferenceId, TraitId},
};

use crate::{
    modules::module_def_id_relative_path,
    requests::TraitReexport,
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

        // If the item is offered via a re-export of it's parent module, this holds the name of the reexport.
        let mut intermediate_name = None;

        if !self.module_def_id_is_visible(module_def_id, visibility, None) {
            // If it's not, try to find a visible reexport of the trait
            // that is visible from the current module
            if let Some(reexport) =
                self.interner.get_trait_reexports(trait_id).iter().find(|reexport| {
                    self.module_def_id_is_visible(
                        module_def_id,
                        reexport.visibility,
                        Some(reexport.module_id),
                    )
                })
            {
                trait_reexport = Some(TraitReexport {
                    module_id: reexport.module_id,
                    name: reexport.name.clone(),
                });
            } else if let Some(reexport) =
                self.get_ancestor_module_reexport(module_def_id, visibility)
            {
                trait_reexport = Some(TraitReexport {
                    module_id: reexport.module_id,
                    name: trait_.name.clone(),
                });
                intermediate_name = Some(reexport.name.clone());
            } else {
                return;
            }
        }

        let trait_name = if let Some(trait_reexport) = &trait_reexport {
            trait_reexport.name.clone()
        } else {
            trait_.name.clone()
        };

        // Check if the trait is currently imported. If yes, no need to suggest anything
        let module_data =
            &self.def_maps[&self.module_id.krate].modules()[self.module_id.local_id.0];
        if !module_data.scope().find_name(&trait_name).is_none() {
            return;
        }

        let module_def_id = ModuleDefId::TraitId(trait_id);
        let current_module_parent_id = self.module_id.parent(self.def_maps);
        let defining_module = trait_reexport.map(|reexport| reexport.module_id);

        let Some(full_path) = module_def_id_relative_path(
            module_def_id,
            &trait_name.0.contents,
            self.module_id,
            current_module_parent_id,
            defining_module,
            &intermediate_name,
            self.interner,
        ) else {
            return;
        };

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

    #[test]
    async fn test_import_trait_via_module_reexport() {
        let title = "Import moo::another::Foo";

        let src = r#"mod moo {
    mod nested {
        pub mod another {
            pub trait Foo {
                fn foobar(self);
            }

            impl Foo for Field {
                fn foobar(self) {}
            }
        }
    }

    pub use nested::another;
}

fn main() {
    let x: Field = 1;
    x.foo>|<bar();
}"#;

        let expected = r#"use moo::another::Foo;

mod moo {
    mod nested {
        pub mod another {
            pub trait Foo {
                fn foobar(self);
            }

            impl Foo for Field {
                fn foobar(self) {}
            }
        }
    }

    pub use nested::another;
}

fn main() {
    let x: Field = 1;
    x.foobar();
}"#;

        assert_code_action(title, src, expected).await;
    }
}
