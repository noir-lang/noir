use std::collections::{HashMap, HashSet};

use arbitrary::Unstructured;
use nargo::errors::Location;
use noirc_frontend::{
    ast::BinaryOpKind,
    monomorphization::{
        ast::{
            Call, Definition, Expression, FuncId, Function, Ident, IdentId, LocalId, Program, Type,
        },
        visitor::visit_expr_mut,
    },
    shared::Visibility,
};

use crate::{
    Config,
    program::{Context, VariableId, expr, types},
};

use super::next_local_and_ident_id;

const LIMIT_NAME: &str = "ctx_limit";

/// To avoid the potential of infinite recursion at runtime, add a `ctx_limit: &mut u32`
/// parameter to all functions, which we use to limit the number of recursive calls.
///
/// This is complicated by the fact that we cannot pass mutable references from ACIR to Brillig.
/// To overcome that, we create a proxy function for unconstrained functions that take
/// `mut ctx_limit: u32` instead, and pass it on as a mutable ref.
///
/// Originally only actually recursive functions (ie. one that called something else)
/// received this extra parameters, but in order to support higher order functions
/// which can be passed a recursive or a non-recursive function as an argument,
/// all functions get the extra parameter.
pub(crate) fn add_recursion_limit(
    ctx: &mut Context,
    u: &mut Unstructured,
) -> arbitrary::Result<()> {
    // Collect functions potentially called from ACIR; they will need proxy functions.
    let called_from_acir = ctx.functions.values().filter(|func| !func.unconstrained).fold(
        HashSet::<FuncId>::new(),
        |mut acc, func| {
            acc.extend(expr::reachable_functions(&func.body));
            acc
        },
    );

    // Create proxies for unconstrained functions called from ACIR.
    let mut proxy_functions = HashMap::new();
    let mut next_func_id = ctx.functions.len() as u32;

    for (func_id, func) in &ctx.functions {
        if !func.unconstrained
            || *func_id == Program::main_id()
            || !called_from_acir.contains(func_id)
        {
            continue;
        }
        let mut proxy = func.clone();
        proxy.id = FuncId(next_func_id);
        proxy.name = format!("{}_proxy", proxy.name);
        // We will replace the body, update the params, and append the function later.
        proxy_functions.insert(*func_id, proxy);
        next_func_id += 1;
    }

    // Rewrite functions.
    for (func_id, func) in ctx.functions.iter_mut() {
        let mut limit_ctx = LimitContext::new(*func_id, func, &ctx.config);

        limit_ctx.rewrite_functions(u, &mut proxy_functions)?;
    }

    // Append proxy functions.
    for (_, proxy) in proxy_functions {
        ctx.functions.insert(proxy.id, proxy);
    }

    Ok(())
}

/// Decide how to pass the recursion limit to function: by value or by ref.
fn ctx_limit_type_for_func_param(callee_unconstrained: bool, param_unconstrained: bool) -> Type {
    // If the function receiving the parameter is ACIR, and the function we pass
    // to it is Brillig, it will have to pass the limit by value.
    // Otherwise by ref should work. We don't pass ACIR to Brillig.
    if !callee_unconstrained && param_unconstrained {
        types::U32
    } else {
        types::ref_mut(types::U32)
    }
}

struct LimitContext<'a, 'b> {
    func_id: FuncId,
    func: &'a mut Function,
    config: &'b Config,
    is_main: bool,
    is_recursive: bool,
    next_local_id: u32,
    next_ident_id: u32,
}

