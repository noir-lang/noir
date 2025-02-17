use lsp_types::TextEdit;
use noirc_errors::Location;
use noirc_frontend::ast::{Ident, Path};

use crate::{
    byte_span_to_range,
    modules::module_def_id_relative_path,
    use_segment_positions::{
        use_completion_item_additional_text_edits, UseCompletionItemAdditionTextEditsRequest,
    },
};

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn import_or_qualify(&mut self, path: &Path) {
        if path.segments.len() != 1 {
            return;
        }

        let ident = &path.segments[0].ident;
        if !self.includes_span(ident.span()) {
            return;
        }

        let location = Location::new(ident.span(), self.file);
        if self.interner.find_referenced(location).is_some() {
            return;
        }

        let current_module_parent_id = self.module_id.parent(self.def_maps);

        // The Path doesn't resolve to anything so it means it's an error and maybe we
        // can suggest an import or to fully-qualify the path.
        for (name, entries) in self.interner.get_auto_import_names() {
            if name != &ident.0.contents {
                continue;
            }

            for entry in entries {
                let module_def_id = entry.module_def_id;
                let visibility = entry.visibility;
                let mut defining_module = entry.defining_module.as_ref().cloned();

                // If the item is offered via a re-export of it's parent module, this holds the name of the reexport.
                let mut intermediate_name = None;

                let is_visible =
                    self.module_def_id_is_visible(module_def_id, visibility, defining_module);
                if !is_visible {
                    if let Some(reexport) = self.get_parent_module_reexport(module_def_id) {
                        defining_module = Some(reexport.module_id);
                        intermediate_name = Some(reexport.name.clone());
                    } else {
                        continue;
                    }
                }

                let Some(full_path) = module_def_id_relative_path(
                    module_def_id,
                    name,
                    self.module_id,
                    current_module_parent_id,
                    defining_module,
                    &intermediate_name,
                    self.interner,
                ) else {
                    continue;
                };

                self.push_import_code_action(&full_path);
                self.push_qualify_code_action(ident, &full_path);
            }
        }
    }

    fn push_import_code_action(&mut self, full_path: &str) {
        let title = format!("Import {}", full_path);

        let text_edits = use_completion_item_additional_text_edits(
            UseCompletionItemAdditionTextEditsRequest {
                full_path,
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

    fn push_qualify_code_action(&mut self, ident: &Ident, full_path: &str) {
        let Some(range) = byte_span_to_range(
            self.files,
            self.file,
            ident.span().start() as usize..ident.span().start() as usize,
        ) else {
            return;
        };

        let mut prefix = full_path.split("::").collect::<Vec<_>>();
        prefix.pop();
        let prefix = prefix.join("::");

        let title = format!("Qualify as {}", full_path);
        let text_edit = TextEdit { range, new_text: format!("{}::", prefix) };

        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(code_action);
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use crate::requests::code_action::tests::assert_code_action;

    #[test]
    async fn test_qualify_code_action_for_struct() {
        let title = "Qualify as foo::bar::SomeTypeInBar";

        let src = r#"
        mod foo {
            pub mod bar {
                pub struct SomeTypeInBar {}
            }
        }

        fn foo(x: SomeType>|<InBar) {}
        "#;

        let expected = r#"
        mod foo {
            pub mod bar {
                pub struct SomeTypeInBar {}
            }
        }

        fn foo(x: foo::bar::SomeTypeInBar) {}
        "#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_import_code_action_for_struct() {
        let title = "Import foo::bar::SomeTypeInBar";

        let src = r#"mod foo {
    pub mod bar {
        pub struct SomeTypeInBar {}
    }
}

fn foo(x: SomeType>|<InBar) {}"#;

        let expected = r#"use foo::bar::SomeTypeInBar;

mod foo {
    pub mod bar {
        pub struct SomeTypeInBar {}
    }
}

fn foo(x: SomeTypeInBar) {}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_import_code_action_for_struct_at_beginning_of_name() {
        let title = "Import foo::bar::SomeTypeInBar";

        let src = r#"mod foo {
    pub mod bar {
        pub struct SomeTypeInBar {}
    }
}

fn foo(x: >|<SomeTypeInBar) {}"#;

        let expected = r#"use foo::bar::SomeTypeInBar;

mod foo {
    pub mod bar {
        pub struct SomeTypeInBar {}
    }
}

fn foo(x: SomeTypeInBar) {}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_qualify_code_action_for_module() {
        let title = "Qualify as foo::bar::some_module_in_bar";

        let src = r#"
        mod foo {
            pub mod bar {
                pub mod some_module_in_bar {}
            }
        }

        fn main() {
          some_mod>|<ule_in_bar
        }
        "#;

        let expected = r#"
        mod foo {
            pub mod bar {
                pub mod some_module_in_bar {}
            }
        }

        fn main() {
          foo::bar::some_module_in_bar
        }
        "#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_import_code_action_for_module() {
        let title = "Import foo::bar::some_module_in_bar";

        let src = r#"mod foo {
    pub mod bar {
        pub(crate) mod some_module_in_bar {}
    }
}

fn main() {
    some_mod>|<ule_in_bar
}"#;

        let expected = r#"use foo::bar::some_module_in_bar;

mod foo {
    pub mod bar {
        pub(crate) mod some_module_in_bar {}
    }
}

fn main() {
    some_module_in_bar
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_import_code_action_for_struct_inserts_into_existing_use() {
        let title = "Import foo::bar::SomeTypeInBar";

        let src = r#"use foo::bar::SomeOtherType;

mod foo {
    pub mod bar {
        pub struct SomeTypeInBar {}
    }
}

fn foo(x: SomeType>|<InBar) {}"#;

        let expected = r#"use foo::bar::{SomeOtherType, SomeTypeInBar};

mod foo {
    pub mod bar {
        pub struct SomeTypeInBar {}
    }
}

fn foo(x: SomeTypeInBar) {}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_import_via_reexport() {
        let title = "Import aztec::protocol_types::SomeStruct";

        let src = r#"mod aztec {
    mod deps {
        pub mod protocol_types {
            pub struct SomeStruct {}
        }
    }

    pub use deps::protocol_types;
}

fn main() {
    SomeStr>|<uct
}"#;

        let expected = r#"use aztec::protocol_types::SomeStruct;

mod aztec {
    mod deps {
        pub mod protocol_types {
            pub struct SomeStruct {}
        }
    }

    pub use deps::protocol_types;
}

fn main() {
    SomeStruct
}"#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_qualify_via_reexport() {
        let title = "Qualify as aztec::protocol_types::SomeStruct";

        let src = r#"mod aztec {
    mod deps {
        pub mod protocol_types {
            pub struct SomeStruct {}
        }
    }

    pub use deps::protocol_types;
}

fn main() {
    SomeStr>|<uct
}"#;

        let expected = r#"mod aztec {
    mod deps {
        pub mod protocol_types {
            pub struct SomeStruct {}
        }
    }

    pub use deps::protocol_types;
}

fn main() {
    aztec::protocol_types::SomeStruct
}"#;

        assert_code_action(title, src, expected).await;
    }
}
