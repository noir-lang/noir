use std::collections::{BTreeSet, VecDeque};

use im::HashMap;
use noirc_frontend::monomorphization::{
    ast::{Definition, Expression, FuncId, Ident, Program},
    visitor::visit_expr_mut,
};

use crate::{expr, program::Context};

/// Remove functions that are unreachable from main.
pub(crate) fn remove_unreachable_functions(ctx: &mut Context) {
    if ctx.functions.is_empty() {
        return;
    }
    let reachable = find_reachable_functions(ctx);

    // We have to re-assign function IDs, because in the `Program`
    // the ID of the function is expected to match its position.
    let remap = reachable
        .into_iter()
        .enumerate()
        .map(|(i, id)| (id, FuncId(i as u32)))
        .collect::<HashMap<_, _>>();

    let functions = std::mem::take(&mut ctx.functions);
    let function_declarations = std::mem::take(&mut ctx.function_declarations);

    // Keep only the reachable ones.
    ctx.functions = functions
        .into_iter()
        .filter_map(|(id, mut func)| {
            remap.get(&id).map(|new_id| {
                func.id = *new_id;
                (*new_id, func)
            })
        })
        .collect();

    ctx.function_declarations = function_declarations
        .into_iter()
        .filter_map(|(id, func)| remap.get(&id).map(|new_id| (*new_id, func)))
        .collect();

    // Remap the old IDs to the new ones wherever the functions are referenced.
    for func in ctx.functions.values_mut() {
        visit_expr_mut(&mut func.body, &mut |expr| {
            if let Expression::Ident(Ident { definition: Definition::Function(id), .. }) = expr {
                let new_id = remap[id];
                *id = new_id;
            }
            true
        });
    }
}

/// Find functions reachable from main.
fn find_reachable_functions(ctx: &Context) -> BTreeSet<FuncId> {
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::new();

    // Start from main.
    queue.push_back(Program::main_id());

    // Find all global functions referred to by their identifier.
    while let Some(id) = queue.pop_front() {
        if !reachable.insert(id) {
            continue;
        }
        let func = &ctx.functions[&id];

        for id in expr::reachable_functions(&func.body) {
            queue.push_back(id);
        }
    }

    reachable
}
