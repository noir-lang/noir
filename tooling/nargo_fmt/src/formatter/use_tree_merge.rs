use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display},
};

use noirc_frontend::ast::{ItemVisibility, PathKind, UseTree, UseTreeKind};

use crate::chunks::{ChunkGroup, TextChunk};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn merge_and_format_imports(
        &mut self,
        imports: Vec<UseTree>,
        visibility: ItemVisibility,
    ) {
        let merged_import = merge_imports(imports);

        for (index, (segment, segment_tree)) in merged_import.tree.into_iter().enumerate() {
            if index > 0 {
                self.write_line_without_skipping_whitespace_and_comments();
            }

            let tree = ImportTree::single(segment, *segment_tree);
            let tree = tree.simplify();

            let group = format_merged_import_with_visibility(tree, visibility);
            self.write_indentation();
            self.format_chunk_group(group);
        }
    }
}

// The logic here is similar to that of `use_tree.rs`, except that it works with ImportTree
// instead of UseTree and we never check or advance the lexer.

fn format_merged_import_with_visibility(
    mut import_tree: ImportTree,
    visibility: ItemVisibility,
) -> ChunkGroup {
    let mut group = ChunkGroup::new();
    match visibility {
        ItemVisibility::Private => (),
        ItemVisibility::PublicCrate => {
            group.text(TextChunk::new("pub(crate) ".to_string()));
        }
        ItemVisibility::Public => {
            group.text(TextChunk::new("pub ".to_string()));
        }
    }
    group.text(TextChunk::new("use ".to_string()));

    let (segment, tree) = import_tree.tree.pop_first().unwrap();
    assert!(import_tree.tree.is_empty());

    group.group(format_merged_import(segment, *tree));
    group.text_attached_to_last_group(TextChunk::new(";".to_string()));
    group
}

fn format_merged_import(segment: Segment, import_tree: ImportTree) -> ChunkGroup {
    let mut group = ChunkGroup::new();

    group.text(TextChunk::new(segment.to_string()));

    if import_tree.tree.is_empty() {
        return group;
    }

    // We check if there are nested lists. If yes, then each item will be on a separate line
    // (it reads better, and this is what rustfmt seems to do too)
    if import_tree.tree.values().all(|tree| tree.tree.is_empty()) {
        group.one_chunk_per_line = false;
    }

    group.text(TextChunk::new("::{".to_string()));
    group.increase_indentation();
    group.line();

    for (index, (segment, import_tree)) in import_tree.tree.into_iter().enumerate() {
        if index > 0 {
            group.text_attached_to_last_group(TextChunk::new(",".to_string()));
            group.space_or_line();
        }

        group.group(format_merged_import(segment, *import_tree));
    }

    group.trailing_comma();
    group.decrease_indentation();
    group.line();
    group.text(TextChunk::new("}".to_string()));

    group
}

/// We keep Crate, Super and Dep as special segments so that they are ordered in that way
/// (they'll come in that order before any plain segment).
#[derive(Debug, PartialEq, Eq)]
enum Segment {
    Crate,
    Super,
    Dep,
    Plain(String),
    Terminal,
}

impl Segment {
    /// Combines two segments into a single one, by joining them with "::".
    fn combine(self, other: Segment) -> Segment {
        if other == Segment::Terminal {
            self
        } else {
            Segment::Plain(format!("{}::{}", self, other))
        }
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        match self {
            Segment::Crate => write!(f, "crate"),
            Segment::Super => write!(f, "super"),
            Segment::Dep => write!(f, "dep"),
            Segment::Plain(s) => write!(f, "{}", s),
            Segment::Terminal => write!(f, "self"),
        }
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Segment::Crate, Segment::Crate) => Ordering::Equal,
            (Segment::Crate, _) => Ordering::Less,
            (Segment::Super, Segment::Crate) => Ordering::Greater,
            (Segment::Super, Segment::Super) => Ordering::Equal,
            (Segment::Super, _) => Ordering::Less,
            (Segment::Dep, Segment::Crate) | (Segment::Dep, Segment::Super) => Ordering::Less,
            (Segment::Dep, Segment::Dep) => Ordering::Equal,
            (Segment::Dep, _) => Ordering::Greater,
            (Segment::Plain(self_string), Segment::Plain(other_string)) => {
                // Case-insensitive comparison for plain segments
                self_string.to_lowercase().cmp(&other_string.to_lowercase())
            }
            (Segment::Plain(_), Segment::Terminal) => Ordering::Less,
            (Segment::Plain(_), _) => Ordering::Greater,
            (Segment::Terminal, _) => Ordering::Greater,
        }
    }
}

