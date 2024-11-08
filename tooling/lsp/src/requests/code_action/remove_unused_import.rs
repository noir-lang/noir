use std::collections::HashMap;

use lsp_types::TextEdit;
use noirc_errors::Span;
use noirc_frontend::{
    ast::{Ident, ItemVisibility, UseTree, UseTreeKind},
    parser::{Item, ItemKind},
    usage_tracker::UnusedItem,
    ParsedModule,
};

use crate::byte_span_to_range;

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn remove_unused_import(
        &mut self,
        use_tree: &UseTree,
        visibility: ItemVisibility,
        span: Span,
    ) {
        if !self.includes_span(span) {
            return;
        }

        let Some(unused_items) = self.usage_tracker.unused_items().get(&self.module_id) else {
            return;
        };

        if unused_items.is_empty() {
            return;
        }

        if has_unused_import(use_tree, unused_items) {
            let byte_span = span.start() as usize..span.end() as usize;
            let Some(range) = byte_span_to_range(self.files, self.file, byte_span) else {
                return;
            };

            let (use_tree, removed_count) = use_tree_without_unused_import(use_tree, unused_items);
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
}

fn has_unused_import(use_tree: &UseTree, unused_items: &HashMap<Ident, UnusedItem>) -> bool {
    match &use_tree.kind {
        UseTreeKind::Path(name, alias) => {
            let ident = alias.as_ref().unwrap_or(name);
            unused_items.contains_key(ident)
        }
        UseTreeKind::List(use_trees) => {
            use_trees.iter().any(|use_tree| has_unused_import(use_tree, unused_items))
        }
    }
}

/// Returns a new `UseTree` with all the unused imports removed, and the number of removed imports.
fn use_tree_without_unused_import(
    use_tree: &UseTree,
    unused_items: &HashMap<Ident, UnusedItem>,
) -> (Option<UseTree>, usize) {
    match &use_tree.kind {
        UseTreeKind::Path(name, alias) => {
            let ident = alias.as_ref().unwrap_or(name);
            if unused_items.contains_key(ident) {
                (None, 1)
            } else {
                (Some(use_tree.clone()), 0)
            }
        }
        UseTreeKind::List(use_trees) => {
            let mut new_use_trees: Vec<UseTree> = Vec::new();
            let mut total_count = 0;

            for use_tree in use_trees {
                let (new_use_tree, count) = use_tree_without_unused_import(use_tree, unused_items);
                if let Some(new_use_tree) = new_use_tree {
                    new_use_trees.push(new_use_tree);
                }
                total_count += count;
            }

            let new_use_tree = if new_use_trees.is_empty() {
                None
            } else if new_use_trees.len() == 1 {
                let new_use_tree = new_use_trees.remove(0);

                let mut prefix = use_tree.prefix.clone();
                prefix.segments.extend(new_use_tree.prefix.segments);

                Some(UseTree { prefix, kind: new_use_tree.kind, span: use_tree.span })
            } else {
                Some(UseTree {
                    prefix: use_tree.prefix.clone(),
                    kind: UseTreeKind::List(new_use_trees),
                    span: use_tree.span,
                })
            };

            (new_use_tree, total_count)
        }
    }
}

fn use_tree_to_string(use_tree: UseTree, visibility: ItemVisibility, nesting: usize) -> String {
    // We are going to use the formatter to format the use tree
    let source = if visibility == ItemVisibility::Private {
        format!("use {};", &use_tree)
    } else {
        format!("{} use {};", visibility, &use_tree)
    };
    let parsed_module = ParsedModule {
        items: vec![Item {
            kind: ItemKind::Import(use_tree, visibility),
            span: Span::from(0..source.len() as u32),
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
        string.lines().map(|line| format!("{}{}", indent, line)).collect::<Vec<_>>().join("\n")
    } else {
        string
    };
    string.trim().to_string()
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use crate::requests::code_action::tests::assert_code_action;

    #[test]
    async fn test_removes_entire_unused_import_at_top_level() {
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

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_removes_entire_unused_import_in_nested_module() {
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

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_removes_single_import() {
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

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_removes_multiple_imports() {
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

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_removes_single_import_with_visibility() {
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

        assert_code_action(title, src, expected).await;
    }
}
