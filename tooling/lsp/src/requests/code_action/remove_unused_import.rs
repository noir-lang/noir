use async_lsp::lsp_types::TextEdit;
use fm::FileId;
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ParsedModule,
    ast::{Ident, ItemVisibility, UseTree},
    fix::use_tree_without_unused_imports,
    parser::{Item, ItemKind},
};

use crate::byte_span_to_range;

use super::CodeActionFinder;

impl CodeActionFinder<'_> {
    pub(super) fn remove_unused_import(
        &mut self,
        use_tree: &UseTree,
        visibility: ItemVisibility,
        span: Span,
    ) {
        if !self.includes_span(span) {
            return;
        }

        let Some(unused_imports) = self.usage_tracker.unused_imports().get(&self.module_id) else {
            return;
        };

        if unused_imports.is_empty() {
            return;
        }

        // The map is keyed by `(name, location)` because two distinct `use`s can import the same
        // name (into different namespaces), so the location is matched too — otherwise a used
        // import would be reported unused just for sharing a name with an unused one.
        let is_unused = |ident: &Ident| {
            unused_imports
                .keys()
                .any(|(name, location)| name == ident && *location == ident.location())
        };

        let (use_tree, removed_count) = use_tree_without_unused_imports(use_tree, &is_unused);
        if removed_count == 0 {
            return;
        }

        let byte_span = span.start() as usize..span.end() as usize;
        let Some(range) = byte_span_to_range(self.files, self.file, byte_span) else {
            return;
        };

        let (title, new_text) = match use_tree {
            Some(use_tree) => (
                if removed_count == 1 {
                    "Remove unused import".to_string()
                } else {
                    "Remove unused imports".to_string()
                },
                use_tree_to_string(use_tree, visibility, self.nesting),
            ),
            None => ("Remove the whole `use` item".to_string(), "".to_string()),
        };

        let text_edit = TextEdit { range, new_text };
        self.unused_imports_text_edits.push(text_edit.clone());

        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(code_action);
    }
}

fn use_tree_to_string(use_tree: UseTree, visibility: ItemVisibility, nesting: usize) -> String {
    // We are going to use the formatter to format the use tree
    let source = if visibility == ItemVisibility::Private {
        format!("use {use_tree};")
    } else {
        format!("{visibility} use {use_tree};")
    };
    let parsed_module = ParsedModule {
        items: vec![Item {
            kind: ItemKind::Import(use_tree, visibility),
            location: Location::new(Span::from(0..source.len() as u32), FileId::dummy()),
            doc_comments: Vec::new(),
        }],
        inner_doc_comments: Vec::new(),
    };

    // Adjust the max width according to the current nesting
    let mut config = nargo_fmt::Config::default();
    config.max_width -= nesting * 4;

    let string = nargo_fmt::format(&source, parsed_module, &config);

    let string = if nesting > 0 && string.contains('\n') {
        // If the import is nested in a module, we just formatted it without indents so we need to add them.
        let indent = " ".repeat(nesting * 4);
        string.lines().map(|line| format!("{indent}{line}")).collect::<Vec<_>>().join("\n")
    } else {
        string
    };
    string.trim().to_string()
}

#[cfg(test)]
mod tests {

    use crate::requests::code_action::tests::assert_code_action;

    #[test]
    fn test_removes_entire_unused_import_at_top_level() {
        let title = "Remove the whole `use` item";

        let src = r#"
        mod moo {
            pub fn foo() {}
        }
        use moo::fo>|<o;

        fn main() {
        }
        "#;

        let expected = r#"
        mod moo {
            pub fn foo() {}
        }
        

        fn main() {
        }
        "#;

        assert_code_action(title, src, expected);
    }

    #[test]
    fn test_removes_entire_unused_import_in_nested_module() {
        let title = "Remove the whole `use` item";

        let src = r#"
        pub(crate) mod moo {
            pub fn foo() {}
        }

        mod qux {
          use super::moo::fo>|<o;
        }

        fn main() {
        }
        "#;

        let expected = r#"
        pub(crate) mod moo {
            pub fn foo() {}
        }

        mod qux {
          
        }

        fn main() {
        }
        "#;

        assert_code_action(title, src, expected);
    }

    #[test]
    fn test_removes_single_import() {
        let title = "Remove unused import";

        let src = r#"
        mod moo {
            pub fn foo() {}
            pub fn bar() {}
        }
        use moo::{fo>|<o, bar};

        fn main() {
            bar();
        }
        "#;

        let expected = r#"
        mod moo {
            pub fn foo() {}
            pub fn bar() {}
        }
        use moo::bar;

        fn main() {
            bar();
        }
        "#;

        assert_code_action(title, src, expected);
    }

    #[test]
    fn test_removes_multiple_imports() {
        let title = "Remove unused imports";

        let src = r#"
        mod moo {
            pub fn foo() {}
            pub fn bar() {}
            pub fn baz() {}
        }
        use moo::{fo>|<o, bar, baz};

        fn main() {
            bar();
        }
        "#;

        let expected = r#"
        mod moo {
            pub fn foo() {}
            pub fn bar() {}
            pub fn baz() {}
        }
        use moo::bar;

        fn main() {
            bar();
        }
        "#;

        assert_code_action(title, src, expected);
    }

    #[test]
    fn test_removes_unused_self_import() {
        let title = "Remove unused import";

        let src = r#"
        mod moo {
            pub fn bar() {}
        }
        use moo::{se>|<lf, bar};

        fn main() {
            bar();
        }
        "#;

        let expected = r#"
        mod moo {
            pub fn bar() {}
        }
        use moo::bar;

        fn main() {
            bar();
        }
        "#;

        assert_code_action(title, src, expected);
    }

    #[test]
    fn test_removes_single_import_with_visibility() {
        let title = "Remove unused import";

        let src = r#"
        mod moo {
            pub fn foo() {}
            pub fn bar() {}
        }
        pub(crate) use moo::{fo>|<o, bar};

        fn main() {
            bar();
        }
        "#;

        let expected = r#"
        mod moo {
            pub fn foo() {}
            pub fn bar() {}
        }
        pub(crate) use moo::bar;

        fn main() {
            bar();
        }
        "#;

        assert_code_action(title, src, expected);
    }
}