/// An import tree to represent merged imports.
/// For example for the given imports:
///
/// use foo::bar::{baz, qux};
/// use foo::another;
///
/// an ImportTree that represents the merged imports would be:
///
/// {
///     "foo" => {
///         "another" => {}
///         "bar" => {"baz", "qux"},
///     }
/// }
#[derive(Debug, Default)]
struct ImportTree {
    tree: BTreeMap<Segment, Box<ImportTree>>,
}

impl ImportTree {
    fn new() -> Self {
        Self { tree: BTreeMap::new() }
    }

    /// Creates an import tree that has `segment` as the only element with `tree` as its value.
    fn single(segment: Segment, tree: ImportTree) -> Self {
        let mut tree_map = BTreeMap::new();
        tree_map.insert(segment, Box::new(tree));
        Self { tree: tree_map }
    }

    /// Inserts a segment to the tree, creating the necessary empty children if they don't exist yet.
    fn insert(&mut self, segment: Segment) -> &mut ImportTree {
        self.tree.entry(segment).or_default()
    }

    /// Inserts a segment that's the final segment in an import path.
    fn insert_terminal(&mut self, segment: Segment) {
        let tree = self.insert(segment);
        tree.insert(Segment::Terminal);
    }

    /// Simplifies a tree by combining segments that only have one child.
    ///
    /// For example, this tree:
    ///
    /// {
    ///     "foo" => {
    ///         "bar" => {"baz", "qux"}
    ///     }
    /// }
    ///
    /// will be simplified to:
    ///
    /// {
    ///     "foo::bar" => {"baz", "qux"}
    /// }
    fn simplify(self) -> ImportTree {
        let mut new_tree = ImportTree::new();
        for (segment, tree) in self.tree.into_iter() {
            let mut tree = tree.simplify();
            if tree.tree.len() == 1 {
                let (first_segment, first_tree) = tree.tree.pop_first().unwrap();
                let new_segment = segment.combine(first_segment);
                new_tree.tree.insert(new_segment, first_tree);
            } else {
                new_tree.tree.insert(segment, Box::new(tree));
            }
        }
        new_tree
    }
}

/// Combines all use trees to form a single ImportTree.
fn merge_imports(imports: Vec<UseTree>) -> ImportTree {
    let mut tree = ImportTree::new();
    merge_imports_in_tree(imports, &mut tree);
    tree
}

