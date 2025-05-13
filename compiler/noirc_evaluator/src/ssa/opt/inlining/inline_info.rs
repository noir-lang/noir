use std::collections::{BTreeMap, VecDeque};

use fxhash::FxHashSet as HashSet;
use im::HashMap;
use petgraph::graph::NodeIndex as PetGraphIndex;

use crate::ssa::{
    ir::{
        call_graph::{called_functions, called_functions_vec, CallGraph}, dfg::DataFlowGraph, function::{Function, FunctionId}
    },
    ssa_gen::Ssa,
};

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
    pub(crate) fn is_inline_target(&self, dfg: &DataFlowGraph) -> bool {
        self.is_brillig_entry_point
            || self.is_acir_entry_point
            // We still want to attempt inlining recursive ACIR functions in case
            // they have a compile-time completion point. 
            || (self.is_recursive && !dfg.runtime().is_acir())
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
///  - Any Acir functions with a [fold inline type][noirc_frontend::monomorphization::ast::InlineType],
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

    let call_graph = CallGraph::from_ssa(ssa);
    // Find mutual recursion in our call graph
    let recursive_functions = call_graph.get_recursive_functions();
    for recursive_func in recursive_functions.iter() {
        inline_infos.entry(*recursive_func).or_default().is_recursive = true;
        compute_function_should_be_inlined(
            ssa,
            inline_no_predicates_functions,
            aggressiveness,
            &times_called,
            &mut inline_infos,
            &call_graph,
            call_graph.ids_to_indices()[recursive_func],
        );
    }

    let acyclic_graph = call_graph.build_acyclic_subgraph(&recursive_functions);
    let topological_order = petgraph::algo::toposort(acyclic_graph.graph(), None).unwrap();

    // We need to reverse the topological sort as we want to process the weight of the leaves first,
    // as the weight of all callees will be used to compute a function's total weight.
    for index in topological_order.into_iter().rev() {
        compute_function_should_be_inlined(
            ssa,
            inline_no_predicates_functions,
            aggressiveness,
            &times_called,
            &mut inline_infos,
            &acyclic_graph,
            index,
        );
    }

    inline_infos
}

