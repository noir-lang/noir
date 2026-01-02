use std::collections::{BTreeMap, VecDeque};

use im::HashMap;
use petgraph::graph::NodeIndex as PetGraphIndex;
use rustc_hash::FxHashSet as HashSet;

use crate::ssa::{
    ir::{
        call_graph::{CallGraph, called_functions},
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic},
        value::Value,
    },
    ssa_gen::Ssa,
};

/// The maximum number of instructions chosen below is an expert estimation of a "small" function
/// in our SSA IR. Generally, inlining small functions with no control flow should enable further optimizations
/// in the compiler while avoiding code size bloat.
///
/// For example, a common "simple" function is writing into a mutable reference.
/// When that function has no control flow, it generally means we can expect all loads and stores within the
/// function to be resolved upon inlining. Inlining this type of basic function both reduces the number of
/// loads/stores to be executed and enables the compiler to continue optimizing at the inline site.
pub const MAX_INSTRUCTIONS: usize = 10;

/// Information about a function to aid the decision about whether to inline it or not.
/// The final decision depends on what we're inlining it into.
#[derive(Default, Debug)]
pub(crate) struct InlineInfo {
    is_brillig_entry_point: bool,
    is_acir_entry_point: bool,
    is_recursive: bool,
    pub(crate) should_inline: bool,
    pub(super) contains_static_assertion: bool,
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
            // A recursive function is going to set `should_inline` to false as well,
            // so we need to determine whether a function is an inline target 
            // with auxiliary runtime information.
            || ((self.is_recursive || !self.should_inline) && !dfg.runtime().is_acir())
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
    call_graph: &CallGraph,
    inline_no_predicates_functions: bool,
    small_function_max_instructions: usize,
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

