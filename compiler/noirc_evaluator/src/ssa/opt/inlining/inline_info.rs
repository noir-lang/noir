use std::collections::{BTreeMap, HashSet, VecDeque};

use im::HashMap;

use crate::ssa::{
    ir::function::{Function, FunctionId},
    ssa_gen::Ssa,
};

use super::{called_functions, called_functions_vec};

/// Information about a function to aid the decision about whether to inline it or not.
/// The final decision depends on what we're inlining it into.
#[derive(Default, Debug)]
pub(crate) struct InlineInfo {
    is_brillig_entry_point: bool,
    is_acir_entry_point: bool,
    is_recursive: bool,
    pub(crate) should_inline: bool,
    weight: i64,
    cost: i64,
}

impl InlineInfo {
    /// Functions which are to be retained, not inlined.
    pub(crate) fn is_inline_target(&self) -> bool {
        self.is_brillig_entry_point
            || self.is_acir_entry_point
            || self.is_recursive
            || !self.should_inline
    }

    pub(crate) fn should_inline(inline_infos: &InlineInfos, called_func_id: FunctionId) -> bool {
        inline_infos.get(&called_func_id).map(|info| info.should_inline).unwrap_or_default()
    }
}

pub(crate) type InlineInfos = BTreeMap<FunctionId, InlineInfo>;

/// The functions we should inline into (and that should be left in the final program) are:
///  - main
///  - Any Brillig function called from Acir
///  - Some Brillig functions depending on aggressiveness and some metrics
///  - Any Acir functions with a [fold inline type][InlineType::Fold],
///
/// The returned `InlineInfos` won't have every function in it, only the ones which the algorithm visited.
pub(crate) fn compute_inline_infos(
    ssa: &Ssa,
    inline_no_predicates_functions: bool,
    aggressiveness: i64,
) -> InlineInfos {
    let mut inline_infos = InlineInfos::default();

    inline_infos.insert(
        ssa.main_id,
        InlineInfo {
            is_acir_entry_point: ssa.main().runtime().is_acir(),
            is_brillig_entry_point: ssa.main().runtime().is_brillig(),
            ..Default::default()
        },
    );

    // Handle ACIR functions.
    for (func_id, function) in ssa.functions.iter() {
        if function.runtime().is_brillig() {
            continue;
        }

        // If we have not already finished the flattening pass, functions marked
        // to not have predicates should be preserved.
        let preserve_function = !inline_no_predicates_functions && function.is_no_predicates();
        if function.runtime().is_entry_point() || preserve_function {
            inline_infos.entry(*func_id).or_default().is_acir_entry_point = true;
        }

        // Any Brillig function called from ACIR is an entry into the Brillig VM.
        for called_func_id in called_functions(function) {
            if ssa.functions[&called_func_id].runtime().is_brillig() {
                inline_infos.entry(called_func_id).or_default().is_brillig_entry_point = true;
            }
        }
    }

    let callers = compute_callers(ssa);
    let times_called = compute_times_called(&callers);

    mark_brillig_functions_to_retain(
        ssa,
        inline_no_predicates_functions,
        aggressiveness,
        &times_called,
        &mut inline_infos,
    );

    inline_infos
}

/// Compute the time each function is called from any other function.
fn compute_times_called(
    callers: &BTreeMap<FunctionId, BTreeMap<FunctionId, usize>>,
) -> HashMap<FunctionId, usize> {
    callers
        .iter()
        .map(|(callee, callers)| {
            let total_calls = callers.values().sum();
            (*callee, total_calls)
        })
        .collect()
}

/// Compute for each function the set of functions that call it, and how many times they do so.
fn compute_callers(ssa: &Ssa) -> BTreeMap<FunctionId, BTreeMap<FunctionId, usize>> {
    ssa.functions
        .iter()
        .flat_map(|(caller_id, function)| {
            let called_functions = called_functions_vec(function);
            called_functions.into_iter().map(|callee_id| (*caller_id, callee_id))
        })
        .fold(
            // Make sure an entry exists even for ones that don't get called.
            ssa.functions.keys().map(|id| (*id, BTreeMap::new())).collect(),
            |mut acc, (caller_id, callee_id)| {
                let callers = acc.entry(callee_id).or_default();
                *callers.entry(caller_id).or_default() += 1;
                acc
            },
        )
}

