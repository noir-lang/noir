use noirc_frontend::{
    ast::{ItemVisibility, PathKind, UseTree, UseTreeKind},
    token::{Keyword, Token},
};

use super::{chunks::Chunks, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn format_import(&mut self, use_tree: UseTree, visibility: ItemVisibility) {
        let mut chunks = Chunks::new();

        chunks.text(self.chunk(|formatter| {
            formatter.format_item_visibility(visibility);
            formatter.write_keyword(Keyword::Use);
            formatter.write_space();
        }));

        chunks.group(self.format_use_tree(use_tree));

        chunks.text(self.chunk(|formatter| {
            formatter.write_token(Token::Semicolon);
        }));

        self.write_indentation();
        self.format_chunks(chunks);
        self.write_line();
    }

    fn format_use_tree(&mut self, use_tree: UseTree) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.one_chunk_per_line = false;

        if !use_tree.prefix.is_empty() {
            chunks.text(self.chunk(|formatter| {
                let kind = use_tree.prefix.kind;

                formatter.format_path(use_tree.prefix);

                if kind == PathKind::Plain {
                    formatter.write_token(Token::DoubleColon);
                }
            }));
        }

        match use_tree.kind {
            UseTreeKind::Path(name, alias) => {
                chunks.text(self.chunk(|formatter| {
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
                let use_trees_len = use_trees.len();

                let left_brace_chunk = self.chunk(|formatter| {
                    formatter.write_left_brace();
                });

                self.wrote_comment = false;

                let mut items_chunk = Chunks::new();
                self.format_items_separated_by_comma(
                    use_trees,
                    false, // force trailing comma
                    false, // surround with spaces
                    &mut items_chunk,
                    |formatter, use_tree, chunks| chunks.group(formatter.format_use_tree(use_tree)),
                );

                let wrote_comment = self.wrote_comment;

                let right_brace_chunk = self.chunk(|formatter| {
                    formatter.write_right_brace();
                });

                if use_trees_len == 1 && !wrote_comment {
                    chunks.chunks.extend(items_chunk.chunks);
                } else {
                    chunks.text(left_brace_chunk);
                    chunks.chunks.extend(items_chunk.chunks);
                    chunks.text(right_brace_chunk);
                }
            }
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_max_width};

    #[test]
    fn format_simple_use() {
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
    bar, baz, qux,
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

    // TODO: ideally the space after `*/` is preserved too
    #[test]
    fn format_use_list_one_item_with_comments() {
        let src = " use foo::{  /* do not remove me */ bar,  };";
        let expected = "use foo::{ /* do not remove me */bar};\n";
        assert_format(src, expected);
    }
}
