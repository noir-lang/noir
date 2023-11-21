use crate::parser::{parse_program, ParsedModule};
use crate::{
    ast,
    ast::{Path, PathKind},
    parser::{Item, ItemKind},
};
use noirc_errors::{Span, Spanned};
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DebugState {
    pub variables: HashMap<u32, String>, // var_id => name
    next_var_id: u32,
    scope: Vec<HashSet<u32>>,
    pub enabled: bool,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            variables: HashMap::default(),
            scope: vec![],
            next_var_id: 0,
            enabled: true, // TODO
        }
    }
}

impl DebugState {
    fn insert_var(&mut self, var_name: &str) -> u32 {
        let var_id = self.next_var_id;
        self.next_var_id += 1;
        self.variables.insert(var_id, var_name.to_string());
        self.scope.last_mut().unwrap().insert(var_id);
        var_id
    }

    fn walk_fn(&mut self, f: &mut ast::FunctionDefinition) {
        self.scope.push(HashSet::new());

        let pvars: Vec<(u32, ast::Ident, bool)> = f
            .parameters
            .iter()
            .flat_map(|param| {
                pattern_vars(&param.pattern)
                    .iter()
                    .map(|(id, is_mut)| (self.insert_var(&id.0.contents), id.clone(), *is_mut))
                    .collect::<Vec<(u32, ast::Ident, bool)>>()
            })
            .collect();

        let set_fn_params = pvars
            .iter()
            .map(|(var_id, id, _is_mut)| self.wrap_assign_var(*var_id, id_expr(id)))
            .collect();

        self.walk_scope(&mut f.body.0);

        // prapend fn params:
        f.body.0 = vec![set_fn_params, f.body.0.clone()].concat();
    }

    // Modify a vector of statements in-place, adding instrumentation for sets and drops.
    // This function will consume a scope level.
    fn walk_scope(&mut self, statements: &mut Vec<ast::Statement>) {
        statements.iter_mut().for_each(|stmt| self.walk_statement(stmt));

        let (ret_stmt, fn_body) =
            statements.split_last().map(|(e, b)| (e.clone(), b.to_vec())).unwrap_or((
                ast::Statement {
                    kind: ast::StatementKind::Expression(ast::Expression {
                        kind: ast::ExpressionKind::Literal(ast::Literal::Unit),
                        span: none_span(),
                    }),
                    span: none_span(),
                },
                vec![],
            ));

        *statements = vec![
            // copy body minus the return expr:
            fn_body,
            // assign return expr to __debug_expr:
            vec![match &ret_stmt.kind {
                ast::StatementKind::Expression(ret_expr) => ast::Statement {
                    kind: ast::StatementKind::Let(ast::LetStatement {
                        pattern: ast::Pattern::Identifier(ident("__debug_expr")),
                        r#type: ast::UnresolvedType::unspecified(),
                        expression: ret_expr.clone(),
                    }),
                    span: none_span(),
                },
                _ => ret_stmt.clone(),
            }],
            // drop fn params:
            self.scope
                .pop()
                .unwrap_or(HashSet::default())
                .iter()
                .map(|var_id| self.wrap_drop_var(*var_id))
                .collect(),
            // return the __debug_expr value:
            vec![match &ret_stmt.kind {
                ast::StatementKind::Expression(_ret_expr) => ast::Statement {
                    kind: ast::StatementKind::Expression(ast::Expression {
                        kind: ast::ExpressionKind::Variable(ast::Path {
                            segments: vec![ident("__debug_expr")],
                            kind: PathKind::Plain,
                        }),
                        span: none_span(),
                    }),
                    span: none_span(),
                },
                _ => ast::Statement {
                    kind: ast::StatementKind::Expression(ast::Expression {
                        kind: ast::ExpressionKind::Literal(ast::Literal::Unit),
                        span: none_span(),
                    }),
                    span: none_span(),
                },
            }],
        ]
        .concat();
    }

    pub fn insert_symbols(&mut self, module: &mut ParsedModule) {
        if !self.enabled {
            return;
        }
        module.items.iter_mut().for_each(|item| match item {
            Item { kind: ItemKind::Function(f), .. } => {
                self.walk_fn(&mut f.def);
            }
            _ => {}
        });
        // this part absolutely must happen after ast traversal above
        // so that oracle functions don't get wrapped, resulting in infinite recursion:
        self.insert_state_set_oracle(module);
    }