    let times_called = call_graph.times_called();
    // Find mutual recursion in our call graph
    let recursive_functions = call_graph.get_recursive_functions();
    let small_function_max_instructions = small_function_max_instructions as i64;
    for recursive_func in recursive_functions.iter() {
        inline_infos.entry(*recursive_func).or_default().is_recursive = true;
        compute_function_should_be_inlined(
            ssa,
            inline_no_predicates_functions,
            small_function_max_instructions,
            aggressiveness,
            &times_called,
            &mut inline_infos,
            call_graph,
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
            small_function_max_instructions,
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
#[allow(clippy::too_many_arguments)]
fn compute_function_should_be_inlined(
    ssa: &Ssa,
    inline_no_predicates_functions: bool,
    small_function_max_instructions: i64,
    aggressiveness: i64,
    times_called: &HashMap<FunctionId, usize>,
    inline_infos: &mut InlineInfos,
    call_graph: &CallGraph,
    index: PetGraphIndex,
) {
    let func_id = call_graph.indices_to_ids()[&index];
    if inline_infos.get(&func_id).is_some_and(|info| {
        info.should_inline
            || info.weight != 0
            || info.is_brillig_entry_point
            || info.is_acir_entry_point
    }) {
        return; // Already processed
    }

    let function = &ssa.functions[&func_id];
    let runtime = function.runtime();

    if runtime.is_acir() {
        let info = inline_infos.entry(func_id).or_default();
        info.should_inline = true;
        return;
    }

    let is_recursive = inline_infos.get(&func_id).is_some_and(|info| info.is_recursive);
    if runtime.is_brillig() && is_recursive {
        return;
    }

    let assert_constant_id = function.dfg.get_intrinsic(Intrinsic::AssertConstant).copied();
    let static_assert_id = function.dfg.get_intrinsic(Intrinsic::StaticAssert).copied();

    let contains_static_assertion = if assert_constant_id.is_none() && static_assert_id.is_none() {
        false
    } else {
        function.reachable_blocks().iter().any(|block| {
            function.dfg[*block].instructions().iter().any(|instruction| {
                match &function.dfg[*instruction] {
                    Instruction::Call { func, .. } => {
                        Some(*func) == assert_constant_id || Some(*func) == static_assert_id
                    }
                    _ => false,
                }
            })
        })
    };

    let neighbors = call_graph.graph().neighbors(index);
    let mut total_weight = compute_function_own_weight(function) as i64;
    let instruction_weight = total_weight;
    for neighbor_index in neighbors {
        let callee = call_graph.indices_to_ids()[&neighbor_index];
        if inline_infos.get(&callee).is_some_and(|info| info.should_inline) {
            total_weight = total_weight.saturating_add(inline_infos[&callee].weight);
        }
    }
    let times = times_called[&func_id] as i64;
    let interface_cost = compute_function_interface_cost(function) as i64;
    let inline_cost = times.saturating_mul(total_weight);
    let retain_cost = times.saturating_mul(interface_cost) + total_weight;
    let net_cost = inline_cost.saturating_sub(retain_cost);
    let info = inline_infos.entry(func_id).or_default();

    info.contains_static_assertion = contains_static_assertion;
    info.weight = total_weight;
    info.cost = net_cost;

    let entry_block_id = function.entry_block();
    let entry_block = &function.dfg[entry_block_id];
    let should_inline_no_pred_function =
        runtime.is_no_predicates() && inline_no_predicates_functions;
    let is_simple_function = entry_block.successors().next().is_none()
        && instruction_weight < small_function_max_instructions;

    let should_inline = is_simple_function
        || net_cost < aggressiveness
        || runtime.is_inline_always()
        || should_inline_no_pred_function
        || contains_static_assertion
        || runtime.is_acir();

    info.should_inline = should_inline;
}

/// Compute something like a topological order of the functions, starting with the ones
/// that do not call any other functions, going towards the entry points. When cycles
/// are detected, take the one which are called by the most to break the ties.
///
/// This can be used to simplify the most often called functions first.
///
/// Returns the functions paired with their own as well as transitive weight,
/// which accumulates the weight of all the functions they call, as well as own.
pub(crate) fn compute_bottom_up_order(
    ssa: &Ssa,
    call_graph: &CallGraph,
) -> Vec<(FunctionId, (usize, usize))> {
    let mut order = Vec::new();
    let mut visited = HashSet::default();

    // Construct a new "call graph" which we'll repeatedly prune to find the "leaves".
    let mut callees = call_graph.callees();
    let callers = call_graph.callers();

    // Number of times a function is called, used to break cycles in the call graph by popping the next candidate.
    let mut times_called = call_graph.times_called().into_iter().collect::<Vec<_>>();
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
        for instruction in func.dfg[block_id].instructions() {
            weight += brillig_cost(*instruction, &func.dfg);
        }
        // TODO: We add one for the terminator. This can be improved as Jmp and Return must move their arguments
        weight += 1;
    }
    // We use an approximation of the average increase in instruction ratio from SSA to Brillig
    // In order to get the actual weight we'd need to codegen this function to brillig.
    weight
}

/// Computes a cost estimate of a basic block
/// WARNING: these are estimates of the runtime cost of each instruction,
/// These numbers can be improved.
fn brillig_cost(instruction: InstructionId, dfg: &DataFlowGraph) -> usize {
    match &dfg[instruction] {
        Instruction::Binary(binary) => {
            // TODO: various operations have different costs for unsigned/signed
            match binary.operator {
                BinaryOp::Add { unchecked } | BinaryOp::Sub { unchecked } => {
                    if unchecked {
                        3
                    } else {
                        7
                    }
                }
                BinaryOp::Mul { unchecked } => {
                    if unchecked {
                        3
                    } else {
                        8
                    }
                }
                // TODO: signed div/mod have different costs
                BinaryOp::Div => 1,
                BinaryOp::Mod => 3,
                BinaryOp::Eq => 1,
                // TODO: unsigned and signed lt have different costs
                BinaryOp::Lt => 5,
                BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => 1,
                // TODO: signed shl/shr have different costs
                BinaryOp::Shl | BinaryOp::Shr => 1,
            }
        }
        // A Cast can be either simplified, or lead to a truncate
        Instruction::Cast(_, _) => 3,
        Instruction::Not(_) => 1,
        Instruction::Truncate { .. } => 7,

        Instruction::Constrain(..) => {
            // TODO: could put estimate cost for static or dynamic message. Just checking static at the moment
            4
        }

        // TODO: Only implemented in ACIR, probably just error here but right we compute costs of all functions
        Instruction::ConstrainNotEqual(..) => 1,

        // TODO: look into how common this is in Brillig, just return one for now
        Instruction::RangeCheck { .. } => 1,

        Instruction::Call { func, arguments } => {
            match dfg[*func] {
                Value::Function(_) => {
                    let results = dfg.instruction_results(instruction);
                    5 + arguments.len() + results.len()
                }
                Value::ForeignFunction(_) => {
                    // TODO: we should differentiate inputs/outputs with array and vector allocations
                    1
                }
                Value::Intrinsic(intrinsic) => {
                    match intrinsic {
                        Intrinsic::ArrayLen => 1,
                        Intrinsic::AsSlice => {
                            10 // mem copy
                            + 8 // vector and array pointer init
                            + 2 // size registers
                        }
                        Intrinsic::BlackBox(_) => {
                            // TODO: we could differentiate inputs/outputs with array and vector inputs (we add one to the pointer)
                            1
                        }
                        Intrinsic::FieldLessThan => 1,
                        _ => 1,
                    }
                }

                Value::Instruction { .. }
                | Value::Param { .. }
                | Value::NumericConstant { .. }
                | Value::Global(_) => {
                    unreachable!("unsupported function call type {:?}", dfg[*func])
                }
            }
        }

        Instruction::Allocate | Instruction::Load { .. } | Instruction::Store { .. } => 1,

        Instruction::ArraySet { .. } => {
            // NOTE: Assumes that the RC is one
            7
        }
        Instruction::ArrayGet { .. } => 1,
        // if less than 10 elements, it is translated into a store for each element
        // if more than 10, it is a loop, so 20 should be a good estimate, worst case being 10 stores and ~10 index increments
        Instruction::MakeArray { .. } => 20,

        Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. } => 3,

        Instruction::EnableSideEffectsIf { .. } | Instruction::Noop => 0,
        // TODO: this is only true for non array values
        Instruction::IfElse { .. } => 1,
    }
}

/// Compute interface cost of a function based on the number of inputs and outputs.
fn compute_function_interface_cost(func: &Function) -> usize {
    func.parameters().len() + func.returns().unwrap_or_default().len()
}

#[cfg(test)]
mod tests {
    use crate::ssa::{
        ir::{call_graph::CallGraph, map::Id},
        opt::inlining::{MAX_INSTRUCTIONS, inline_info::compute_bottom_up_order},
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
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let inline_infos =
            compute_inline_infos(&ssa, &call_graph, false, MAX_INSTRUCTIONS, i64::MAX);

        let func_0 = inline_infos.get(&Id::test_new(0)).expect("Should have computed inline info");
        assert!(!func_0.is_recursive);

        let func_1 = inline_infos.get(&Id::test_new(1)).expect("Should have computed inline info");
        assert!(!func_1.is_recursive);

        let func_2 = inline_infos.get(&Id::test_new(2)).expect("Should have computed inline info");
        assert!(func_2.is_recursive);

        let func_3 = inline_infos.get(&Id::test_new(3)).expect("Should have computed inline info");
        assert!(func_3.is_recursive);
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
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let order = compute_bottom_up_order(&ssa, &call_graph);

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

    #[test]
    fn mark_static_assertions_to_always_be_inlined() {
        let src = "
        brillig(inline) fn main f0 {
            b0():
              call f1(Field 1)
              return
        }
        brillig(inline) fn foo f1 {
            b0(v0: Field):
              call assert_constant(v0)
              return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let infos = compute_inline_infos(&ssa, &call_graph, false, MAX_INSTRUCTIONS, 0);

        let f1 = infos.get(&Id::test_new(1)).expect("f1 should be analyzed");
        assert!(
            f1.contains_static_assertion,
            "f1 should be marked as containing a static assertion"
        );
        assert!(f1.should_inline, "f1 should be inlined due to static assertion");
    }

    #[test]
    fn no_predicates() {
        let src = "
        acir(inline) fn main f0 {
            b0():
              call f1()
              return
        }
        acir(no_predicates) fn no_predicates f1 {
            b0():
              return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let infos = compute_inline_infos(&ssa, &call_graph, false, MAX_INSTRUCTIONS, i64::MIN);
        let f1 = infos.get(&Id::test_new(1)).expect("Should analyze f1");
        assert!(
            !f1.should_inline,
            "no_predicates functions should NOT be inlined if the flag is false"
        );

        let infos = compute_inline_infos(&ssa, &call_graph, false, MAX_INSTRUCTIONS, 0);
        let f1 = infos.get(&Id::test_new(1)).expect("Should analyze f1");
        assert!(
            !f1.should_inline,
            "no_predicates functions should NOT be inlined if the flag is false"
        );

        let infos = compute_inline_infos(&ssa, &call_graph, false, MAX_INSTRUCTIONS, i64::MAX);
        let f1 = infos.get(&Id::test_new(1)).expect("Should analyze f1");
        assert!(
            !f1.should_inline,
            "no_predicates functions should NOT be inlined if the flag is false"
        );

        let infos = compute_inline_infos(&ssa, &call_graph, true, MAX_INSTRUCTIONS, i64::MIN);
        let f1 = infos.get(&Id::test_new(1)).expect("Should analyze f1");
        assert!(f1.should_inline, "no_predicates functions should be inlined if the flag is true");

        let infos = compute_inline_infos(&ssa, &call_graph, true, MAX_INSTRUCTIONS, 0);
        let f1 = infos.get(&Id::test_new(1)).expect("Should analyze f1");
        assert!(f1.should_inline, "no_predicates functions should be inlined if the flag is true");

        let infos = compute_inline_infos(&ssa, &call_graph, true, MAX_INSTRUCTIONS, i64::MAX);
        let f1 = infos.get(&Id::test_new(1)).expect("Should analyze f1");
        assert!(f1.should_inline, "no_predicates functions should be inlined if the flag is true");
    }

    #[test]
    fn inline_always_functions_are_inlined() {
        let src = "
        brillig(inline) fn main f0 {
            b0():
              call f1()
              return
        }

        brillig(inline_always) fn always_inline f1 {
            b0():
              return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let infos = compute_inline_infos(&ssa, &call_graph, false, MAX_INSTRUCTIONS, 0);

        let f1 = infos.get(&Id::test_new(1)).expect("Should analyze f1");
        assert!(f1.should_inline, "inline_always functions should be inlined");
    }

    #[test]
    fn basic_inlining_brillig_not_inlined_into_acir() {
        let src = "
        acir(inline) fn foo f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }
        brillig(inline) fn bar f1 {
          b0():
            return Field 72
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_graph = CallGraph::from_ssa_weighted(&ssa);
        let infos = compute_inline_infos(&ssa, &call_graph, false, MAX_INSTRUCTIONS, 0);

        let f1 = infos.get(&Id::test_new(1)).expect("Should analyze f1");
        assert!(!f1.should_inline, "Brillig entry points should never be inlined");
    }
}