fn merge_imports_in_tree(imports: Vec<UseTree>, mut tree: &mut ImportTree) {
    for import in imports {
        let mut tree = match import.prefix.kind {
            PathKind::Crate => tree.insert(Segment::Crate),
            PathKind::Super => tree.insert(Segment::Super),
            PathKind::Dep => tree.insert(Segment::Dep),
            PathKind::Plain => &mut tree,
        };

        for segment in import.prefix.segments {
            tree = tree.insert(Segment::Plain(segment.ident.to_string()));
        }

        match import.kind {
            UseTreeKind::Path(ident, alias) => {
                if let Some(alias) = alias {
                    tree.insert_terminal(Segment::Plain(format!("{} as {}", ident, alias)));
                } else {
                    tree.insert_terminal(Segment::Plain(ident.to_string()));
                }
            }
            UseTreeKind::List(trees) => {
                merge_imports_in_tree(trees, tree);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_max_width};

    #[test]
    fn format_simple_use_without_alias() {
        let src = " mod moo {  pub  use  foo ;  }";
        let expected = "mod moo {
    pub use foo;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_simple_use_with_alias() {
        let src = " mod moo {  use  foo :: bar   as   baz ;  }";
        let expected = "mod moo {
    use foo::bar as baz;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_simple_use_with_path_kind() {
        let src = "use  super :: foo ;";
        let expected = "use super::foo;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_use_list_two_items() {
        let src = " use foo::{ bar,  baz  };";
        let expected = "use foo::{bar, baz};\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_use_trees_with_max_width() {
        let src = " use foo::{ bar,  baz , qux , one::{two, three} };";
        let expected = "use foo::{
    bar,
    baz,
    one::{
        three, two,
    },
    qux,
};
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_use_list_one_item() {
        let src = " use foo::{  bar,  };";
        let expected = "use foo::bar;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_long_use_list_one_item() {
        let src = "use one::two::{three::{four, five}};";
        let expected = "use one::two::three::{
    five, four,
};
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_use_list_one_item_with_comments() {
        let src = " use foo::{  /* do not remove me */ bar,  };";
        let expected = "use foo::{ /* do not remove me */ bar};\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_use_crate_with_list() {
        let src = " use crate :: hash :: { Hash, Hasher };  ";
        let expected = "use crate::hash::{Hash, Hasher};\n";
        assert_format(src, expected);
    }

    #[test]
    fn attaches_semicolon_to_last_group() {
        let src = " use crate::hash::{Hash, Hasher};  ";
        let expected = "use crate::hash::{
    Hash, Hasher,
};
";
        assert_format_with_max_width(src, expected, "use crate::hash::{Hash, Hasher}".len());
    }

    #[test]
    fn does_not_merge_imports_if_they_are_separated_by_two_lines() {
        let src = "
            use foo::baz;

            use foo::{def, abc};
";
        let expected = "use foo::baz;

use foo::{abc, def};
";
        assert_format(src, expected);
    }

    #[test]
    fn does_not_merge_imports_if_they_have_different_visibilities() {
        let src = "
            pub use foo::baz;
            use foo::{def, abc};
";
        let expected = "pub use foo::baz;
use foo::{abc, def};
";
        assert_format(src, expected);
    }

    #[test]
    fn does_not_merge_imports_if_they_have_trailing_comments_on_the_first_use() {
        let src = "
            use foo; // trailing
            use bar;

            fn foo() {}
";
        let expected = "use foo; // trailing
use bar;

fn foo() {}
";
        assert_format(src, expected);
    }

    #[test]
    fn does_not_merge_imports_if_they_have_trailing_comments_followed_by_item() {
        let src = "
            use foo;
            use bar; // trailing

            fn foo() {}
";
        let expected = "use foo;
use bar; // trailing

fn foo() {}
";
        assert_format(src, expected);
    }

    #[test]
    fn does_not_merge_imports_if_they_have_trailing_comments_followed_by_nothing() {
        let src = "
            use foo;
            use bar; // trailing
";
        let expected = "use foo;
use bar; // trailing
";
        assert_format(src, expected);
    }

    #[test]
    fn merges_and_sorts_imports_just_two() {
        let src = "
            use foo::baz;
            use foo::bar;
        ";
        let expected = "use foo::{bar, baz};\n";
        assert_format(src, expected);
    }

    #[test]
    fn merges_and_sorts_imports_2() {
        let src = "
use aztec::{
    context::Context, log::emit_unencrypted_log, note::{
        note_getter_options::NoteGetterOptions, note_header::NoteHeader,
    }, state_vars::{
        Map, PrivateSet, PublicMutable,
    }, types::{
        address::AztecAddress, type_serialization::field_serialization::{
            FIELD_SERIALIZED_LEN, FieldSerializationMethods,
        },
    },
};
        ";
        let expected = "use aztec::{
    context::Context,
    log::emit_unencrypted_log,
    note::{note_getter_options::NoteGetterOptions, note_header::NoteHeader},
    state_vars::{Map, PrivateSet, PublicMutable},
    types::{
        address::AztecAddress,
        type_serialization::field_serialization::{FIELD_SERIALIZED_LEN, FieldSerializationMethods},
    },
};
";
        assert_format(src, expected);
    }

    #[test]
    fn merges_and_sorts_imports_with_different_path_kinds() {
        let src = "
            use bar::baz;
            use foo::bar;
            use crate::foo;
        ";
        let expected = "use crate::foo;
use bar::baz;
use foo::bar;
";
        assert_format(src, expected);
    }

    #[test]
    fn sorts_import() {
        let src = "
        use value_note::{
            utils::{increment, decrement}, value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
        };
        ";
        let expected = "use value_note::{
    utils::{decrement, increment},
    value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
};
";
        assert_format(src, expected);
    }

    #[test]
    fn sorts_import_ignoring_case() {
        let src = "
        use foo::{def, ZETA, ABC, efg};
        use BAR;
        ";
        let expected = "use BAR;
use foo::{ABC, def, efg, ZETA};
";
        assert_format(src, expected);
    }

    #[test]
    fn merges_nested_import() {
        let src = "
        use foo::bar;
        use foo;
        ";
        let expected = "use foo::{bar, self};\n";
        assert_format(src, expected);
    }

    #[test]
    fn idempotent_test_check_next_test() {
        let src = "
        use std::as_witness;
use std::merkle::compute_merkle_root;
        ";
        let expected = "use std::{as_witness, merkle::compute_merkle_root};\n";
        assert_format(src, expected);
    }

    #[test]
    fn idempotent_test_check_previous_test() {
        let src = "use std::{as_witness, merkle::compute_merkle_root};";
        let expected = "use std::{as_witness, merkle::compute_merkle_root};\n";
        assert_format(src, expected);
    }
}
