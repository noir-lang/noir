use std::collections::{HashMap,HashSet};
use crate::parser::{ParsedModule,parse_program};
use crate::{ast, parser::{Item,ItemKind}, ast::{Path,PathKind}};
use noirc_errors::{Span, Spanned};
use std::collections::VecDeque;

#[derive(Debug,Clone)]
pub struct DebugState {
    var_id_to_name: HashMap<u32,String>,
    next_var_id: u32,
    scope: Vec<HashSet<u32>>,
    pub enabled: bool,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            var_id_to_name: HashMap::default(),
            scope: vec![],
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
            debug_state.next_var_id = debug_state.next_var_id.max(var_id+1);
        }
        debug_state
    }

    pub fn get_variables(&self) -> HashMap<u32,String> {
        self.var_id_to_name.clone()
    }

    fn insert_var(&mut self, var_name: &str) -> u32 {
        let var_id = self.next_var_id;
        self.next_var_id += 1;
        self.var_id_to_name.insert(var_id, var_name.to_string());
        self.scope.last_mut().unwrap().insert(var_id);
        var_id
    }

    fn walk_fn(&mut self, f: &mut ast::FunctionDefinition) {
        self.scope.push(HashSet::new());

        let pvars: Vec<(u32,ast::Ident)> = f.parameters.iter()
            .flat_map(|(pattern, _utype, _vis)| {
                pattern_vars(pattern).iter().map(|id| {
                    (self.insert_var(&id.0.contents), id.clone())
                }).collect::<Vec<(u32,ast::Ident)>>()
            })
            .collect();

        f.body.0.iter_mut().for_each(|stmt| self.walk_statement(stmt));
        f.body.0 = vec![
            // prapend fn params:
            pvars.iter().map(|(var_id, id)| {
                self.wrap_set_var(*var_id, id_expr(id))
            }).collect(),

            f.body.0.clone(),
        ].concat();
        self.walk_scope(&mut f.body.0);
    }

    // Modify a vector of statements in-place, adding instrumentation for sets and drops.
    // This function will consume a scope level.
    fn walk_scope(&mut self, statements: &mut Vec<ast::Statement>) {
        statements.iter_mut().for_each(|stmt| self.walk_statement(stmt));

        let (ret_stmt, fn_body) = statements.split_last()
            .map(|(e,b)| (e.clone(), b.to_vec()))
            .unwrap_or((
                ast::Statement {
                    kind: ast::StatementKind::Expression(ast::Expression {
                        kind: ast::ExpressionKind::Literal(ast::Literal::Unit),
                        span: none_span(),
                    }),
                    span: none_span(),
                },
                vec![]
            ));

        *statements = vec![
            // copy body minus the return expr:
            fn_body,

            // assign return expr to __debug_state_return:
            vec![match &ret_stmt.kind {
                ast::StatementKind::Expression(ret_expr) => {
                    ast::Statement {
                        kind: ast::StatementKind::Let(ast::LetStatement {
                            pattern: ast::Pattern::Identifier(ident("__debug_state_return")),
                            r#type: ast::UnresolvedType::unspecified(),
                            expression: ret_expr.clone(),
                        }),
                        span: none_span(),
                    }
                },
                _ => ret_stmt.clone(),
            }],

            // drop fn params:
            self.scope.pop().unwrap_or(HashSet::default()).iter().map(|var_id| {
                self.wrap_drop_var(*var_id)
            }).collect(),

            // return the __debug_state_return value:
            vec![match &ret_stmt.kind {
                ast::StatementKind::Expression(ret_expr) => {
                    ast::Statement {
                        kind: ast::StatementKind::Expression(ast::Expression {
                            kind: ast::ExpressionKind::Variable(ast::Path {
                                segments: vec![ident("__debug_state_return")],
                                kind: PathKind::Plain,
                            }),
                            span: none_span(),
                        }),
                        span: none_span(),
                    }
                },
                _ => ast::Statement {
                    kind: ast::StatementKind::Expression(ast::Expression {
                        kind: ast::ExpressionKind::Literal(ast::Literal::Unit),
                        span: none_span(),
                    }),
                    span: none_span(),
                },
            }],
        ].concat();
    }

    pub fn insert_symbols(&mut self, module: &mut ParsedModule) {
        if !self.enabled { return }
        module.items.iter_mut().for_each(|item| {
            match item {
                Item { kind: ItemKind::Function(f), .. } => {
                    self.walk_fn(&mut f.def);
                },
                _ => {},
            }
        });
        // this part absolutely must happen after ast traversal above
        // so that oracle functions don't get wrapped, resulting in infinite recursion:
        self.insert_state_set_oracle(module);
    }

    fn wrap_set_var(&mut self, var_id: u32, expr: ast::Expression) -> ast::Statement {
        let kind = ast::ExpressionKind::Call(Box::new(ast::CallExpression {
            func: Box::new(ast::Expression {
                kind: ast::ExpressionKind::Variable(ast::Path {
                    segments: vec![ident("__debug_state_set")],
                    kind: PathKind::Plain
                }),
                span: none_span(),
            }),
            arguments: vec![
                ast::Expression {
                    kind: ast::ExpressionKind::Literal(ast::Literal::Integer(
                        (var_id as u128).into()
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
                    segments: vec![ident("__debug_state_drop")],
                    kind: PathKind::Plain
                }),
                span: none_span(),
            }),
            arguments: vec![
                ast::Expression {
                    kind: ast::ExpressionKind::Literal(ast::Literal::Integer(
                        (var_id as u128).into()
                    )),
                    span: none_span(),
                },
            ],
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
        let vars_pattern: Vec<ast::Pattern> = vars.iter().map(|id| {
            ast::Pattern::Identifier(id.clone())
        }).collect();
        let vars_exprs: Vec<ast::Expression> = vars.iter().map(|id| id_expr(id)).collect();

        let mut block_stmts = vec![
            ast::Statement {
                kind: ast::StatementKind::Let(let_stmt.clone()),
                span: none_span(),
            },
        ];
        block_stmts.extend(vars.iter().map(|id| {
            let var_id = self.insert_var(&id.0.contents);
            self.wrap_set_var(var_id, id_expr(id))
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

    fn walk_expr(&mut self, expr: &mut ast::Expression) {
        match &mut expr.kind {
            ast::ExpressionKind::Block(ast::BlockExpression(ref mut statements)) => {
                self.scope.push(HashSet::new());
                self.walk_scope(statements);
            },
            _ => {},
        }
    }

    fn walk_for(&mut self, for_stmt: &mut ast::ForLoopStatement) {
        let var_name = &for_stmt.identifier.0.contents;
        let var_id = self.insert_var(var_name);

        let set_stmt = self.wrap_set_var(var_id, id_expr(&for_stmt.identifier));
        let drop_stmt = self.wrap_drop_var(var_id);

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
            },
            ast::StatementKind::Expression(expr) => {
                self.walk_expr(expr);
            },
            ast::StatementKind::Semi(expr) => {
                self.walk_expr(expr);
            },
            ast::StatementKind::For(ref mut for_stmt) => {
                self.walk_for(for_stmt);
            },
            ast::StatementKind::Assign(_assign_stmt) => {
                // TODO
            },
            _ => {}, // Constrain, Error
        }
    }

    fn insert_state_set_oracle(&self, module: &mut ParsedModule) {
        let (program, errors) = parse_program(r#"
            #[oracle(__debug_state_set)]
            unconstrained fn __debug_state_set_oracle<T>(_var_id: u32, _input: T) {}

            #[oracle(__debug_state_drop)]
            unconstrained fn __debug_state_drop_oracle<T>(_var_id: u32) {}

            unconstrained fn __debug_state_set_inner<T>(var_id: u32, value: T) {
                __debug_state_set_oracle(var_id, value);
            }

            unconstrained fn __debug_state_drop_inner<T>(var_id: u32) {
                __debug_state_drop_oracle(var_id);
            }

            pub fn __debug_state_set<T>(var_id: u32, value: T) {
                __debug_state_set_inner(var_id, value);
            }

            pub fn __debug_state_drop<T>(var_id: u32) {
                __debug_state_drop_inner(var_id);
            }
        "#);
        if !errors.is_empty() { panic!("errors parsing internal oracle definitions: {errors:?}") }
        module.items.extend(program.items);
    }
}

fn pattern_vars(pattern: &ast::Pattern) -> Vec<ast::Ident> {
    let mut vars = vec![];
    let mut stack = VecDeque::from([ pattern ]);
    while stack.front().is_some() {
        let pattern = stack.pop_front().unwrap();
        match pattern {
            ast::Pattern::Identifier(id) => {
                vars.push(id.clone());
            },
            ast::Pattern::Mutable(pattern, _) => {
                stack.push_back(pattern);
            },
            ast::Pattern::Tuple(patterns, _) => {
                stack.extend(patterns.iter());
            },
            ast::Pattern::Struct(_, pids, _) => {
                stack.extend(pids.iter().map(|(_, pattern)| pattern));
                vars.extend(pids.iter().map(|(id, _)| id.clone()));
            },
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

fn none_span() -> Span { Span::from_str("") }
