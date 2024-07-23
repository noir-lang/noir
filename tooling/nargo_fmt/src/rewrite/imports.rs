use noirc_frontend::ast;

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
    Super,
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
            UseSegment::Super => "super".into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct UseTree {
    path: Vec<UseSegment>,
}

impl UseTree {
    pub(crate) fn from_ast(use_tree: ast::UseTree) -> Self {
        let mut result = UseTree { path: vec![] };

        match use_tree.prefix.kind {
            ast::PathKind::Crate => result.path.push(UseSegment::Crate),
            ast::PathKind::Dep => result.path.push(UseSegment::Dep),
            ast::PathKind::Super => result.path.push(UseSegment::Super),
            ast::PathKind::Plain => {}
        };

        result.path.extend(
            use_tree
                .prefix
                .segments
                .into_iter()
                .map(|segment| UseSegment::Ident(segment.to_string(), None)),
        );

        match use_tree.kind {
            ast::UseTreeKind::Path(name, alias) => {
                result.path.push(UseSegment::Ident(
                    name.to_string(),
                    alias.map(|rename| rename.to_string()),
                ));
            }
            ast::UseTreeKind::List(list) => {
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
            let mut segment_str = segment.rewrite(visitor, shape);
            if segment_str.contains('{')
                && !segment_str.contains(',')
                && !segment_str.contains("::")
            {
                let empty = "";
                segment_str = segment_str.replace(['{', '}'], empty);
            }
            result.push_str(&segment_str);

            if iter.peek().is_some() {
                result.push_str("::");
            }
        }

        result
    }
}
