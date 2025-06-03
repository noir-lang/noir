use noirc_frontend::monomorphization::ast::{Call, Expression, Function, Ident, Program, Type};

use super::{
    expr, types,
    visitor::{visit_expr, visit_expr_mut},
};

mod limit;

pub(crate) use limit::add_recursion_limit;

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
///
/// The function also takes care of changing all function pointers into unconstrained ones.
pub fn change_all_functions_into_unconstrained(mut program: Program) -> Program {
    for f in program.functions.iter_mut() {
        if f.unconstrained {
            continue;
        }
        // Modify the function.
        f.unconstrained = true;
        // Modify any function pointers it takes.
        for (_, _, _, typ, _) in f.parameters.iter_mut() {
            if let Type::Function(_, _, _, unconstrained) = types::unref_mut(typ) {
                *unconstrained = true;
            }
        }
        // Modify the calls it makes (we don't call ACIR from Brillig).
        visit_expr_mut(&mut f.body, &mut |expr| {
            let Expression::Call(Call { func, .. }) = expr else {
                return true;
            };
            let Expression::Ident(Ident { typ, .. }) = expr::unref_mut(func.as_mut()) else {
                unreachable!("functions are expected to be called by ident; got {func}");
            };
            let Type::Function(_, _, _, unconstrained) = types::unref_mut(typ) else {
                unreachable!("function idents are expected to have Function type; got {typ}");
            };
            *unconstrained = true;
            true
        });
        f.handle_ownership();
    }
    program
}
