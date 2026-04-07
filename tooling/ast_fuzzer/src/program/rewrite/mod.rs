use std::rc::Rc;

use nargo::errors::Location;
use noirc_frontend::{
    monomorphization::{
        ast::{
            Call, Definition, Expression, FuncId, Function, Ident, IdentId, InlineType, LocalId,
            Program, Type,
        },
        visitor::{visit_expr, visit_expr_mut},
    },
    shared::Visibility,
};

use super::{Context, expr, types};

mod limit;
mod unreachable;

pub(crate) use limit::add_recursion_limit;
pub(crate) use unreachable::remove_unreachable_functions;

/// Find the next local ID and ident IDs (in that order) that we can use to add
/// variables to a [Function] during mutations.
///
/// A more sophisticated alternative would be to return metadata along with the `Program`
/// that contains these values, so we don't have to traverse the AST again, and keep
/// the logic of what introduces new IDs in sync.
pub fn next_local_and_ident_id(func: &Function) -> (u32, u32) {
    let mut next_local_id = func.parameters.iter().map(|p| p.0.0 + 1).max().unwrap_or_default();
    let mut next_ident_id = 0;

    let mut acc_local_id = |id: LocalId| {
        next_local_id = next_local_id.max(id.0 + 1);
    };

    visit_expr(&func.body, &mut |expr| {
        match expr {
            Expression::Ident(ident) => {
                next_ident_id = next_ident_id.max(ident.id.0 + 1);
            }
            Expression::Let(let_) => acc_local_id(let_.id),
            Expression::For(for_) => acc_local_id(for_.index_variable),
            Expression::Match(match_) => {
                for case in &match_.cases {
                    for (id, _) in &case.arguments {
                        acc_local_id(*id);
                    }
                }
            }
            _ => {}
        }
        true
    });
    (next_local_id, next_ident_id)
}

/// Turn all ACIR functions into Brillig functions.
///
/// This is more involved than flipping the `unconstrained` property because of the
/// "ownership analysis", which can only run on a function once.
///
/// The function also takes care of changing all function pointers into unconstrained ones.
pub fn change_all_functions_into_unconstrained(mut program: Program) -> Program {
    for f in &mut program.functions {
        if f.unconstrained {
            continue;
        }
        // Modify the function.
        f.unconstrained = true;
        // Modify any function pointers it takes.
        for (_, _, _, typ, _) in &mut f.parameters {
            types::unref_mut_rc(typ, |unref_mut_typ| {
                if let Type::Function(args, ret, env, _unconstrained) = unref_mut_typ {
                    Type::Function(args, ret, env, true)
                } else {
                    unref_mut_typ
                }
            });
        }
        // Modify the calls it makes (we don't call ACIR from Brillig).
        visit_expr_mut(&mut f.body, &mut |expr| {
            let Expression::Call(Call { func, .. }) = expr else {
                return true;
            };
            let Expression::Ident(Ident { typ, .. }) = expr::unref_mut(func.as_mut()) else {
                unreachable!("functions are expected to be called by ident; got {func}");
            };

            types::unref_mut_rc(typ, |unref_mut_typ| {
                let Type::Function(args, ret, env, _unconstrained) = unref_mut_typ else {
                    unreachable!(
                        "function idents are expected to have Function type; got {unref_mut_typ}"
                    );
                };
                Type::Function(args, ret, env, true)
            });
            true
        });
        f.handle_ownership();
    }
    program
}