/// Compute for each function the set of functions called by it, and how many times it does so.
fn compute_callees(ssa: &Ssa) -> BTreeMap<FunctionId, BTreeMap<FunctionId, usize>> {
    ssa.functions
        .iter()
        .flat_map(|(caller_id, function)| {
            let called_functions = called_functions_vec(function);
            called_functions.into_iter().map(|callee_id| (*caller_id, callee_id))
        })
        .fold(
            // Make sure an entry exists even for ones that don't call anything.
            ssa.functions.keys().map(|id| (*id, BTreeMap::new())).collect(),
            |mut acc, (caller_id, callee_id)| {
                let callees = acc.entry(caller_id).or_default();
                *callees.entry(callee_id).or_default() += 1;
                acc
            },
        )
}

/// Compute something like a topological order of the functions, starting with the ones
/// that do not call any other functions, going towards the entry points. When cycles
/// are detected, take the one which are called by the most to break the ties.
///
/// This can be used to simplify the most often called functions first.
///
/// Returns the functions paired with their own as well as transitive weight,
/// which accumulates the weight of all the functions they call, as well as own.
pub(crate) fn compute_bottom_up_order(ssa: &Ssa) -> Vec<(FunctionId, (usize, usize))> {
    let mut order = Vec::new();
    let mut visited = HashSet::new();

    // Call graph which we'll repeatedly prune to find the "leaves".
    let mut callees = compute_callees(ssa);
    let callers = compute_callers(ssa);

    // Number of times a function is called, used to break cycles in the call graph by popping the next candidate.
    let mut times_called = compute_times_called(&callers).into_iter().collect::<Vec<_>>();
    times_called.sort_by_key(|(id, cnt)| {
        // Sort by called the *least* by others, as these are less likely to cut the graph when removed.
        let called_desc = -(*cnt as i64);
        // Sort entries first (last to be popped).
        let is_entry_asc = -called_desc.signum();
        // Finally break ties by ID.
        (is_entry_asc, called_desc, *id)
    });

    // Start with the weight of the functions in isolation, then accumulate as we pop off the ones they call.
    let own_weights = ssa
        .functions
        .iter()
        .map(|(id, f)| (*id, compute_function_own_weight(f)))
        .collect::<HashMap<_, _>>();
    let mut weights = own_weights.clone();

    // Seed the queue with functions that don't call anything.
    let mut queue = callees
        .iter()
        .filter_map(|(id, callees)| callees.is_empty().then_some(*id))
        .collect::<VecDeque<_>>();

    loop {
        while let Some(id) = queue.pop_front() {
            // Pull the current weight of yet-to-be emitted callees (a nod to mutual recursion).
            for (callee, cnt) in &callees[&id] {
                if *callee != id {
                    weights[&id] = weights[&id].saturating_add(cnt.saturating_mul(weights[callee]));
                }
            }
            // Own weight plus the weights accumulated from callees.
            let weight = weights[&id];
            let own_weight = own_weights[&id];

            // Emit the function.
            order.push((id, (own_weight, weight)));
            visited.insert(id);

            // Update the callers of this function.
            for (caller, cnt) in &callers[&id] {
                // Update the weight of the caller with the weight of this function.
                weights[caller] = weights[caller].saturating_add(cnt.saturating_mul(weight));
                // Remove this function from the callees of the caller.
                let callees = callees.get_mut(caller).unwrap();
                callees.remove(&id);
                // If the caller doesn't call any other function, enqueue it,
                // unless it's the entry function, which is never called by anything, so it should be last.
                if callees.is_empty() && !visited.contains(caller) && !callers[caller].is_empty() {
                    queue.push_back(*caller);
                }
            }
        }
        // If we ran out of the queue, maybe there is a cycle; take the next most called function.
        while let Some((id, _)) = times_called.pop() {
            if !visited.contains(&id) {
                queue.push_back(id);
                break;
            }
        }
        if times_called.is_empty() && queue.is_empty() {
            assert_eq!(order.len(), callers.len());
            return order;
        }
    }
}

/// Compute a weight of a function based on the number of instructions in its reachable blocks.
fn compute_function_own_weight(func: &Function) -> usize {
    let mut weight = 0;
    for block_id in func.reachable_blocks() {
        weight += func.dfg[block_id].instructions().len() + 1; // We add one for the terminator
    }
    // We use an approximation of the average increase in instruction ratio from SSA to Brillig
    // In order to get the actual weight we'd need to codegen this function to brillig.
    weight
}

