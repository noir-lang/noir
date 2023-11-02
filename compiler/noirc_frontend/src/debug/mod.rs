use std::collections::HashMap;
use crate::parser::{ParsedModule,parse_program};
use crate::{ast, parser::{Item,ItemKind}, ast::{Path,PathKind}};
use noirc_errors::{Span, Spanned};
use std::collections::VecDeque;

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
        self.insert_state_set_oracle(module);

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

    fn wrap_var_expr(&mut self, var_name: &str, expr: ast::Expression) -> ast::Expression {
        let var_id = self.insert_var(var_name);
        let kind = ast::ExpressionKind::Call(Box::new(ast::CallExpression {
            func: Box::new(ast::Expression {
                kind: ast::ExpressionKind::Variable(ast::Path {
                    segments: vec![ident("__debug_state_set")],
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

    fn wrap_let_statement(&mut self, let_stmt: &ast::LetStatement) -> ast::Statement {
        // rewrites let statements written like this:
        //   let (((a,b,c),D { d }),e,f) = x;
        //
        // into statements like this:
        //
        //   let (a,b,c,d,e,f,g) = {
        //     let (((a,b,c),D { d }),e,f) = x;
        //     wrap(a);
        //     wrap(b);
        //     ...
        //     wrap(f);
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
                span: Span::single_char(0),
            },
        ];
        block_stmts.extend(vars.iter().map(|id| {
            let var_name = &id.0.contents;
            ast::Statement {
                kind: ast::StatementKind::Semi(self.wrap_var_expr(var_name, id_expr(id))),
                span: Span::single_char(0),
            }
        }));
        block_stmts.push(ast::Statement {
            kind: ast::StatementKind::Expression(ast::Expression {
                kind: ast::ExpressionKind::Tuple(vars_exprs),
                span: Span::single_char(0),
            }),
            span: Span::single_char(0),
        });

        ast::Statement {
            kind: ast::StatementKind::Let(ast::LetStatement {
                pattern: ast::Pattern::Tuple(vars_pattern, Span::single_char(0)),
                r#type: ast::UnresolvedType::unspecified(),
                expression: ast::Expression {
                    kind: ast::ExpressionKind::Block(ast::BlockExpression(block_stmts)),
                    span: Span::single_char(0),
                },
            }),
            span: Span::single_char(0),
        }
    }

    fn walk_statement(&mut self, stmt: &mut ast::Statement) {
        match &mut stmt.kind {
            ast::StatementKind::Let(let_stmt) => {
                *stmt = self.wrap_let_statement(&let_stmt);
            },
            _ => {},
        }
    }

    fn insert_state_set_oracle(&self, module: &mut ParsedModule) {
        let (program, errors) = parse_program(r#"
            #[oracle(__debug_state_set)]
            unconstrained fn __debug_state_set_oracle<T>(_var_id: u32, _input: T) {}

            unconstrained pub fn __debug_state_set<T>(var_id: u32, value: T) -> T {
                __debug_state_set_oracle(var_id, value);
                value
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
    ast::Ident(Spanned::from(Span::single_char(0), s.to_string()))
}

fn id_expr(id: &ast::Ident) -> ast::Expression {
    ast::Expression {
        kind: ast::ExpressionKind::Variable(Path {
            segments: vec![id.clone()],
            kind: PathKind::Plain,
        }),
        span: Span::single_char(0),
    }
}
