use noirc_errors::Span;

use crate::{
    ast::{Ident, Path, PathKind, UseTree, UseTreeKind},
    parser::labels::ParsingRuleLabel,
    token::{Keyword, Token},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_use_tree(&mut self) -> UseTree {
        let start_span = self.current_token_span;

        let kind = self.parse_path_kind();
        if kind != PathKind::Plain {
            self.eat_or_error(Token::DoubleColon);
        }

        let use_tree = self.parse_use_tree_without_kind(start_span, kind);
        if !self.eat_semicolons() {
            self.expected_token(Token::Semicolon);
        }
        use_tree
    }

    pub(super) fn parse_use_tree_without_kind(
        &mut self,
        start_span: Span,
        kind: PathKind,
    ) -> UseTree {
        let prefix = self.parse_path_after_kind(
            kind, false, // allow turbofish
            false, // allow trailing double colon
            start_span,
        );
        let prefix = prefix.unwrap_or_else(|| Path {
            segments: Vec::new(),
            kind: PathKind::Plain,
            span: self.span_since(start_span),
        });

        let trailing_double_colon = if prefix.segments.is_empty() && kind != PathKind::Plain {
            true
        } else {
            self.eat_double_colon()
        };

        if trailing_double_colon {
            if self.eat_left_brace() {
                let mut use_trees = Vec::new();
                let mut trailing_comma = false;
                loop {
                    let start_span = self.current_token_span;

                    let use_tree =
                        self.parse_use_tree_without_kind(self.current_token_span, PathKind::Plain);

                    // If we didn't advance at all, we are done
                    if start_span == self.current_token_span {
                        break;
                    }

                    if !trailing_comma && !use_trees.is_empty() {
                        self.expected_token_separating_items(",", "use trees", start_span);
                    }

                    use_trees.push(use_tree);

                    trailing_comma = self.eat_commas();

                    if self.eat_right_brace() {
                        break;
                    }
                }
                UseTree { prefix, kind: UseTreeKind::List(use_trees) }
            } else {
                self.expected_token(Token::LeftBrace);
                self.parse_path_use_tree_end(prefix)
            }
        } else {
            self.parse_path_use_tree_end(prefix)
        }
    }

    pub(super) fn parse_path_use_tree_end(&mut self, mut prefix: Path) -> UseTree {
        if prefix.segments.is_empty() {
            self.expected_label(ParsingRuleLabel::UseSegment);
            UseTree { prefix, kind: UseTreeKind::Path(Ident::default(), None) }
        } else {
            let ident = prefix.segments.pop().unwrap().ident;
            if self.eat_keyword(Keyword::As) {
                if let Some(alias) = self.eat_ident() {
                    UseTree { prefix, kind: UseTreeKind::Path(ident, Some(alias)) }
                } else {
                    self.expected_identifier();
                    UseTree { prefix, kind: UseTreeKind::Path(ident, None) }
                }
            } else {
                UseTree { prefix, kind: UseTreeKind::Path(ident, None) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ItemVisibility, PathKind, UseTreeKind},
        parser::{
            parser::{parse_program, tests::expect_no_errors},
            ItemKind,
        },
    };

    #[test]
    fn parse_simple() {
        let src = "use foo;";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(use_tree, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Plain);
        assert_eq!("foo", use_tree.to_string());
        let UseTreeKind::Path(ident, alias) = &use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert!(alias.is_none());
    }

    #[test]
    fn parse_simple_pub() {
        let src = "pub use foo;";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(_, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::Public);
    }

    #[test]
    fn parse_simple_pub_crate() {
        let src = "pub(crate) use foo;";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(_, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::PublicCrate);
    }

    #[test]
    fn parse_simple_with_alias() {
        let src = "use foo as bar;";
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Plain);
        assert_eq!("foo as bar", use_tree.to_string());
        let UseTreeKind::Path(ident, alias) = use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert_eq!("bar", alias.unwrap().to_string());
    }

    #[test]
    fn parse_with_crate_prefix() {
        let src = "use crate::foo;";
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Crate);
        assert_eq!("crate::foo", use_tree.to_string());
        let UseTreeKind::Path(ident, alias) = use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert!(alias.is_none());
    }

    #[test]
    fn parse_with_dep_prefix() {
        let src = "use dep::foo;";
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Dep);
        assert_eq!("dep::foo", use_tree.to_string());
        let UseTreeKind::Path(ident, alias) = use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert!(alias.is_none());
    }

    #[test]
    fn parse_with_super_prefix() {
        let src = "use super::foo;";
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Super);
        assert_eq!("super::foo", use_tree.to_string());
        let UseTreeKind::Path(ident, alias) = use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert!(alias.is_none());
    }

    #[test]
    fn parse_list() {
        let src = "use foo::{bar, baz};";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(use_tree, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Plain);
        assert_eq!("foo::{bar, baz}", use_tree.to_string());
        let UseTreeKind::List(use_trees) = &use_tree.kind else {
            panic!("Expected list");
        };
        assert_eq!(use_trees.len(), 2);
    }

    #[test]
    fn parse_list_trailing_comma() {
        let src = "use foo::{bar, baz, };";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(use_tree, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Plain);
        assert_eq!("foo::{bar, baz}", use_tree.to_string());
        let UseTreeKind::List(use_trees) = &use_tree.kind else {
            panic!("Expected list");
        };
        assert_eq!(use_trees.len(), 2);
    }

    #[test]
    fn parse_list_that_starts_with_crate() {
        let src = "use crate::{foo, bar};";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(use_tree, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::Private);
        assert_eq!("crate::{foo, bar}", use_tree.to_string());
    }
}