use std::collections::BTreeMap;

use arbitrary::Unstructured;
use im::HashMap;
use nargo::errors::Location;
use noirc_frontend::{
    ast::BinaryOpKind,
    monomorphization::ast::{
        Call, Definition, Expression, FuncId, Function, Ident, IdentId, LocalId, Program, Type,
    },
    shared::Visibility,
};

use super::{
    Context, VariableId, expr, types,
    visitor::{visit_expr, visit_expr_mut},
};

/// Find recursive functions and add a `ctx_limit: &mut u32` parameter to them,
/// which we use to limit the number of recursive calls. This is complicated by
/// the fact that we cannot pass mutable references from ACIR to Brillig. To
/// overcome that, we create a proxy function for unconstrained functions that
/// take `mut ctx_limit: u32` instead, and pass it on as a mutable ref.
pub(crate) fn add_recursion_limit(
    ctx: &mut Context,
    u: &mut Unstructured,
) -> arbitrary::Result<()> {
    // Collect recursive functions, ie. the ones which call other functions.
    // Remember if they are unconstrained; those need proxies as well.
    let recursive_functions = ctx
        .functions
        .iter()
        .filter_map(|(id, func)| expr::has_call(&func.body).then_some((*id, func.unconstrained)))
        .collect::<BTreeMap<_, _>>();

    // Create proxies for unconstrained recursive functions.
    // We could check whether they are called from ACIR, but that would require further traversals.
    let mut proxy_functions = HashMap::new();
    let mut next_func_id = FuncId(ctx.functions.len() as u32);

    for (func_id, unconstrained) in &recursive_functions {
        if !*unconstrained || *func_id == Program::main_id() {
            continue;
        }
        let mut proxy = ctx.functions[func_id].clone();
        proxy.id = next_func_id;
        proxy.name = format!("{}_proxy", proxy.name);
        // We will replace the body and update the params later.
        proxy_functions.insert(*func_id, proxy);
        next_func_id = FuncId(next_func_id.0 + 1);
    }

    // Rewrite recursive functions.
    for (func_id, unconstrained) in recursive_functions.iter() {
        let func = ctx.functions.get_mut(func_id).unwrap();
        let is_main = *func_id == Program::main_id();

        // We'll need a new ID for variables or parameters. We could speed this up by
        // 1) caching this value in a "function meta" construct, or
        // 2) using `u32::MAX`, but then we would be in a worse situation next time
        // 3) draw values from `Context` instead of `FunctionContext`, which breaks continuity, but saves an extra traversal.
        // We wouldn't be able to add caching to `Program` without changing it, so eventually we'll need to look at the values
        // to do random mutations, or we have to pass back some meta along with `Program` and look it up there. For now we
        // traverse the AST to figure out what the next ID to use is.
        let (mut next_local_id, mut next_ident_id) = next_local_and_ident_id(func);

        let mut next_local_id = || {
            let id = next_local_id;
            next_local_id += 1;
            LocalId(id)
        };

        let mut next_ident_id = || {
            let id = next_ident_id;
            next_ident_id += 1;
            IdentId(id)
        };

        let limit_name = "ctx_limit".to_string();
        let limit_id = next_local_id();
        let limit_var = VariableId::Local(limit_id);

        if is_main {
            // In main we initialize the limit to its maximum value.
            let init_limit = expr::let_var(
                limit_id,
                true,
                limit_name.clone(),
                expr::u32_literal(ctx.config.max_recursive_calls as u32),
            );
            expr::prepend(&mut func.body, init_limit);
        } else {
            // In non-main we look at the limit and return a random value if it's zero,
            // otherwise decrease it by one and continue with the original body.
            let limit_type = types::ref_mut(types::U32);
            func.parameters.push((
                limit_id,
                false,
                limit_name.clone(),
                limit_type.clone(),
                Visibility::Private,
            ));

            // Generate a random value to return.
            let default_return = expr::gen_literal(u, &func.return_type)?;

            let limit_ident = expr::ident_inner(
                limit_var,
                next_ident_id(),
                false,
                limit_name.clone(),
                limit_type,
            );
            let limit_expr = Expression::Ident(limit_ident.clone());

            expr::replace(&mut func.body, |mut body| {
                expr::prepend(
                    &mut body,
                    expr::assign_ref(
                        limit_ident,
                        expr::binary(
                            expr::deref(limit_expr.clone(), types::U32),
                            BinaryOpKind::Subtract,
                            expr::u32_literal(1),
                        ),
                    ),
                );
                expr::if_else(
                    expr::equal(expr::deref(limit_expr.clone(), types::U32), expr::u32_literal(0)),
                    default_return,
                    body,
                    func.return_type.clone(),
                )
            });
        }

        // Add the non-reference version of the parameter to the proxy function.
        if let Some(proxy) = proxy_functions.get_mut(func_id) {
            proxy.parameters.push((
                limit_id,
                true,
                limit_name.clone(),
                types::U32,
                Visibility::Private,
            ));
            // The body is just a call the the non-proxy function.
            proxy.body = Expression::Call(Call {
                func: Box::new(Expression::Ident(Ident {
                    location: None,
                    definition: Definition::Function(*func_id),
                    mutable: false,
                    name: func.name.clone(),
                    typ: Type::Function(
                        func.parameters.iter().map(|p| p.3.clone()).collect(),
                        Box::new(func.return_type.clone()),
                        Box::new(Type::Unit),
                        func.unconstrained,
                    ),
                    id: next_ident_id(),
                })),
                arguments: proxy
                    .parameters
                    .iter()
                    .map(|(id, mutable, name, typ, _visibility)| {
                        if *id == limit_id {
                            // Pass mutable reference to the limit.
                            expr::ref_mut(
                                expr::ident(
                                    VariableId::Local(*id),
                                    next_ident_id(),
                                    *mutable,
                                    name.clone(),
                                    typ.clone(),
                                ),
                                typ.clone(),
                            )
                        } else {
                            // Pass every other parameter as-is.
                            expr::ident(
                                VariableId::Local(*id),
                                next_ident_id(),
                                *mutable,
                                name.clone(),
                                typ.clone(),
                            )
                        }
                    })
                    .collect(),
                return_type: proxy.return_type.clone(),
                location: Location::dummy(),
            });
        }

        // Update calls to pass along the limit and call the proxy if necessary.
        visit_expr_mut(&mut func.body, &mut |expr: &mut Expression| {
            if let Expression::Call(call) = expr {
                let Expression::Ident(func) = call.func.as_mut() else {
                    unreachable!("functions are called by ident");
                };
                let Definition::Function(func_id) = func.definition else {
                    unreachable!("function definition expected");
                };
                // If the callee isn't recursive, it won't have the extra parameter.
                let Some(callee_unconstrained) = recursive_functions.get(&func_id) else {
                    return true;
                };
                let Type::Function(param_types, _, _, _) = &mut func.typ else {
                    unreachable!("function type expected");
                };
                if *callee_unconstrained && !unconstrained {
                    // Calling Brillig from ACIR: call the proxy.
                    let Some(proxy) = proxy_functions.get(&func_id) else {
                        unreachable!("expected to have a proxy");
                    };
                    func.name = proxy.name.clone();
                    func.definition = Definition::Function(proxy.id);
                    // Pass the limit by value.
                    let limit_expr = if is_main {
                        expr::ident(
                            limit_var,
                            next_ident_id(),
                            true,
                            limit_name.clone(),
                            types::U32,
                        )
                    } else {
                        expr::deref(
                            expr::ident(
                                limit_var,
                                next_ident_id(),
                                false,
                                limit_name.clone(),
                                types::ref_mut(types::U32),
                            ),
                            types::U32,
                        )
                    };
                    param_types.push(types::U32);
                    call.arguments.push(limit_expr);
                } else {
                    // Pass the limit by reference.
                    let limit_type = types::ref_mut(types::U32);
                    let limit_expr = if is_main {
                        expr::ref_mut(
                            expr::ident(
                                limit_var,
                                next_ident_id(),
                                true,
                                limit_name.clone(),
                                types::U32,
                            ),
                            limit_type,
                        )
                    } else {
                        expr::ident(
                            limit_var,
                            next_ident_id(),
                            false,
                            limit_name.clone(),
                            limit_type,
                        )
                    };
                    param_types.push(types::U32);
                    call.arguments.push(limit_expr);
                }
            }
            true
        });
    }

    // Append proxy functions.
    for (_, proxy) in proxy_functions {
        ctx.functions.insert(proxy.id, proxy);
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

/// Turn all ACIR functions into Brillig functions.
///
/// This is more involved than flipping the `unconstrained` property because of the
/// "ownership analysis", which can only run on a function once.
pub fn change_all_functions_into_unconstrained(mut program: Program) -> Program {
    for f in program.functions.iter_mut() {
        if f.unconstrained {
            continue;
        }
        f.unconstrained = true;
        f.handle_ownership();
    }
    program
}