impl<'a, 'b> LimitContext<'a, 'b> {
    fn new(func_id: FuncId, func: &'a mut Function, config: &'b Config) -> Self {
        let is_main = func_id == Program::main_id();

        // Recursive functions are those that call another function.
        let is_recursive = expr::has_call(&func.body);

        // We'll need a new ID for variables or parameters. We could speed this up by
        // 1) caching this value in a "function meta" construct, or
        // 2) using `u32::MAX`, but then we would be in a worse situation next time
        // 3) draw values from `Context` instead of `FunctionContext`, which breaks continuity, but saves an extra traversal.
        // We wouldn't be able to add caching to `Program` without changing it, so eventually we'll need to look at the values
        // to do random mutations, or we have to pass back some meta along with `Program` and look it up there. For now we
        // traverse the AST to figure out what the next ID to use is.
        let (next_local_id, next_ident_id) = next_local_and_ident_id(func);

        Self { func_id, func, config, is_main, is_recursive, next_local_id, next_ident_id }
    }

    /// Rewrite the function and its proxy (if it has one).
    fn rewrite_functions(
        &mut self,
        u: &mut Unstructured,
        proxy_functions: &mut HashMap<FuncId, Function>,
    ) -> arbitrary::Result<()> {
        let limit_id = self.next_local_id();

        // Limit variable operations in the body
        if self.is_main {
            self.modify_body_when_main(limit_id);
        } else if self.is_recursive {
            self.modify_body_when_recursive(u, limit_id)?;
        } else {
            self.modify_body_when_non_recursive(limit_id);
        }

        // Call forwarding in the proxy
        self.set_proxy_function(limit_id, proxy_functions);

        // Passing along the limit in calls
        self.modify_calls(limit_id, proxy_functions);

        // Update function pointer types to have the extra parameter.
        self.modify_function_pointer_param_types(proxy_functions);

        Ok(())
    }

    fn next_local_id(&mut self) -> LocalId {
        let id = self.next_local_id;
        self.next_local_id += 1;
        LocalId(id)
    }

    fn next_ident_id(&mut self) -> IdentId {
        let id = self.next_ident_id;
        self.next_ident_id += 1;
        IdentId(id)
    }

    /// In `main` we initialize the recursion limit.
    fn modify_body_when_main(&mut self, limit_id: LocalId) {
        let init_limit = expr::let_var(
            limit_id,
            true,
            LIMIT_NAME.to_string(),
            expr::u32_literal(self.config.max_recursive_calls as u32),
        );
        expr::prepend(&mut self.func.body, init_limit);
    }

    /// In non-main we look at the limit and return a random value if it's zero,
    /// otherwise decrease it by one and continue with the original body.
    fn modify_body_when_recursive(
        &mut self,
        u: &mut Unstructured,
        limit_id: LocalId,
    ) -> arbitrary::Result<()> {
        let limit_var = VariableId::Local(limit_id);

        let limit_type = types::ref_mut(types::U32);
        self.func.parameters.push((
            limit_id,
            false,
            LIMIT_NAME.to_string(),
            limit_type.clone(),
            Visibility::Private,
        ));

        // Generate a random value to return.
        let default_return = expr::gen_literal(u, &self.func.return_type, self.config)?;

        let limit_ident = expr::ident_inner(
            limit_var,
            self.next_ident_id(),
            false,
            LIMIT_NAME.to_string(),
            limit_type,
        );
        let limit_expr = Expression::Ident(limit_ident.clone());

        expr::replace(&mut self.func.body, |mut body| {
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
                self.func.return_type.clone(),
            )
        });

