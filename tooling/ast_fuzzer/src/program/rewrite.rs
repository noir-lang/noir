use std::collections::HashSet;

use arbitrary::Unstructured;
use noirc_frontend::{
    ast::BinaryOpKind,
    monomorphization::ast::{Expression, LocalId, Program, Type},
    shared::Visibility,
};

use super::{Context, VariableId, expr, func, types, visitor::visit_expr_mut};

/// Find recursive functions and add a `ctx_call_depth` parameter to them.
pub(crate) fn add_recursion_depth(
    ctx: &mut Context,
    u: &mut Unstructured,
) -> arbitrary::Result<()> {
    // Collect recursive functions, ie. the ones which call other functions.
    let callers = ctx
        .functions
        .iter()
        .filter_map(|(id, func)| expr::has_call(&func.body).then_some(*id))
        .collect::<HashSet<_>>();

    for (func_id, func) in ctx.functions.iter_mut().filter(|(id, _)| callers.contains(id)) {
        let is_main = *func_id == Program::main_id();
        // We'll need a new ID for variables or parameters. We could speed this up by
        // 1) caching this value in a "function meta" construct, or
        // 2) using `u32::MAX`, but we wouldn't be able to add caching to `Program`,
        // so eventually we'll need to look at the values to do random mutations.
        let depth_id = LocalId(func::next_local_id(func));
        let depth_name = "ctx_call_depth".to_string();
        let depth_ident_inner = expr::ident_inner(
            VariableId::Local(depth_id),
            !is_main,
            depth_name.clone(),
            types::U32,
        );
        let depth_ident = Expression::Ident(depth_ident_inner.clone());

        if is_main {
            let init_depth = expr::let_var(
                depth_id,
                false,
                depth_name,
                expr::u32_literal(ctx.config.max_call_depth as u32),
            );
            expr::prepend(&mut func.body, init_depth);
        } else {
            func.parameters.push((depth_id, true, depth_name.clone(), types::U32));
            func.func_sig.0.push(func::hir_param(true, &types::U32, Visibility::Private));

            let default_return = expr::gen_literal(u, &func.return_type)?;

            expr::replace(&mut func.body, |body| {
                expr::if_then(
                    expr::equal(depth_ident.clone(), expr::u32_literal(0)),
                    default_return,
                    Some(Expression::Block(vec![expr::assign(depth_ident_inner, body)])),
                    func.return_type.clone(),
                )
            });
        }

        // Update calls to pass along the depth.
        let decrease_depth =
            expr::binary(depth_ident, BinaryOpKind::Subtract, expr::u32_literal(1));

        visit_expr_mut(&mut func.body, &mut |expr| {
            if let Expression::Call(call) = expr {
                call.arguments.push(decrease_depth.clone());
                if let Expression::Ident(func) = call.func.as_mut() {
                    if let Type::Function(param_types, _, _, _) = &mut func.typ {
                        param_types.push(types::U32)
                    }
                }
            }
            true
        });
    }

    Ok(())
}
