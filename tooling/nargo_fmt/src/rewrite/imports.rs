use noirc_frontend::{PathKind, UseTreeKind};

use crate::{
    items::Item,
    visitor::{
        expr::{format_exprs, Tactic},
        FmtVisitor, Shape,
    },
};

#[derive(Debug)]
pub(crate) enum UseSegment {
    Ident(String, Option<String>),
    List(Vec<UseTree>),
    Dep,
    Crate,
}

impl UseSegment {
    fn rewrite(&self, visitor: &FmtVisitor, shape: Shape) -> String {
        match self {
            UseSegment::Ident(ident, None) => ident.clone(),
            UseSegment::Ident(ident, Some(rename)) => format!("{ident} as {rename}"),
            UseSegment::List(use_tree_list) => {
                let mut nested_shape = shape;
                nested_shape.indent.block_indent(visitor.config);

                let items: Vec<_> = use_tree_list
                    .iter()
                    .map(|item| Item {
                        leading: String::new(),
                        value: item.rewrite(visitor, shape).clone(),
                        trailing: String::new(),
                        different_line: false,
                    })
                    .collect();

                let list_str =
                    format_exprs(visitor.config, Tactic::Mixed, false, items, nested_shape, true);

                if list_str.contains('\n') {
                    format!(
                        "{{\n{}{list_str}\n{}}}",
                        nested_shape.indent.to_string(),
                        shape.indent.to_string()
                    )
                } else {
                    format!("{{{list_str}}}")
                }
            }
            UseSegment::Dep => "dep".into(),
            UseSegment::Crate => "crate".into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct UseTree {
    path: Vec<UseSegment>,
}

impl UseTree {
    pub(crate) fn from_ast(use_tree: noirc_frontend::UseTree) -> Self {
        let mut result = UseTree { path: vec![] };

        match use_tree.prefix.kind {
            PathKind::Crate => result.path.push(UseSegment::Crate),
            PathKind::Dep => result.path.push(UseSegment::Dep),
            PathKind::Plain => {}
        };

        result.path.extend(
            use_tree
                .prefix
                .segments
                .into_iter()
                .map(|segment| UseSegment::Ident(segment.to_string(), None)),
        );

        match use_tree.kind {
            UseTreeKind::Path(name, alias) => {
                result.path.push(UseSegment::Ident(
                    name.to_string(),
                    alias.map(|rename| rename.to_string()),
                ));
            }
            UseTreeKind::List(list) => {
                let segment = UseSegment::List(list.into_iter().map(UseTree::from_ast).collect());
                result.path.push(segment);
            }
        }

        result
    }

    pub(crate) fn rewrite_top_level(&self, visitor: &FmtVisitor, shape: Shape) -> String {
        format!("use {};", self.rewrite(visitor, shape))
    }

    fn rewrite(&self, visitor: &FmtVisitor, shape: Shape) -> String {
        let mut result = String::new();

        let mut iter = self.path.iter().peekable();
        while let Some(segment) = iter.next() {
            let segment_str = segment.rewrite(visitor, shape);
            result.push_str(&segment_str);

            if iter.peek().is_some() {
                result.push_str("::");
            }
        }

        result
    }
}