    fn wrap_assign_var(&mut self, var_id: u32, expr: ast::Expression) -> ast::Statement {
        let kind = ast::ExpressionKind::Call(Box::new(ast::CallExpression {
            func: Box::new(ast::Expression {
                kind: ast::ExpressionKind::Variable(ast::Path {
                    segments: vec![ident("__debug_var_assign")],
                    kind: PathKind::Plain,
                }),
                span: none_span(),
            }),
            arguments: vec![
                ast::Expression {
                    kind: ast::ExpressionKind::Literal(ast::Literal::Integer(
                        (var_id as u128).into(),
                    )),
                    span: none_span(),
                },
                expr,
            ],
        }));
        ast::Statement {
            kind: ast::StatementKind::Semi(ast::Expression { kind, span: none_span() }),
            span: none_span(),
        }
    }

    fn wrap_drop_var(&mut self, var_id: u32) -> ast::Statement {
        let kind = ast::ExpressionKind::Call(Box::new(ast::CallExpression {
            func: Box::new(ast::Expression {
                kind: ast::ExpressionKind::Variable(ast::Path {
                    segments: vec![ident("__debug_var_drop")],
                    kind: PathKind::Plain,
                }),
                span: none_span(),
            }),
            arguments: vec![ast::Expression {
                kind: ast::ExpressionKind::Literal(ast::Literal::Integer((var_id as u128).into())),
                span: none_span(),
            }],
        }));
        ast::Statement {
            kind: ast::StatementKind::Semi(ast::Expression { kind, span: none_span() }),
            span: none_span(),
        }
    }

    fn wrap_let_statement(&mut self, let_stmt: &ast::LetStatement, span: &Span) -> ast::Statement {
        // rewrites let statements written like this:
        //   let (((a,b,c),D { d }),e,f) = x;
        //
        // into statements like this:
        //
        //   let (a,b,c,d,e,f,g) = {
        //     let (((a,b,c),D { d }),e,f) = x;
        //     wrap(1, a);
        //     wrap(2, b);
        //     ...
        //     wrap(6, f);
        //     (a,b,c,d,e,f,g)
        //   };

        let vars = pattern_vars(&let_stmt.pattern);
        let vars_pattern: Vec<ast::Pattern> = vars
            .iter()
            .map(|(id, is_mut)| {
                if *is_mut {
                    ast::Pattern::Mutable(
                        Box::new(ast::Pattern::Identifier(id.clone())),
                        none_span(),
                    )
                } else {
                    ast::Pattern::Identifier(id.clone())
                }
            })
            .collect();
        let vars_exprs: Vec<ast::Expression> = vars.iter().map(|(id, _)| id_expr(id)).collect();

        let mut block_stmts = vec![ast::Statement {
            kind: ast::StatementKind::Let(let_stmt.clone()),
            span: none_span(),
        }];
        block_stmts.extend(vars.iter().map(|(id, _)| {
            let var_id = self.insert_var(&id.0.contents);
            self.wrap_assign_var(var_id, id_expr(id))
        }));
        block_stmts.push(ast::Statement {
            kind: ast::StatementKind::Expression(ast::Expression {
                kind: ast::ExpressionKind::Tuple(vars_exprs),
                span: none_span(),
            }),
            span: none_span(),
        });

        ast::Statement {
            kind: ast::StatementKind::Let(ast::LetStatement {
                pattern: ast::Pattern::Tuple(vars_pattern, none_span()),
                r#type: ast::UnresolvedType::unspecified(),
                expression: ast::Expression {
                    kind: ast::ExpressionKind::Block(ast::BlockExpression(block_stmts)),
                    span: none_span(),
                },
            }),
            span: span.clone(),
        }
    }

