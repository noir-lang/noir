use noirc_frontend::{
    ast::{ItemVisibility, UseTree, UseTreeKind},
    token::{Keyword, Token},
};

use crate::chunks::{Chunk, ChunkFormatter, ChunkGroup};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_import(&mut self, use_tree: UseTree, visibility: ItemVisibility) {
        let group = self.chunk_formatter().format_import(use_tree, visibility);

        self.write_indentation();
        self.format_chunk_group(group);
    }
}

impl<'a, 'b> ChunkFormatter<'a, 'b> {
    pub(super) fn format_import(
        &mut self,
        use_tree: UseTree,
        visibility: ItemVisibility,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        group.text(self.chunk(|formatter| {
            formatter.format_item_visibility(visibility);
            formatter.write_keyword(Keyword::Use);
            formatter.write_space();
        }));

        group.group(self.format_use_tree(use_tree));
        group.semicolon(self);

        group
    }

    fn format_use_tree(&mut self, use_tree: UseTree) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.one_chunk_per_line = false;

        if !use_tree.prefix.is_empty() {
            group.text(self.chunk(|formatter| {
                let has_segments = !use_tree.prefix.segments.is_empty();

                formatter.format_path(use_tree.prefix);

                // If the path has segments, like in `foo` or `crate::foo`, we need to add a double colon.
                // But for example for `crate::` we don't need to add a double colon (there are no segments
                // and `crate::` was already written).
                if has_segments {
                    formatter.write_token(Token::DoubleColon);
                }
            }));
        }

        match use_tree.kind {
            UseTreeKind::Path(name, alias) => {
                group.text(self.chunk(|formatter| {
                    formatter.write_identifier(name);
                    if let Some(alias) = alias {
                        formatter.write_space();
                        formatter.write_keyword(Keyword::As);
                        formatter.write_space();
                        formatter.write_identifier(alias);
                    }
                }));
            }
            UseTreeKind::List(use_trees) => {
                // We check if there are nested lists. If yes, then each item will be on a separate line
                // (it reads better, and this is what rustfmt seems to do too)
                let has_nested_list =
                    use_trees.iter().any(|use_tree| matches!(use_tree.kind, UseTreeKind::List(..)));

                let use_trees_len = use_trees.len();

                let left_brace_chunk = self.chunk(|formatter| {
                    formatter.write_left_brace();
                });

                let comments_count_before = self.written_comments_count;

                let mut items_chunk = ChunkGroup::new();
                self.format_items_separated_by_comma(
                    use_trees,
                    false, // force trailing comma
                    false, // surround with spaces
                    &mut items_chunk,
                    |formatter, use_tree, chunks| chunks.group(formatter.format_use_tree(use_tree)),
                );

                let wrote_comment = self.written_comments_count > comments_count_before;

                let right_brace_chunk = self.chunk(|formatter| {
                    formatter.write_right_brace();
                });

                if use_trees_len == 1 && !wrote_comment {
                    // We are only interested in keeping the single Group: everything else
                    // is lines, indentation and trailing comma that we don't need and would
                    // actually produce incorrect code.
                    let single_group =
                        items_chunk.chunks.into_iter().filter_map(Chunk::group).next().unwrap();
                    group.chunks.extend(single_group.chunks);
                } else {
                    if has_nested_list {
                        group.one_chunk_per_line = true;
                    }

                    group.text(left_brace_chunk);
                    group.chunks.extend(items_chunk.chunks);
                    group.text(right_brace_chunk);
                }
            }
        }

        group
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format_with_config, config::ImportsGranularity, Config};

    fn assert_format(src: &str, expected: &str) {
        let config = Config {
            imports_granularity: ImportsGranularity::Preserve,
            reorder_imports: false,
            ..Config::default()
        };
        assert_format_with_config(src, expected, config);
    }

    fn assert_format_with_max_width(src: &str, expected: &str, max_width: usize) {
        let config = Config {
            imports_granularity: ImportsGranularity::Preserve,
            reorder_imports: false,
            max_width,
            ..Config::default()
        };
        assert_format_with_config(src, expected, config);
    }

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
    qux,
    one::{
        two, three,
    },
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
    four, five,
};
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_use_list_one_item_with_comments() {
        let src = " use foo::{  /* do not remove me */ bar,  };";
        let expected = "use foo::{/* do not remove me */ bar};\n";
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
    fn do_not_merge_and_sort_imports() {
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
}
