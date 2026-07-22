//! Tests for [crate::remove_unused_imports], which rewrites source code to prune imports
//! that the elaborator determined are unused.

use std::collections::HashSet;

use noirc_errors::Location;

use crate::ast::Ident;
use crate::remove_unused_imports::remove_unused_imports;
use crate::test_utils::get_program;

/// Compiles `src`, collects every unused import from the usage tracker, and returns `src`
/// rewritten with those imports removed. Returns `None` if there was nothing to remove.
fn source_without_unused_imports(src: &str) -> Option<String> {
    let (parsed_module, context, _errors) = get_program(src);
    let unused_imports: HashSet<(Ident, Location)> = context
        .usage_tracker
        .unused_imports()
        .values()
        .flat_map(|imports| imports.keys().cloned())
        .collect();
    remove_unused_imports(src, &parsed_module, &unused_imports)
}

#[test]
fn removes_unused_imports_from_composite_bracketed_use() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
    pub mod baz {
        pub fn qux() {}
        pub fn corge() {}
    }
}

use foo::{bar, spam, baz::{qux, corge}};

fn main() {
    bar();
    qux();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
        pub mod baz {
            pub fn qux() {}
            pub fn corge() {}
        }
    }

    use foo::{bar, baz::qux};

    fn main() {
        bar();
        qux();
    }
    ");
}

#[test]
fn collapses_bracketed_list_with_single_remaining_import() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
}

use foo::{bar, spam};

fn main() {
    bar();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
    }

    use foo::bar;

    fn main() {
        bar();
    }
    ");
}

#[test]
fn collapses_nested_bracketed_lists() {
    let src = r#"mod foo {
    pub mod bar {
        pub fn qux() {}
        pub fn corge() {}
    }
}

use foo::{bar::{qux, corge}};

fn main() {
    corge();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub mod bar {
            pub fn qux() {}
            pub fn corge() {}
        }
    }

    use foo::bar::corge;

    fn main() {
        corge();
    }
    ");
}

#[test]
fn removes_entire_use_item_when_all_imports_unused() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
}

use foo::{bar, spam};

fn main() {}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
    }

    fn main() {}
    ");
}

#[test]
fn removes_unused_aliased_import() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
}

use foo::{bar as b, spam as s};

fn main() {
    b();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
    }

    use foo::bar as b;

    fn main() {
        b();
    }
    ");
}

#[test]
fn removes_unused_self_import_from_list() {
    let src = r#"mod foo {
    pub fn bar() {}
}

use foo::{self, bar};

fn main() {
    bar();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
    }

    use foo::bar;

    fn main() {
        bar();
    }
    ");
}

#[test]
fn removes_unused_imports_in_nested_modules_in_one_go() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
}

mod qux {
    use super::foo::{bar, spam};

    pub fn corge() {
        bar();
    }
}

use foo::spam;

fn main() {
    qux::corge();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
    }

    mod qux {
        use super::foo::bar;

        pub fn corge() {
            bar();
        }
    }

    fn main() {
        qux::corge();
    }
    ");
}

#[test]
fn removes_adjacent_fully_unused_use_items_without_leaving_blank_lines() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
}

use foo::bar;
use foo::spam;

fn main() {}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
    }

    fn main() {}
    ");
}

#[test]
fn collapsing_to_self_import_drops_the_self_segment() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
}

use foo::{self, spam};

fn main() {
    foo::bar();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
    }

    use foo;

    fn main() {
        foo::bar();
    }
    ");
}

#[test]
fn rewrites_multi_line_use_item() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
    pub fn qux() {}
}

use foo::{
    bar,
    spam,
    qux,
};

fn main() {
    bar();
    qux();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
        pub fn qux() {}
    }

    use foo::{bar, qux};

    fn main() {
        bar();
        qux();
    }
    ");
}

/// Removal is a single round of a fixpoint, not the fixpoint itself. Resolving an import's
/// path marks its first segment as used, so an import whose only consumer is another `use`
/// statement is *not* reported unused until that other `use` has been removed and the
/// program re-elaborated. Each round therefore removes one level of such a cascade.
#[test]
fn import_used_only_by_a_removed_import_is_only_removed_on_a_second_run() {
    let src = r#"mod foo {
    pub mod bar {
        pub fn qux() {}
    }
}

use foo::bar;
use bar::qux;

fn main() {}
"#;
    // The first round only removes `use bar::qux;`: at this point `use foo::bar;` is
    // considered used, because resolving the path `bar::qux` referenced it.
    let after_first_run =
        source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(after_first_run, @r"
    mod foo {
        pub mod bar {
            pub fn qux() {}
        }
    }

    use foo::bar;

    fn main() {}
    ");

    // Re-elaborating the pruned source reveals that `use foo::bar;` is now unused too.
    let after_second_run =
        source_without_unused_imports(&after_first_run).expect("expected imports to be removed");
    insta::assert_snapshot!(after_second_run, @r"
    mod foo {
        pub mod bar {
            pub fn qux() {}
        }
    }

    fn main() {}
    ");

    // The fixpoint is reached: a third round has nothing left to remove.
    assert_eq!(source_without_unused_imports(&after_second_run), None);
}

#[test]
fn returns_none_when_there_are_no_unused_imports() {
    let src = r#"mod foo {
    pub fn bar() {}
}

use foo::bar;

fn main() {
    bar();
}
"#;
    assert_eq!(source_without_unused_imports(src), None);
}

#[test]
fn preserves_visibility_when_rewriting_use_item() {
    let src = r#"mod foo {
    pub fn bar() {}
    pub fn spam() {}
}

mod qux {
    pub(crate) use super::foo::{bar, spam};

    pub fn corge() {
        spam();
    }
}

fn main() {
    qux::corge();
}
"#;
    let result = source_without_unused_imports(src).expect("expected imports to be removed");
    insta::assert_snapshot!(result, @r"
    mod foo {
        pub fn bar() {}
        pub fn spam() {}
    }

    mod qux {
        pub(crate) use super::foo::spam;

        pub fn corge() {
            spam();
        }
    }

    fn main() {
        qux::corge();
    }
    ");
}
