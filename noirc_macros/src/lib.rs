// use noirc_frontend::macros_api::{parse_program, SortedModule, CrateId

use noirc_frontend::macros_api::parse_program;
use noirc_frontend::macros_api::SortedModule;
use noirc_frontend::macros_api::{CrateId, FileId};
use noirc_frontend::macros_api::{
    Expression, ExpressionKind, HirContext, Ident, Path, PathKind, Span, Statement, StatementKind,
};
use noirc_frontend::macros_api::{MacroError, MacroProcessor};

pub struct AssertMessageMacro;

impl MacroProcessor for AssertMessageMacro {
    fn process_untyped_ast(
        &self,
        ast: SortedModule,
        _crate_id: &CrateId,
        _context: &HirContext,
    ) -> Result<SortedModule, (MacroError, FileId)> {
        transform(ast)
    }

    // This macro does not need to process any information after name resolution
    fn process_typed_ast(&self, _crate_id: &CrateId, _context: &mut HirContext) {}
}

fn transform(mut ast: SortedModule) -> Result<SortedModule, (MacroError, FileId)> {
    let assert_message_oracles = "
    #[oracle(assert_message)]
    unconstrained fn assert_message_oracle<T>(_input: T) {}
    unconstrained pub fn resolve_assert_message<T>(input: T) {
        assert_message_oracle(input);
    }";
    // TODO: return parsing errors?
    let (assert_msg_funcs_ast, _) = parse_program(assert_message_oracles);
    let assert_msg_funcs_ast = assert_msg_funcs_ast.into_sorted();
    // TODO: first check whether we have any constrain exprs with messages in the first place
    // other while we can skip this transform
    for func in assert_msg_funcs_ast.functions {
        ast.functions.push(func)
    }

    for func in ast.functions.iter_mut() {
        let mut calls_to_insert = Vec::new();
        for (i, stmt) in func.def.body.0.iter().enumerate() {
            let Statement { kind, span } = stmt;
            if let StatementKind::Constrain(constrain_stmt) = kind {
                if let Some(assert_msg_expr) = &constrain_stmt.1 {
                    let call_expr = Expression::call(
                        Expression {
                            kind: ExpressionKind::Variable(Path {
                                segments: vec![
                                    Ident::from("std"),
                                    Ident::from("resolve_assert_message"),
                                ],
                                kind: PathKind::Dep,
                                span: Span::default(),
                            }),
                            span: Span::default(),
                        },
                        vec![assert_msg_expr.clone()],
                        *span,
                    );
                    calls_to_insert.push((i + calls_to_insert.len(), call_expr, *span));
                } else {
                    let kind = ExpressionKind::string("".to_owned());
                    let arg = Expression { kind, span: Span::default() };
                    let call_expr = Expression::call(
                        Expression {
                            kind: ExpressionKind::Variable(Path {
                                segments: vec![
                                    Ident::from("std"),
                                    Ident::from("resolve_assert_message"),
                                ],
                                kind: PathKind::Dep,
                                span: Span::default(),
                            }),
                            span: Span::default(),
                        },
                        vec![arg],
                        *span,
                    );
                    calls_to_insert.push((i + calls_to_insert.len(), call_expr, *span));
                }
            }
        }

        for (i, call_expr, span) in calls_to_insert {
            func.def
                .body
                .0
                .insert(i, Statement { kind: StatementKind::Expression(call_expr), span });
        }
    }
    Ok(ast)
}