/// Determines whether a function should be inlined.
///
/// Inlining is determined by the following:
/// - the function is not recursive
/// - the cost of inlining outweighs the cost of not doing so
///
/// The total weight of a function and its cost are computed in this method.
/// The total weight is calculated by taking the function's own weight and multiplying
/// it by the weight of each callee. We then determine the cost of inlining to be
/// the times a function has been called multiplied by its total weight.
///
/// To determine the cost of retaining a function we first need the function interface cost,
/// computed in [compute_function_interface_cost].
/// The cost of retaining of a function is then (times a function has been called) * (interface cost) + total weight.
///
/// A function's net cost is then (cost of inlining - cost of retaining).
/// The net cost is then compared against the inliner aggressiveness setting. If the net cost is less than the aggressiveness,
/// we inline the function (granted there are not other restrictions such as recursion).
fn compute_function_should_be_inlined(
    ssa: &Ssa,
    inline_no_predicates_functions: bool,
    aggressiveness: i64,
    times_called: &HashMap<FunctionId, usize>,
    inline_infos: &mut InlineInfos,
    call_graph: &CallGraph,
    index: PetGraphIndex,
) {
    let func_id = call_graph.indices_to_ids()[&index];
    if inline_infos.get(&func_id).is_some_and(|info| info.should_inline || info.weight != 0) {
        return; // Already processed
    }

    let neighbors = call_graph.graph().neighbors(index);
    let mut total_weight = compute_function_own_weight(&ssa.functions[&func_id]) as i64;
    for neighbor_index in neighbors {
        let callee = call_graph.indices_to_ids()[&neighbor_index];
        if inline_infos.get(&callee).is_some_and(|info| info.should_inline) {
            total_weight = total_weight.saturating_add(inline_infos[&callee].weight);
        }
    }
    let times = times_called[&func_id] as i64;
    let interface_cost = compute_function_interface_cost(&ssa.functions[&func_id]) as i64;
    let inline_cost = times.saturating_mul(total_weight);
    let retain_cost = times.saturating_mul(interface_cost) + total_weight;
    let net_cost = inline_cost.saturating_sub(retain_cost);
    let runtime = ssa.functions[&func_id].runtime();
    let info = inline_infos.entry(func_id).or_default();

    let should_inline = ((net_cost < aggressiveness)
        || runtime.is_inline_always()
        || (runtime.is_no_predicates() && inline_no_predicates_functions))
        && !info.is_recursive;

    info.should_inline = should_inline;
    info.weight = total_weight;
    info.cost = net_cost;
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
    let mut visited = HashSet::default();

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

#[cfg(test)]
mod tests {
    use crate::ssa::{
        ir::map::Id,
        opt::inlining::inline_info::{compute_bottom_up_order, compute_callees, compute_callers},
        ssa_gen::Ssa,
    };

    use super::compute_inline_infos;

    #[test]
    fn mark_mutually_recursive_functions() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        brillig(inline) fn starter f1 {
          b0():
            call f2()
            return
        }
        brillig(inline) fn ping f2 {
          b0():
            call f3()
            return
        }
        brillig(inline) fn pong f3 {
          b0():
            call f2()
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let inline_infos = compute_inline_infos(&ssa, false, i64::MAX);

        let func_0 = inline_infos.get(&Id::test_new(0)).expect("Should have computed inline info");
        assert!(!func_0.is_recursive);

        let func_1 = inline_infos.get(&Id::test_new(1)).expect("Should have computed inline info");
        assert!(!func_1.is_recursive);

        let func_2 = inline_infos.get(&Id::test_new(2)).expect("Should have computed inline info");
        assert!(func_2.is_recursive);

        let func_3 = inline_infos.get(&Id::test_new(3)).expect("Should have computed inline info");
        assert!(func_3.is_recursive);
    }

    fn callers_and_callees_src() -> &'static str {
        r#"
        acir(inline) fn main f0 {
          b0():
            call f1()
            call f1()
            call f2()
            return
        }
        acir(inline) fn foo f1 {
          b0():
            call f3()
            return
        }
        brillig(inline) fn bar f2 {
          b0():
            call f3()
            call f4()
            call f4()
            return
        }
        brillig(inline) fn baz f3 {
          b0():
            return
        }
        brillig(inline) fn qux f4 {
          b0():
            call f3()
            return
        }
        "#
    }

    #[test]
    fn callers() {
        let ssa = Ssa::from_str(callers_and_callees_src()).unwrap();
        let callers = compute_callers(&ssa);

        let f0_callers = callers.get(&Id::test_new(0)).expect("Should have callers");
        assert!(f0_callers.is_empty());

        let f1_callers = callers.get(&Id::test_new(1)).expect("Should have callers");
        assert_eq!(f1_callers.len(), 1);
        let times_f1_called_by_f0 =
            *f1_callers.get(&Id::test_new(0)).expect("Should have times called");
        assert_eq!(times_f1_called_by_f0, 2);

        let f2_callers = callers.get(&Id::test_new(2)).expect("Should have callers");
        assert_eq!(f2_callers.len(), 1);
        let times_f2_called_by_f0 =
            *f2_callers.get(&Id::test_new(0)).expect("Should have times called");
        assert_eq!(times_f2_called_by_f0, 1);

        let f3_callers = callers.get(&Id::test_new(3)).expect("Should have callers");
        assert_eq!(f3_callers.len(), 3);
        let times_f3_called_by_f1 =
            *f3_callers.get(&Id::test_new(1)).expect("Should have times called");
        assert_eq!(times_f3_called_by_f1, 1);
        let times_f3_called_by_f2 =
            *f3_callers.get(&Id::test_new(2)).expect("Should have times called");
        assert_eq!(times_f3_called_by_f2, 1);
        let times_f3_called_by_f4 =
            *f3_callers.get(&Id::test_new(4)).expect("Should have times called");
        assert_eq!(times_f3_called_by_f4, 1);

        let f4_callers = callers.get(&Id::test_new(4)).expect("Should have callers");
        assert_eq!(f4_callers.len(), 1);
        let times_f4_called_by_f2 =
            *f4_callers.get(&Id::test_new(2)).expect("Should have times called");
        assert_eq!(times_f4_called_by_f2, 2);
    }

    #[test]
    fn callees() {
        let ssa = Ssa::from_str(callers_and_callees_src()).unwrap();
        let callees = compute_callees(&ssa);

        let f0_callees = callees.get(&Id::test_new(0)).expect("Should have callees");
        assert_eq!(f0_callees.len(), 2);
        let times_f0_calls_f1 =
            *f0_callees.get(&Id::test_new(1)).expect("Should have times called");
        assert_eq!(times_f0_calls_f1, 2);
        let times_f0_calls_f2 =
            *f0_callees.get(&Id::test_new(2)).expect("Should have times called");
        assert_eq!(times_f0_calls_f2, 1);

        let f1_callees = callees.get(&Id::test_new(1)).expect("Should have callees");
        assert_eq!(f1_callees.len(), 1);
        let times_f1_calls_f3 =
            *f1_callees.get(&Id::test_new(3)).expect("Should have times called");
        assert_eq!(times_f1_calls_f3, 1);

        let f2_callees = callees.get(&Id::test_new(2)).expect("Should have callees");
        assert_eq!(f2_callees.len(), 2);
        let times_f2_calls_f3 =
            *f2_callees.get(&Id::test_new(3)).expect("Should have times called");
        assert_eq!(times_f2_calls_f3, 1);
        let times_f2_calls_f4 =
            *f2_callees.get(&Id::test_new(4)).expect("Should have times called");
        assert_eq!(times_f2_calls_f4, 2);

        let f3_callees = callees.get(&Id::test_new(3)).expect("Should have callees");
        assert!(f3_callees.is_empty());

        let f4_callees = callees.get(&Id::test_new(4)).expect("Should have callees");
        let times_f4_calls_f3 =
            *f4_callees.get(&Id::test_new(3)).expect("Should have times called");
        assert_eq!(times_f4_calls_f3, 1);
    }

    #[test]
    fn bottom_up_order_and_weights() {
        let src = "
          brillig(inline) fn main f0 {
            b0(v0: u32, v1: u1):
              v3 = call f2(v0) -> u1
              v4 = eq v3, v1
              constrain v3 == v1
              return
          }
          brillig(inline) fn is_even f1 {
            b0(v0: u32):
              v3 = eq v0, u32 0
              jmpif v3 then: b2, else: b1
            b1():
              v5 = call f3(v0) -> u32
              v7 = call f2(v5) -> u1
              jmp b3(v7)
            b2():
              jmp b3(u1 1)
            b3(v1: u1):
              return v1
          }
          brillig(inline) fn is_odd f2 {
            b0(v0: u32):
              v3 = eq v0, u32 0
              jmpif v3 then: b2, else: b1
            b1():
              v5 = call f3(v0) -> u32
              v7 = call f1(v5) -> u1
              jmp b3(v7)
            b2():
              jmp b3(u1 0)
            b3(v1: u1):
              return v1
          }
          brillig(inline) fn decrement f3 {
            b0(v0: u32):
              v2 = sub v0, u32 1
              return v2
          }
        ";
        // main
        //   |
        //   V
        // is_odd <-> is_even
        //      |     |
        //      V     V
        //      decrement

        let ssa = Ssa::from_str(src).unwrap();
        let order = compute_bottom_up_order(&ssa);

        assert_eq!(order.len(), 4);
        let (ids, ws): (Vec<_>, Vec<_>) = order.into_iter().map(|(id, w)| (id.to_u32(), w)).unzip();
        let (ows, tws): (Vec<_>, Vec<_>) = ws.into_iter().unzip();

        // Check order
        assert_eq!(ids[0], 3, "decrement: first, it doesn't call anything");
        assert_eq!(ids[1], 1, "is_even: called by is_odd; removing first avoids cutting the graph");
        assert_eq!(ids[2], 2, "is_odd: called by is_odd and main");
        assert_eq!(ids[3], 0, "main: last, it's the entry");

        // Check own weights
        assert_eq!(ows, [2, 7, 7, 4]);

        // Check transitive weights
        assert_eq!(tws[0], ows[0], "decrement");
        assert_eq!(
            tws[1],
            ows[1] + // own
            tws[0] + // pushed from decrement
            (ows[2] + tws[0]), // pulled from is_odd at the time is_even is emitted
            "is_even"
        );
        assert_eq!(
            tws[2],
            ows[2] + // own
            tws[0] + // pushed from decrement
            tws[1], // pushed from is_even
            "is_odd"
        );
        assert_eq!(
            tws[3],
            ows[3] + // own
            tws[2], // pushed from is_odd
            "main"
        );
        assert!(tws[3] > std::cmp::max(tws[1], tws[2]), "ideally 'main' has the most weight");
    }
}