    fn wrap_assign_statement(
        &mut self,
        assign_stmt: &ast::AssignStatement,
        span: &Span,
    ) -> ast::Statement {
        // X = Y becomes:
        // X = {
        //   let __debug_expr = Y;
        //   wrap(1, __debug_expr);
        //   __debug_expr
        // };

        let let_kind = ast::StatementKind::Let(ast::LetStatement {
            pattern: ast::Pattern::Identifier(ident("__debug_expr")),
            r#type: ast::UnresolvedType::unspecified(),
            expression: assign_stmt.expression.clone(),
        });
        let new_assign_stmt = match &assign_stmt.lvalue {
            ast::LValue::Ident(id) => {
                let var_id = self.insert_var(&id.0.contents);
                self.wrap_assign_var(var_id, id_expr(&ident("__debug_expr")))
            }
            ast::LValue::MemberAccess { object: _object, field_name: _field_name } => {
                // TODO
                unimplemented![]
            }
            ast::LValue::Index { array: _array, index: _index } => {
                // TODO
                // TODO: also remember to self.walk(index)
                unimplemented![]
            }
            ast::LValue::Dereference(_lv) => {
                // TODO
                unimplemented![]
            }
        };
        let ret_kind = ast::StatementKind::Expression(id_expr(&ident("__debug_expr")));

        ast::Statement {
            kind: ast::StatementKind::Assign(ast::AssignStatement {
                lvalue: assign_stmt.lvalue.clone(),
                expression: ast::Expression {
                    kind: ast::ExpressionKind::Block(ast::BlockExpression(vec![
                        ast::Statement { kind: let_kind, span: none_span() },
                        new_assign_stmt,
                        ast::Statement { kind: ret_kind, span: none_span() },
                    ])),
                    span: none_span(),
                },
            }),
            span: span.clone(),
        }
    }

    fn walk_expr(&mut self, expr: &mut ast::Expression) {
        match &mut expr.kind {
            ast::ExpressionKind::Block(ast::BlockExpression(ref mut statements)) => {
                self.scope.push(HashSet::new());
                self.walk_scope(statements);
            }
            ast::ExpressionKind::Prefix(prefix_expr) => {
                self.walk_expr(&mut prefix_expr.rhs);
            }
            ast::ExpressionKind::Index(index_expr) => {
                self.walk_expr(&mut index_expr.collection);
                self.walk_expr(&mut index_expr.index);
            }
            ast::ExpressionKind::Call(call_expr) => {
                // TODO: push a stack frame or something here?
                self.walk_expr(&mut call_expr.func);
                call_expr.arguments.iter_mut().for_each(|ref mut expr| {
                    self.walk_expr(expr);
                });
            }
            ast::ExpressionKind::MethodCall(mc_expr) => {
                // TODO: also push a stack frame here
                self.walk_expr(&mut mc_expr.object);
                mc_expr.arguments.iter_mut().for_each(|ref mut expr| {
                    self.walk_expr(expr);
                });
            }
            ast::ExpressionKind::Constructor(c_expr) => {
                c_expr.fields.iter_mut().for_each(|(_id, ref mut expr)| {
                    self.walk_expr(expr);
                });
            }
            ast::ExpressionKind::MemberAccess(ma_expr) => {
                self.walk_expr(&mut ma_expr.lhs);
            }
            ast::ExpressionKind::Cast(cast_expr) => {
                self.walk_expr(&mut cast_expr.lhs);
            }
            ast::ExpressionKind::Infix(infix_expr) => {
                self.walk_expr(&mut infix_expr.lhs);
                self.walk_expr(&mut infix_expr.rhs);
            }
            ast::ExpressionKind::If(if_expr) => {
                self.walk_expr(&mut if_expr.condition);
                self.walk_expr(&mut if_expr.consequence);
                if let Some(ref mut alt) = if_expr.alternative {
                    self.walk_expr(alt);
                }
            }
            ast::ExpressionKind::Tuple(exprs) => {
                exprs.iter_mut().for_each(|ref mut expr| {
                    self.walk_expr(expr);
                });
            }
            ast::ExpressionKind::Lambda(lambda) => {
                self.walk_expr(&mut lambda.body);
            }
            ast::ExpressionKind::Parenthesized(expr) => {
                self.walk_expr(expr);
            }
            _ => {}
        }
    }

    fn walk_for(&mut self, for_stmt: &mut ast::ForLoopStatement) {
        let var_name = &for_stmt.identifier.0.contents;
        let var_id = self.insert_var(var_name);

        let set_stmt = self.wrap_assign_var(var_id, id_expr(&for_stmt.identifier));
        let drop_stmt = self.wrap_drop_var(var_id);

        self.walk_expr(&mut for_stmt.block);
        for_stmt.block = ast::Expression {
            kind: ast::ExpressionKind::Block(ast::BlockExpression(vec![
                set_stmt,
                ast::Statement {
                    kind: ast::StatementKind::Semi(for_stmt.block.clone()),
                    span: none_span(),
                },
                drop_stmt,
            ])),
            span: none_span(),
        };
    }