/// Wrap the fuzzer's direct oracle print calls in wrapper functions.
///
/// In nargo-compiled code, `println` goes through wrapper functions
/// (`println` -> `print_unconstrained` -> oracle). The fuzzer instead
/// generates direct oracle calls, which hits a compiler optimization:
/// SSA codegen skips `Clone` (and therefore `inc_rc`) for oracle call
/// arguments, since oracles cannot modify their inputs. However, the
/// corresponding `Drop`/`dec_rc` is still emitted, so the reference
/// count ends up lower than ownership analysis intended. This breaks
/// Brillig's copy-on-write invariant and can corrupt array values.
///
/// Adding wrapper functions around the oracle calls matches nargo's
/// structure, so `Clone`/`inc_rc` is preserved at the call site.
///
/// Found via seed `0x6a98890f00100000` in `comptime_vs_brillig_direct` at commit `c09ce9a7db`.
pub(crate) fn wrap_oracle_prints_in_functions(ctx: &mut Context) {
    let func_ids: Vec<FuncId> = ctx.functions.keys().copied().collect();
    let mut next_func_id = ctx.functions.len() as u32;
    let mut wrappers = vec![];

    for &func_id in &func_ids {
        let func = ctx.functions.get_mut(&func_id).unwrap();
        visit_expr_mut(&mut func.body, &mut |e| {
            // Clone to release the borrow on `e` so we can mutate it below.
            let info =
                oracle_print_info(e).map(|(vt, nl, ti)| (vt.clone(), nl.clone(), ti.to_vec()));
            let Some((value_type, newline, type_info)) = info else { return true };
            // Skip function pointers: no copy-on-write, and their Tuple args
            // flatten to multiple SSA values which wouldn't match one parameter.
            if types::is_function(&value_type) {
                return true;
            }

            let wrapper_id = FuncId(next_func_id);
            next_func_id += 1;
            wrappers.push(make_print_wrapper(wrapper_id, &value_type, newline, &type_info));
            replace_with_wrapper_call(e, wrapper_id, value_type);
            true
        });
    }

    for w in wrappers {
        ctx.functions.insert(w.id, w);
    }
}

/// Retarget an oracle print call to a wrapper function, keeping only the value argument.
fn replace_with_wrapper_call(e: &mut Expression, wrapper_id: FuncId, value_type: Type) {
    let Expression::Call(call) = e else { unreachable!() };
    let Expression::Ident(ident) = call.func.as_mut() else { unreachable!() };
    ident.definition = Definition::Function(wrapper_id);
    ident.name = format!("print_wrapper_{}", wrapper_id.0);
    ident.typ =
        Rc::new(Type::Function(vec![value_type], Rc::new(Type::Unit), Rc::new(Type::Unit), true));
    // Keep only the value argument (index 1); drop newline and type info.
    let value_arg = call.arguments.remove(1);
    call.arguments = vec![value_arg];
}

/// If `expr` is a `print` oracle call, return the value type, newline flag, and type-info arguments.
fn oracle_print_info(expr: &Expression) -> Option<(&Type, &Expression, &[Expression])> {
    let Expression::Call(Call { func, arguments, .. }) = expr else { return None };
    let Expression::Ident(Ident { definition: Definition::Oracle(name), typ, .. }) = func.as_ref()
    else {
        return None;
    };
    if name != "print" {
        return None;
    }
    let Type::Function(params, ..) = typ.as_ref() else { return None };
    let value_type = params.get(1)?;
    Some((value_type, &arguments[0], &arguments[2..]))
}

/// Build a wrapper: `unconstrained fn(value: T) -> () { print_oracle(newline, value, ...) }`
fn make_print_wrapper(
    id: FuncId,
    value_type: &Type,
    newline: Expression,
    type_info_args: &[Expression],
) -> Function {
    let value_type_rc = Rc::new(value_type.clone());

    let value_ident = Ident {
        location: None,
        definition: Definition::Local(LocalId(0)),
        mutable: false,
        name: "value".to_string(),
        typ: Rc::clone(&value_type_rc),
        id: IdentId(1),
    };

    let oracle_ident = Ident {
        location: None,
        definition: Definition::Oracle("print".to_string()),
        mutable: false,
        name: "print_oracle".to_string(),
        typ: Rc::new(Type::Function(
            vec![Type::Bool, value_type.clone()],
            Rc::new(Type::Unit),
            Rc::new(Type::Unit),
            true,
        )),
        id: IdentId(0),
    };

    let mut oracle_args = vec![newline, Expression::Ident(value_ident)];
    oracle_args.extend_from_slice(type_info_args);

    Function {
        id,
        name: format!("print_wrapper_{}", id.0),
        parameters: vec![(
            LocalId(0),
            false,
            "value".to_string(),
            value_type_rc,
            Visibility::Private,
        )],
        body: Expression::Call(Call {
            func: Box::new(Expression::Ident(oracle_ident)),
            arguments: oracle_args,
            return_type: Type::Unit,
            location: Location::dummy(),
        }),
        return_type: Type::Unit,
        return_visibility: Visibility::Private,
        unconstrained: true,
        inline_type: InlineType::default(),
        is_entry_point: false,
    }
}