        Ok(())
    }

    /// For non-recursive functions just add an unused parameter.
    /// In non-main we look at the limit and return a random value if it's zero,
    /// otherwise decrease it by one and continue with the original body.
    fn modify_body_when_non_recursive(&mut self, limit_id: LocalId) {
        let limit_type = types::ref_mut(types::U32);
        self.func.parameters.push((
            limit_id,
            false,
            format!("_{LIMIT_NAME}"),
            limit_type.clone(),
            Visibility::Private,
        ));
    }

    /// Fill the body of a `func_{i}_proxy` with an expression to forward the call
    /// to the original function. Add the `ctx_parameter` as well.
    fn set_proxy_function(
        &mut self,
        limit_id: LocalId,
        proxy_functions: &mut HashMap<FuncId, Function>,
    ) {
        let Some(proxy) = proxy_functions.get_mut(&self.func_id) else {
            return;
        };

        proxy.parameters.push((
            limit_id,
            true,
            LIMIT_NAME.to_string(),
            types::U32,
            Visibility::Private,
        ));

        // The body is just a call the the non-proxy function.
        proxy.body = Expression::Call(Call {
            func: Box::new(Expression::Ident(Ident {
                location: None,
                definition: Definition::Function(self.func_id),
                mutable: false,
                name: self.func.name.clone(),
                typ: Type::Function(
                    self.func.parameters.iter().map(|p| p.3.clone()).collect(),
                    Box::new(self.func.return_type.clone()),
                    Box::new(Type::Unit),
                    self.func.unconstrained,
                ),
                id: self.next_ident_id(),
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
                                self.next_ident_id(),
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
                            self.next_ident_id(),
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

    /// Visit all the calls made by this function and pass along the limit.
    fn modify_calls(&mut self, limit_id: LocalId, proxy_functions: &HashMap<FuncId, Function>) {
        let limit_var = VariableId::Local(limit_id);

        // Swap out the body because we need mutable access to self in the visitor.
        let mut body = Expression::Break;
        std::mem::swap(&mut self.func.body, &mut body);

        // Update calls to pass along the limit and call the proxy if necessary.
        // Also find places where we are passing a function pointer, and change
        // it into the proxy version if necessary.
        visit_expr_mut(&mut body, &mut |expr: &mut Expression| {
            if let Expression::Call(call) = expr {
                let Expression::Ident(ident) = expr::unref_mut(call.func.as_mut()) else {
                    unreachable!("functions are called by ident; got {}", call.func);
                };

                let proxy = match &ident.definition {
                    Definition::Function(id) => proxy_functions.get(id),
                    Definition::Local(_) => {
                        // Doesn't have a proxy, but still needs its parameters adjusted.
                        None
                    }
                    Definition::Oracle(_) | Definition::Builtin(_) => {
                        // Oracles don't participate in recursion, let's leave them alone.
                        return true;
                    }
                    other => unreachable!("unexpected call target definition: {}", other),
                };

                let Type::Function(param_types, _, _, callee_unconstrained) =
                    types::unref_mut(&mut ident.typ)
                else {
                    unreachable!("function type expected");
                };

                if *callee_unconstrained && !self.func.unconstrained {
                    // Calling Brillig from ACIR: call the proxy if it's global.
                    if let Some(proxy) = proxy {
                        ident.name = proxy.name.clone();
                        ident.definition = Definition::Function(proxy.id);
                    }
                    // Pass the limit by value.
                    let limit_expr = if self.is_main {
                        expr::ident(
                            limit_var,
                            self.next_ident_id(),
                            true,
                            LIMIT_NAME.to_string(),
                            types::U32,
                        )
                    } else {
                        expr::deref(
                            expr::ident(
                                limit_var,
                                self.next_ident_id(),
                                false,
                                LIMIT_NAME.to_string(),
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
                    let limit_expr = if self.is_main {
                        // In main we take a mutable reference to the limit.
                        expr::ref_mut(
                            expr::ident(
                                limit_var,
                                self.next_ident_id(),
                                true,
                                LIMIT_NAME.to_string(),
                                types::U32,
                            ),
                            limit_type,
                        )
                    } else {
                        // In non-main we just pass along the parameter.
                        expr::ident(
                            limit_var,
                            self.next_ident_id(),
                            false,
                            LIMIT_NAME.to_string(),
                            limit_type,
                        )
                    };
                    param_types.push(types::U32);
                    call.arguments.push(limit_expr);
                }

                // Now go through all the parameters: if they are function pointer,
                // change the signature type of the parameter based on the caller.
                modify_function_pointer_param_types(param_types, *callee_unconstrained);

                // Go through the arguments of the call: if they point at a global
                // function, they might need to point at the proxy instead.
                modify_function_pointer_param_values(
                    &mut call.arguments,
                    param_types,
                    *callee_unconstrained,
                    proxy_functions,
                );
            }

            // Continue the visiting expressions.
            true
        });

        // Put the result back.
        std::mem::swap(&mut self.func.body, &mut body);
    }

    /// Update any function pointer and the function and its proxy's signature to take the limit.
    fn modify_function_pointer_param_types(
        &mut self,
        proxy_functions: &mut HashMap<FuncId, Function>,
    ) {
        for (_, _, _, param_type, _) in self.func.parameters.iter_mut() {
            modify_function_pointer_param_type(param_type, self.func.unconstrained);
        }
        if let Some(proxy) = proxy_functions.get_mut(&self.func_id) {
            for (_, _, _, param_type, _) in proxy.parameters.iter_mut() {
                modify_function_pointer_param_type(param_type, self.func.unconstrained);
            }
        }
    }
}

/// Go through the types of each function parameter. If they are function pointers,
/// then they need the context, depending on the callee type.
fn modify_function_pointer_param_types(param_types: &mut [Type], callee_unconstrained: bool) {
    for param_type in param_types.iter_mut() {
        modify_function_pointer_param_type(param_type, callee_unconstrained);
    }
}

/// Recursively modify function pointers in the param type.
fn modify_function_pointer_param_type(param_type: &mut Type, callee_unconstrained: bool) {
    let Type::Function(param_types, _, _, param_unconstrained) = types::unref_mut(param_type)
    else {
        return;
    };

    let limit_typ = ctx_limit_type_for_func_param(callee_unconstrained, *param_unconstrained);

    // Add the limit to the function described in the parameter.
    param_types.push(limit_typ);

    // We need to recurse into the parameters of the function pointer.
    modify_function_pointer_param_types(param_types, *param_unconstrained);
}

/// Go through the call arguments and update global function pointers to their
/// proxy equivalents if necessary.
fn modify_function_pointer_param_values(
    args: &mut [Expression],
    param_types: &[Type],
    callee_unconstrained: bool,
    proxy_functions: &HashMap<FuncId, Function>,
) {
    for i in 0..param_types.len() {
        // We only consider parameters that take functions, not function references,
        // because if something can take a function reference, and we can call it,
        // then it must be a Brillig to Brillig call, and we don't have to change
        // it to pass the proxy instead.
        let Type::Function(_, _, _, param_unconstrained) = &param_types[i] else {
            continue;
        };
        let limit_typ = ctx_limit_type_for_func_param(callee_unconstrained, *param_unconstrained);

        // If it's passed by reference we can leave it alone.
        if types::is_reference(&limit_typ) {
            continue;
        }

        // If we need to pass by value, then it's going to the proxy, but only if it's a global function,
        // and not a function parameter, which we wouldn't know what to change to, and doing so happens
        // when it's first passed as a global.
        let arg = &mut args[i];

        // If we are dereferencing a variable, then it's not a global function we are passing.
        if expr::is_deref(arg) {
            continue;
        }
        // Otherwise we should be passing a function by identifier directly.
        let Expression::Ident(param_func_ident) = arg else {
            unreachable!("functions are passed by ident; got {arg}");
        };
        let param_func_id = match &param_func_ident.definition {
            Definition::Function(id) => id,
            Definition::Local(_) => continue,
            other => {
                unreachable!("function definition expected; got {}", other);
            }
        };
        let Some(proxy) = proxy_functions.get(param_func_id) else {
            unreachable!(
                "expected to have a proxy for the function pointer: {param_func_id}; only have them for {:?}",
                proxy_functions.keys().collect::<Vec<_>>()
            );
        };
        param_func_ident.name = proxy.name.clone();
        param_func_ident.definition = Definition::Function(proxy.id);
    }
}