/// Compute interface cost of a function based on the number of inputs and outputs.
fn compute_function_interface_cost(func: &Function) -> usize {
    func.parameters().len() + func.returns().len()
}

/// Traverse the call graph starting from a given function, marking function to be retained if they are:
/// * recursive functions, or
/// * the cost of inlining outweighs the cost of not doing so
fn mark_functions_to_retain_recursive(
    ssa: &Ssa,
    inline_no_predicates_functions: bool,
    aggressiveness: i64,
    times_called: &HashMap<FunctionId, usize>,
    inline_infos: &mut InlineInfos,
    mut explored_functions: im::HashSet<FunctionId>,
    func: FunctionId,
) {
    // Check if we have set any of the fields this method touches.
    let decided = |inline_infos: &InlineInfos| {
        inline_infos
            .get(&func)
            .map(|info| info.is_recursive || info.should_inline || info.weight != 0)
            .unwrap_or_default()
    };

    // Check if we have already decided on this function
    if decided(inline_infos) {
        return;
    }

    // If recursive, this function won't be inlined
    if explored_functions.contains(&func) {
        inline_infos.entry(func).or_default().is_recursive = true;
        return;
    }
    explored_functions.insert(func);

    // Decide on dependencies first, so we know their weight.
    let called_functions = called_functions_vec(&ssa.functions[&func]);
    for callee in &called_functions {
        mark_functions_to_retain_recursive(
            ssa,
            inline_no_predicates_functions,
            aggressiveness,
            times_called,
            inline_infos,
            explored_functions.clone(),
            *callee,
        );
    }

    // We could have decided on this function while deciding on dependencies
    // if the function is recursive.
    if decided(inline_infos) {
        return;
    }

    // We'll use some heuristics to decide whether to inline or not.
    // We compute the weight (roughly the number of instructions) of the function after inlining
    // And the interface cost of the function (the inherent cost at the callsite, roughly the number of args and returns)
    // We then can compute an approximation of the cost of inlining vs the cost of retaining the function
    // We do this computation using saturating i64s to avoid overflows,
    // and because we want to calculate a difference which can be negative.

    // Total weight of functions called by this one, unless we decided not to inline them.
    // Callees which appear multiple times would be inlined multiple times.
    let inlined_function_weights: i64 = called_functions.iter().fold(0, |acc, callee| {
        let info = &inline_infos[callee];
        // If the callee is not going to be inlined then we can ignore its cost.
        if info.should_inline {
            acc.saturating_add(info.weight)
        } else {
            acc
        }
    });

    let this_function_weight = inlined_function_weights
        .saturating_add(compute_function_own_weight(&ssa.functions[&func]) as i64);

    let interface_cost = compute_function_interface_cost(&ssa.functions[&func]) as i64;

    let times_called = times_called[&func] as i64;

    let inline_cost = times_called.saturating_mul(this_function_weight);
    let retain_cost = times_called.saturating_mul(interface_cost) + this_function_weight;
    let net_cost = inline_cost.saturating_sub(retain_cost);

    let runtime = ssa.functions[&func].runtime();
    // We inline if the aggressiveness is higher than inline cost minus the retain cost
    // If aggressiveness is infinite, we'll always inline
    // If aggressiveness is 0, we'll inline when the inline cost is lower than the retain cost
    // If aggressiveness is minus infinity, we'll never inline (other than in the mandatory cases)
    let should_inline = (net_cost < aggressiveness)
        || runtime.is_inline_always()
        || (runtime.is_no_predicates() && inline_no_predicates_functions);

    let info = inline_infos.entry(func).or_default();
    info.should_inline = should_inline;
    info.weight = this_function_weight;
    info.cost = net_cost;
}

/// Mark Brillig functions that should not be inlined because they are recursive or expensive.
fn mark_brillig_functions_to_retain(
    ssa: &Ssa,
    inline_no_predicates_functions: bool,
    aggressiveness: i64,
    times_called: &HashMap<FunctionId, usize>,
    inline_infos: &mut InlineInfos,
) {
    let brillig_entry_points = inline_infos
        .iter()
        .filter_map(|(id, info)| info.is_brillig_entry_point.then_some(*id))
        .collect::<Vec<_>>();

    for entry_point in brillig_entry_points {
        mark_functions_to_retain_recursive(
            ssa,
            inline_no_predicates_functions,
            aggressiveness,
            times_called,
            inline_infos,
            im::HashSet::default(),
            entry_point,
        );
    }
}
