use lsp_types::TextEdit;
use noirc_errors::Location;
use noirc_frontend::{
    ast::{Ident, Path},
    hir::def_map::ModuleDefId,
};

use crate::{
    byte_span_to_range,
    modules::{relative_module_full_path, relative_module_id_path},
    use_segment_positions::{
        use_completion_item_additional_text_edits, UseCompletionItemAdditionTextEditsRequest,
    },
    visibility::module_def_id_is_visible,
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

            for (module_def_id, visibility, defining_module) in entries {
                if !module_def_id_is_visible(
                    *module_def_id,
                    self.module_id,
                    *visibility,
                    *defining_module,
                    self.interner,
                    self.def_maps,
                ) {
                    continue;
                }

                let module_full_path = if let Some(defining_module) = defining_module {
                    relative_module_id_path(
                        *defining_module,
                        &self.module_id,
                        current_module_parent_id,
                        self.interner,
                    )
                } else {
                    let Some(module_full_path) = relative_module_full_path(
                        *module_def_id,
                        self.module_id,
                        current_module_parent_id,
                        self.interner,
                    ) else {
                        continue;
                    };
                    module_full_path
                };

                let full_path = if defining_module.is_some()
                    || !matches!(module_def_id, ModuleDefId::ModuleId(..))
                {
                    format!("{}::{}", module_full_path, name)
                } else {
                    module_full_path.clone()
                };

                let qualify_prefix = if let ModuleDefId::ModuleId(..) = module_def_id {
                    let mut segments: Vec<_> = module_full_path.split("::").collect();
                    segments.pop();
                    segments.join("::")
                } else {
                    module_full_path
                };

                self.push_import_code_action(&full_path);
                self.push_qualify_code_action(ident, &qualify_prefix, &full_path);
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

    fn push_qualify_code_action(&mut self, ident: &Ident, prefix: &str, full_path: &str) {
        let Some(range) = byte_span_to_range(
            self.files,
            self.file,
            ident.span().start() as usize..ident.span().start() as usize,
        ) else {
            return;
        };

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
}
