use std::collections::HashSet;

use arbitrary::Unstructured;
use noirc_frontend::{
    ast::BinaryOpKind,
    monomorphization::ast::{Definition, Expression, Function, IdentId, LocalId, Program, Type},
    shared::Visibility,
};

use super::{
    Context, VariableId, expr, func, types,
    visitor::{visit_expr, visit_expr_mut},
};

/// Find recursive functions and add a `ctx_depth` parameter to them.
pub(crate) fn add_recursion_depth(
    ctx: &mut Context,
    u: &mut Unstructured,
) -> arbitrary::Result<()> {
    // Collect recursive functions, ie. the ones which call other functions.
    let recursive_functions = ctx
        .functions
        .iter()
        .filter_map(|(id, func)| expr::has_call(&func.body).then_some(*id))
        .collect::<HashSet<_>>();

    for (func_id, func) in
        ctx.functions.iter_mut().filter(|(id, _)| recursive_functions.contains(id))
    {
        let is_main = *func_id == Program::main_id();
        // We'll need a new ID for variables or parameters. We could speed this up by
        // 1) caching this value in a "function meta" construct, or
        // 2) using `u32::MAX`, but we wouldn't be able to add caching to `Program`,
        // so eventually we'll need to look at the values to do random mutations.
        let (next_local_id, next_ident_id) = next_local_and_ident_id(func);
        let depth_id = LocalId(next_local_id);
        let depth_name = "ctx_depth".to_string();
        let depth_ident_id = IdentId(next_ident_id);
        let depth_ident = expr::ident_inner(
            VariableId::Local(depth_id),
            depth_ident_id,
            !is_main,
            depth_name.clone(),
            types::U32,
        );
        let depth_expr = Expression::Ident(depth_ident.clone());
        let depth_decreased =
            expr::binary(depth_expr.clone(), BinaryOpKind::Subtract, expr::u32_literal(1));

        if is_main {
            // In main we initialize the depth to its maximum value.
            let init_depth = expr::let_var(
                depth_id,
                false,
                depth_name,
                expr::u32_literal(ctx.config.max_call_depth as u32),
            );
            expr::prepend(&mut func.body, init_depth);
        } else {
            // In non-main we look at the depth and return a random value if it's zero,
            // otherwise decrease it by one and continue with the original body.
            func.parameters.push((depth_id, true, depth_name.clone(), types::U32));
            func.func_sig.0.push(func::hir_param(true, &types::U32, Visibility::Private));

            let default_return = expr::gen_literal(u, &func.return_type)?;

            expr::replace(&mut func.body, |body| {
                expr::if_else(
                    expr::equal(depth_expr.clone(), expr::u32_literal(0)),
                    default_return,
                    Expression::Block(vec![
                        expr::assign(depth_ident, depth_decreased.clone()),
                        body,
                    ]),
                    func.return_type.clone(),
                )
            });
        }

        // Update calls to pass along the depth.
        visit_expr_mut(&mut func.body, &mut |expr| {
            if let Expression::Call(call) = expr {
                let Expression::Ident(func) = call.func.as_mut() else {
                    unreachable!("functions are called by ident");
                };
                let Definition::Function(func_id) = func.definition else {
                    unreachable!("function definition expected");
                };
                // If the callee isn't recursive, it won't have the extra parameter.
                if !recursive_functions.contains(&func_id) {
                    return true;
                }
                let Type::Function(param_types, _, _, _) = &mut func.typ else {
                    unreachable!("function type expected");
                };
                param_types.push(types::U32);
                call.arguments.push(depth_expr.clone());
            }
            true
        });
    }

    Ok(())
}

/// Find the next local ID and ident IDs (in that order) that we can use to add
/// variables to a [Function] during mutations.
fn next_local_and_ident_id(func: &Function) -> (u32, u32) {
    let mut next_local_id = func.parameters.iter().map(|p| p.0.0 + 1).max().unwrap_or_default();
    let mut next_ident_id = 0;

    visit_expr(&func.body, &mut |expr| {
        let local_id = match expr {
            Expression::Let(let_) => Some(let_.id),
            Expression::For(for_) => Some(for_.index_variable),
            Expression::Ident(ident) => {
                next_ident_id = next_ident_id.max(ident.id.0 + 1);
                None
            }
            _ => None,
        };
        if let Some(id) = local_id {
            next_local_id = next_local_id.max(id.0 + 1);
        }
        true
    });
    (next_local_id, next_ident_id)
}