    fn walk_statement(&mut self, stmt: &mut ast::Statement) {
        match &mut stmt.kind {
            ast::StatementKind::Let(let_stmt) => {
                *stmt = self.wrap_let_statement(&let_stmt, &stmt.span);
            }
            ast::StatementKind::Assign(assign_stmt) => {
                *stmt = self.wrap_assign_statement(&assign_stmt, &stmt.span);
            }
            ast::StatementKind::Expression(expr) => {
                self.walk_expr(expr);
            }
            ast::StatementKind::Semi(expr) => {
                self.walk_expr(expr);
            }
            ast::StatementKind::For(ref mut for_stmt) => {
                self.walk_for(for_stmt);
            }
            _ => {} // Constrain, Error
        }
    }

    fn insert_state_set_oracle(&self, module: &mut ParsedModule) {
        let (program, errors) = parse_program(
            r#"
            #[oracle(__debug_var_assign)]
            unconstrained fn __debug_var_assign_oracle<T>(_var_id: u32, _value: T) {}
            unconstrained fn __debug_var_assign_inner<T>(var_id: u32, value: T) {
                __debug_var_assign_oracle(var_id, value);
            }
            pub fn __debug_var_assign<T>(var_id: u32, value: T) {
                __debug_var_assign_inner(var_id, value);
            }

            #[oracle(__debug_var_drop)]
            unconstrained fn __debug_var_drop_oracle<T>(_var_id: u32) {}
            unconstrained fn __debug_var_drop_inner<T>(var_id: u32) {
                __debug_var_drop_oracle(var_id);
            }
            pub fn __debug_var_drop<T>(var_id: u32) {
                __debug_var_drop_inner(var_id);
            }

            #[oracle(__debug_member_assign)]
            unconstrained fn __debug_member_assign_oracle<T>(_var_id: u32, _member_id: u32, _value: T) {}
            unconstrained fn __debug_member_assign_inner<T>(var_id: u32, member_id: u32, value: T) {
                __debug_member_assign_oracle(var_id, member_id, value);
            }
            pub fn __debug_member_assign<T>(var_id: u32, member_id: u32, value: T) {
                __debug_member_assign_inner(var_id, member_id, value);
            }

            #[oracle(__debug_index_assign)]
            unconstrained fn __debug_index_assign_oracle<T>(_var_id: u32, _index: Field, _value: T) {}
            unconstrained fn __debug_index_assign_inner<T>(var_id: u32, index: Field, value: T) {
                __debug_index_assign_oracle(var_id, index, value);
            }
            pub fn __debug_index_assign<T>(var_id: u32, index: Field, value: T) {
                __debug_index_assign_inner(var_id, index, value);
            }

            #[oracle(__debug_dereference_assign)]
            unconstrained fn __debug_dereference_assign_oracle<T>(_var_id: u32, _value: T) {}
            unconstrained fn __debug_dereference_assign_inner<T>(var_id: u32, value: T) {
                __debug_dereference_assign_oracle(var_id, value);
            }
            pub fn __debug_dereference_assign<T>(var_id: u32, value: T) {
                __debug_dereference_assign_inner(var_id, value);
            }
        "#,
        );
        if !errors.is_empty() {
            panic!("errors parsing internal oracle definitions: {errors:?}")
        }
        module.items.extend(program.items);
    }
}

fn pattern_vars(pattern: &ast::Pattern) -> Vec<(ast::Ident, bool)> {
    let mut vars = vec![];
    let mut stack = VecDeque::from([(pattern, false)]);
    while stack.front().is_some() {
        let (pattern, is_mut) = stack.pop_front().unwrap();
        match pattern {
            ast::Pattern::Identifier(id) => {
                vars.push((id.clone(), is_mut));
            }
            ast::Pattern::Mutable(pattern, _) => {
                stack.push_back((pattern, true));
            }
            ast::Pattern::Tuple(patterns, _) => {
                stack.extend(patterns.iter().map(|pattern| (pattern, false)));
            }
            ast::Pattern::Struct(_, pids, _) => {
                stack.extend(pids.iter().map(|(_, pattern)| (pattern, is_mut)));
                vars.extend(pids.iter().map(|(id, _)| (id.clone(), false)));
            }
        }
    }
    vars
}

fn ident(s: &str) -> ast::Ident {
    ast::Ident(Spanned::from(none_span(), s.to_string()))
}

fn id_expr(id: &ast::Ident) -> ast::Expression {
    ast::Expression {
        kind: ast::ExpressionKind::Variable(Path {
            segments: vec![id.clone()],
            kind: PathKind::Plain,
        }),
        span: none_span(),
    }
}

fn none_span() -> Span {
    Span::from_str("")
}
