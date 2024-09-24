use noirc_errors::Span;

use crate::{
    ast::{Ident, Path, PathKind, PathSegment, UseTree, UseTreeKind},
    token::{Keyword, Token},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_use_tree(&mut self) -> UseTree {
        let start_span = self.current_token_span;

        let kind = self.parse_path_kind();
        if kind != PathKind::Plain {
            if self.eat(Token::DoubleColon).is_none() {
                // TODO: error
            }
        }

        let use_tree = self.parse_use_tree_without_kind(start_span, kind);
        if !self.eat_semicolons() {
            // TODO: error
        }
        use_tree
    }

    pub(super) fn parse_use_tree_without_kind(
        &mut self,
        start_span: Span,
        kind: PathKind,
    ) -> UseTree {
        let mut segments = Vec::new();
        let mut trailing_double_colon = false;

        while let Some(ident) = self.eat_ident() {
            let span = ident.span();
            segments.push(PathSegment { ident, generics: None, span });
            if self.eat(Token::DoubleColon).is_some() {
                trailing_double_colon = true;
            } else {
                trailing_double_colon = false;
                break;
            }
        }

        let span = if segments.is_empty() { start_span } else { self.span_since(start_span) };
        let prefix = Path { segments, kind, span };

        if trailing_double_colon {
            if self.eat(Token::LeftBrace).is_some() {
                let mut use_trees = Vec::new();
                loop {
                    let current_span = self.current_token_span;

                    let use_tree =
                        self.parse_use_tree_without_kind(self.current_token_span, PathKind::Plain);

                    // If we didn't advance at all, we are done
                    if current_span == self.current_token_span {
                        break;
                    }

                    use_trees.push(use_tree);

                    self.eat_commas();

                    if self.eat(Token::RightBrace).is_some() {
                        break;
                    }
                }
                UseTree { prefix, kind: UseTreeKind::List(use_trees) }
            } else {
                // TODO: error
                self.parse_path_use_tree_end(prefix)
            }
        } else {
            self.parse_path_use_tree_end(prefix)
        }
    }

    pub(super) fn parse_path_use_tree_end(&mut self, mut prefix: Path) -> UseTree {
        if prefix.segments.is_empty() {
            // TODO: error
            UseTree {
                prefix,
                kind: UseTreeKind::Path(Ident::new(String::new(), Span::default()), None),
            }
        } else {
            let ident = prefix.segments.pop().unwrap().ident;
            if self.eat_keyword(Keyword::As) {
                if let Some(alias) = self.eat_ident() {
                    UseTree { prefix, kind: UseTreeKind::Path(ident, Some(alias)) }
                } else {
                    // TODO: error
                    UseTree { prefix, kind: UseTreeKind::Path(ident, None) }
                }
            } else {
                UseTree { prefix, kind: UseTreeKind::Path(ident, None) }
            }
        }
    }

    pub(super) fn parse_path_kind(&mut self) -> PathKind {
        if self.eat_keyword(Keyword::Crate) {
            PathKind::Crate
        } else if self.eat_keyword(Keyword::Dep) {
            PathKind::Dep
        } else if self.eat_keyword(Keyword::Super) {
            PathKind::Super
        } else {
            PathKind::Plain
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ItemVisibility, PathKind, UseTreeKind},
        parse_program,
        parser::ItemKind,
    };

    #[test]
    fn parse_simple() {
        let src = "use foo;";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(use_tree, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Plain);
        assert_eq!("", use_tree.prefix.to_string());
        let UseTreeKind::Path(ident, alias) = &use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert!(alias.is_none());
    }

    #[test]
    fn parse_simple_with_alias() {
        let src = "use foo as bar;";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Plain);
        assert_eq!("", use_tree.prefix.to_string());
        let UseTreeKind::Path(ident, alias) = use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert_eq!("bar", alias.unwrap().to_string());
    }

    #[test]
    fn parse_with_crate_prefix() {
        let src = "use crate::foo";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Crate);
        assert_eq!("crate::", use_tree.prefix.to_string());
        let UseTreeKind::Path(ident, alias) = use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert!(alias.is_none());
    }

    #[test]
    fn parse_with_dep_prefix() {
        let src = "use dep::foo";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Dep);
        assert_eq!("dep::", use_tree.prefix.to_string());
        let UseTreeKind::Path(ident, alias) = use_tree.kind else {
            panic!("Expected path");
        };
        assert_eq!("foo", ident.to_string());
        assert!(alias.is_none());
    }

    #[test]
    fn parse_with_super_prefix() {
        let src = "use super::foo";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Super);
        assert_eq!("super::", use_tree.prefix.to_string());
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
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(use_tree, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Plain);
        assert_eq!("foo", use_tree.prefix.to_string());
        let UseTreeKind::List(use_trees) = &use_tree.kind else {
            panic!("Expected list");
        };
        assert_eq!(use_trees.len(), 2);
    }

    #[test]
    fn parse_list_trailing_comma() {
        let src = "use foo::{bar, baz, };";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Import(use_tree, visibility) = &item.kind else {
            panic!("Expected import");
        };
        assert_eq!(visibility, &ItemVisibility::Private);
        assert_eq!(use_tree.prefix.kind, PathKind::Plain);
        assert_eq!("foo", use_tree.prefix.to_string());
        let UseTreeKind::List(use_trees) = &use_tree.kind else {
            panic!("Expected list");
        };
        assert_eq!(use_trees.len(), 2);
    }
}
