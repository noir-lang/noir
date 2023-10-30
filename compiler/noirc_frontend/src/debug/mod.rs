use std::collections::HashMap;
use crate::parser::ParsedModule;
use crate::{ast, parser::{Item,ItemKind}, ast::{Path,PathKind,UseTreeKind}};
use noirc_errors::{Span, Spanned};

#[derive(Debug,Clone)]
pub struct DebugState {
    var_id_to_name: HashMap<u32,String>,
    var_name_to_id: HashMap<String,u32>,
    next_var_id: u32,
    pub enabled: bool,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            var_id_to_name: HashMap::new(),
            var_name_to_id: HashMap::new(),
            next_var_id: 0,
            enabled: true, // TODO
        }
    }
}

impl DebugState {
    pub fn new(vars: HashMap<String,u32>) -> Self {
        let mut debug_state = Self::default();
        for (var_name, var_id) in vars.iter() {
            debug_state.var_id_to_name.insert(*var_id, var_name.clone());
            debug_state.var_name_to_id.insert(var_name.clone(), *var_id);
            debug_state.next_var_id = debug_state.next_var_id.max(var_id+1);
        }
        debug_state
    }

    pub fn get_variables(&self) -> HashMap<String,u32> {
        self.var_name_to_id.clone()
    }

    fn insert_var(&mut self, var_name: &str) -> u32 {
        let var_id = self.next_var_id;
        self.next_var_id += 1;
        self.var_id_to_name.insert(var_id, var_name.to_string());
        self.var_name_to_id.insert(var_name.to_string(), var_id);
        var_id
    }

    pub fn insert_symbols(&mut self, module: &mut ParsedModule) {
        if !self.enabled { return }
        /*
        let empty = Span::single_char(0);
        let prefix = Path {
            segments: vec![ ast::Ident(Spanned::from(empty.clone(), "std".to_string())) ],
            kind: PathKind::Dep,
        };
        let kind = UseTreeKind::Path(
            ast::Ident(Spanned::from(empty.clone(), "__debug_state_set".to_string())),
            Some(ast::Ident(Spanned::from(empty.clone(), "__debug_state_set".to_string()))),
        );
        module.items.push(Item {
            kind: ItemKind::Import(ast::UseTree { prefix, kind }),
            span: empty.clone(),
        });
        */
        module.items.iter_mut().for_each(|item| {
            match item {
                Item { kind: ItemKind::Function(f), .. } => {
                    // todo: f.def.parameters
                    f.def.body.0.iter_mut().for_each(|stmt| self.walk_statement(stmt));
                },
                _ => {},
            }
        });
    }

    fn walk_expr(&mut self, _expr: &mut ast::Expression) {
    }

    fn debug_expr(&mut self, var_name: &str, expr: ast::Expression) -> ast::Expression {
        let var_id = self.insert_var(var_name);
        let kind = ast::ExpressionKind::Call(Box::new(ast::CallExpression {
            func: Box::new(ast::Expression {
                kind: ast::ExpressionKind::Variable(ast::Path {
                    segments: vec![ast::Ident(
                        Spanned::from(Span::single_char(0), "__debug_state_set".to_string())
                    )],
                    kind: PathKind::Plain
                }),
                span: Span::single_char(0),
            }),
            arguments: vec![
                ast::Expression {
                    kind: ast::ExpressionKind::Literal(ast::Literal::Integer(
                        (var_id as u128).into()
                    )),
                    span: Span::single_char(0),
                },
                expr
            ],
        }));
        ast::Expression { kind, span: Span::single_char(0) }
    }

    fn walk_statement(&mut self, stmt: &mut ast::Statement) {
        match &mut stmt.kind {
            ast::StatementKind::Let(ref mut let_stmt) => {
                match &let_stmt.pattern {
                    ast::Pattern::Identifier(id) => {
                        let expr = let_stmt.expression.clone();
                        let_stmt.expression = self.debug_expr(&id.0.contents, expr);
                    },
                    ast::Pattern::Mutable(_, _) => {
                    },
                    ast::Pattern::Tuple(_, _) => {
                    },
                    ast::Pattern::Struct(_, _, _) => {
                    },
                }
                self.walk_expr(&mut let_stmt.expression);
            },
            _ => {},
        }
    }
}
