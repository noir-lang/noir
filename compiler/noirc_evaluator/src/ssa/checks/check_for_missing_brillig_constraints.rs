//! This module defines the [Ssa::check_for_missing_brillig_constraints] method.
//!
//! It verifies that the output of Brillig calls is connected to the inputs of the calls
//! by assertions; in other words, that the circuit has constraints that the output is
//! correct, given the inputs.
//!
//! To do so, it tracks the ancestry of every expression, and checks that any
//! variable which is an output of a Brillig call has a descendant which appears
//! in an assertion, where the other side has an ancestor that is an input of the call.
//!
//! Essentially, to consider a particular Brillig call constrained, we are looking
//! for a constraint where the ancestors of the constraint arguments intersect both of:
//! * the descendants of the results of the call (outputs)
//! * the ancestors of the arguments of the call (inputs)
//!
//! For example take the following graph of variables feeding into calls:
//! ```text
//!   v1     v2      v3
//!    \   /  \    /
//!     \ /    \  /
//!      v4     v5 = call(v2, v3)
//!      |\     |
//!      | \    |
//!      |  \   |
//!      |   \  |
//!      |    \ |
//!      |      v6 = call(v5, v4)
//!      |     /
//!      |    /
//!      |   /
//!      |  /
//! constrain(v4, v6)
//! ```
//!
//! Both calls are considered constrained:
//! * The output of the 2nd call (v6) is constrained directly against its input (v4)
//! * The output of the 1st call (v5) has a descendant (v6) which is constrained against
//!   a value (v4) that has an ancestor (v2) which is also an ancestor of an argument of
//!   of the call itself.
//!
//! The goal isn't to verify that the constraint is correct, just that some (indirect)
//! connection between inputs and outputs is made.
use crate::ssa::checks::is_numeric_constant;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::function::{Function, FunctionId};
use crate::ssa::ir::instruction::{Instruction, InstructionId, Intrinsic};
use crate::ssa::ir::post_order::PostOrder;
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::ssa_gen::Ssa;
use acvm::AcirField;
use bit_vec::BitVec;
use noirc_artifacts::ssa::{InternalBug, SsaReport};
use rayon::prelude::*;
use std::cmp;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

/// The maximum length of arrays that we attempt to constrain item-by-item.
///
/// Arrays longer than this value will be considered constrained if any item
/// we get from them gets constrained.
///
/// The higher this value the longer it will take to check them all,
/// which can slow down the compilation of larger rollup circuits.
const MAX_ARRAY_OUTPUT_LENGTH: u32 = 64;

/// Limit how far back the BFS traverses to try to find a relation between
/// the ancestors of constrained values and Brillig inputs/outputs.
///
/// This exists to help keep the runtime down on the largest protocol circuits,
/// such as `rollup-checkpoint-root` and `rollup-checkpoint-root-single-block`,
/// which have hundreds of thousands of constraints that we need to check,
/// even though they are only checked against a few dozen of Brillig outputs.
const MAX_ANCESTOR_DISTANCE: u32 = 10;

impl Ssa {
    /// Detect Brillig calls left unconstrained with manual asserts
    /// and return a vector of bug reports if any have been found
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) fn check_for_missing_brillig_constraints(&mut self) -> Vec<SsaReport> {
        // Skip the check if there are no Brillig functions involved
        if !self.functions.values().any(|func| func.runtime().is_brillig()) {
            return vec![];
        }

        self.functions
            .values()
            .filter(|func| func.runtime().is_acir() && has_call_to_brillig(func, &self.functions))
            .par_bridge()
            .flat_map(|func| {
                Context::new(func)
                    .build_tainted(func, &self.functions)
                    .build_parent_graph(func)
                    .constrain_tainted(func, &self.functions)
                    .into_warnings(func)
            })
            .collect()
    }
}

/// A more compact representation of a `HashSet<ValueId>` to limit memory use.
#[derive(Debug)]
struct ValueSet(BitVec<u32>);

impl ValueSet {
    fn new(dfg: &DataFlowGraph) -> Self {
        Self(BitVec::from_elem(dfg.values_iter().count(), false))
    }

    fn contains(&self, value: &ValueId) -> bool {
        self.0.get(value.to_u32().try_into().unwrap()).expect("initialized with all values")
    }

    fn insert(&mut self, value: ValueId) {
        self.0.set(value.to_u32().try_into().unwrap(), true);
    }

    fn extend(&mut self, values: &[ValueId]) {
        for value in values {
            self.insert(*value);
        }
    }
}

/// Outputs of a Brillig call and their descendants.
#[derive(Debug)]
struct TaintedDescendants {
    /// Inputs of the call.
    ///
    /// To consider the call constrained, the constraint must be on a value which has
    /// an ancestry that intersects with the ancestry of an argument.
    arguments: Vec<ValueId>,
    /// Non-array outputs of the call.
    ///
    /// To consider the output constrained, we have to find a constraint such that
    /// the output is an ancestor of the constrained value.
    single_outputs: HashSet<ValueId>,
    /// Array outputs of the call, tracked per index, accumulating their individual
    /// dependencies (only the values read from the array).
    ///
    /// To consider an element constrained, we have to find a constraint such that
    /// the constrained value appears in the descendants.
    array_outputs: HashMap<ValueId, HashMap<u32, HashSet<ValueId>>>,
    /// The union of all values reachable from any argument by following parents and
    /// equivalences backwards. Includes the arguments themselves.
    ///
    /// Pre-computed after the parent graph is built so that `arguments_intersect`
    /// can check membership in O(1) rather than re-running BFS for every constraint.
    arg_ancestors: HashSet<ValueId>,
    /// Set of values the constraints on which are interesting for at least one of
    /// the outputs. This helps eliminate constraints which are of no effect.
    constrainable: ValueSet,
}

impl TaintedDescendants {
    /// Create a new `TaintedDescendants` from the arguments and results of a call.
    ///
    /// Populates `single_outputs` and `array_outputs` according to the result types.
    /// Leaves `arg_ancestors` to be populated later.
    fn new(func: &Function, arguments: Vec<ValueId>, result_ids: &[ValueId]) -> Self {
        let mut single_outputs = HashSet::new();
        let mut array_outputs = HashMap::new();
        for result_id in result_ids {
            match func.dfg.try_get_array_length(*result_id) {
                // If the result value is an array, create an empty descendant set for
                // every element to be accessed further on and record the indices
                // of the resulting sets for future reference
                Some(length) if length.0 > 0 && length.0 <= MAX_ARRAY_OUTPUT_LENGTH => {
                    let mut index_outputs = HashMap::new();
                    for i in 0..length.0 {
                        index_outputs.insert(i, HashSet::new());
                    }
                    array_outputs.insert(*result_id, index_outputs);
                }
                // For very large arrays or non-arrays, treat the whole result as a single value
                // to avoid memory/time issues when tracking individual elements
                Some(_) | None => {
                    single_outputs.insert(*result_id);
                }
            }
        }

        let mut constrainable = ValueSet::new(&func.dfg);
        constrainable.extend(result_ids);

        Self {
            arguments,
            single_outputs,
            array_outputs,
            arg_ancestors: HashSet::new(),
            constrainable,
        }
    }

    /// Whether there are any unconstrained outputs left.
    fn is_constrained(&self) -> bool {
        self.single_outputs.is_empty() && self.array_outputs.is_empty()
    }

    /// Try to constrain some of the outputs if:
    /// * one of the constrained values is a descendant of the output, and
    /// * another constrained value shares an ancestor with an input, and it is not tainted
    ///
    /// Exceptions to this rule are:
    /// * if there are no input arguments (they were all numeric constants, or there were no args)
    /// * if there is only one constrained value (an output against a constant)
    ///
    /// Return `true` if all outputs have been constrained.
    fn try_constrain(
        &mut self,
        constrained_values: &[ValueId],
        parents: &HashMap<ValueId, Vec<ValueId>>,
        equivalences: &HashMap<ValueId, Vec<ValueId>>,
        all_tainted: &ValueSet,
    ) -> bool {
        // Make sure this constraint has something to do with the outputs.
        if !constrained_values.iter().any(|v| self.constrainable.contains(v)) {
            return false;
        }

        let is_against_const = constrained_values.len() == 1;
        let is_const_args = self.arguments.is_empty();

        // Make sure this constraint has something to do with the inputs,
        // unless there are no inputs, or the output is against a constant.
        if !is_against_const
            && !is_const_args
            && !self.arguments_intersect(constrained_values, parents, equivalences, all_tainted)
        {
            return false;
        }

        // Remove any results that have been directly or indirectly constrained.
        self.single_outputs.retain(|output| {
            let constrained = constrained_values
                .iter()
                .any(|value| any_ancestor_is(*value, *output, parents, equivalences));
            !constrained
        });

        self.array_outputs.retain(|array, index_outputs| {
            // If the array itself is not an ancestor of the constrained value, then we don't have to check the items.
            let can_constrain = constrained_values
                .iter()
                .any(|value| any_ancestor_is(*value, *array, parents, equivalences));

            if !can_constrain {
                return true;
            }

            // Remove whichever index was constrained.
            index_outputs.retain(|_index, descendants| {
                // Until we have seen an ArrayGet and know which value is the output,
                // we can't tell this index has been constrained.
                if descendants.is_empty() {
                    return true;
                }
                let constrained = constrained_values
                    .iter()
                    .any(|value| any_ancestor_in(*value, descendants, parents, equivalences));
                !constrained
            });

            // Keep the array until all indexed items have been constrained.
            !index_outputs.is_empty()
        });

        self.is_constrained()
    }

    /// Whether one of the constrained values:
    /// * shares an ancestor with a call argument (checked via pre-computed `arg_ancestors`), and
    /// * is not tainted
    fn arguments_intersect(
        &self,
        constrained_values: &[ValueId],
        parents: &HashMap<ValueId, Vec<ValueId>>,
        equivalences: &HashMap<ValueId, Vec<ValueId>>,
        all_tainted: &ValueSet,
    ) -> bool {
        for &cv in constrained_values {
            if all_tainted.contains(&cv) {
                // Allowing these would mean we could constrain the output of one call
                // with the output of another Brillig call, and also that outputs of
                // the call would trivially connect to the inputs.
                continue;
            }
            // arg_ancestors contains the arguments themselves and all their transitive ancestors.
            // BFS from cv to check if cv or any ancestor of cv is in arg_ancestors.
            if any_ancestor_in(cv, &self.arg_ancestors, parents, equivalences) {
                return true;
            }
        }
        false
    }

    /// Add to the descendants of a particular array element.
    ///
    /// This is only called when we read from an array. Later on we can use the
    /// ancestry information to connect constrained values back to values we read
    /// from the array.
    fn extend_array_result(&mut self, array: ValueId, index: u32, results: &[ValueId]) {
        let Some(index_outputs) = self.array_outputs.get_mut(&array) else {
            return;
        };
        let Some(descendants) = index_outputs.get_mut(&index) else {
            return;
        };
        descendants.extend(results);
    }

    /// If any of the `args` is one of the constrainable values, then extend them with the `results`.
    fn extend_constrainable(&mut self, args: &[ValueId], results: &[ValueId]) {
        if args.iter().any(|v| self.constrainable.contains(v)) {
            self.constrainable.extend(results);
        }
    }
}

#[derive(Debug)]
struct Context {
    /// Block IDs in Post Order.
    post_order: Vec<BasicBlockId>,

    /// Descendants of Brillig calls.
    tainted: HashMap<InstructionId, TaintedDescendants>,

    /// Constraints which will be relevant to constraining Brillig outputs.
    ///
    /// These are determined during the initial top-down pass,
    /// so that we can limit the amount of ancestry we collect.
    constraints: HashSet<InstructionId>,

    /// Direct parent graph for tracked values.
    ///
    /// `parents[v]` = the immediate instruction arguments that produced `v`,
    /// plus the active side-effect condition (if any) at the time `v` was produced.
    ///
    /// We track parents for values which either:
    /// * have constraints on them, or
    /// * are inputs to a Brillig call.
    ///
    /// Transitive ancestry is computed on demand via BFS instead of being pre-computed.
    parents: HashMap<ValueId, Vec<ValueId>>,

    /// Bidirectional equivalence edges from `constrain v1 == v2` instructions.
    ///
    /// If `v1` and `v2` are equivalent, any ancestor of `v1` is also an ancestor of `v2`
    /// and vice versa. BFS follows these edges alongside `parents` edges.
    equivalences: HashMap<ValueId, Vec<ValueId>>,
}

impl Context {
    fn new(func: &Function) -> Self {
        Self {
            post_order: PostOrder::with_function(func).into_vec(),
            tainted: HashMap::default(),
            constraints: HashSet::default(),
            parents: HashMap::default(),
            equivalences: HashMap::default(),
        }
    }

    /// Build a direct parent graph for tracked values, then compute `arg_ancestors` for each
    /// tainted Brillig call via BFS.
    ///
    /// This avoids having to have a transitive-closure `ancestors` map, with a compact representation:
    /// `parents[v]` stores only the immediate instruction arguments of `v` (plus the active
    /// side-effect condition, if any). Transitive ancestry is computed on demand during BFS.
    fn build_parent_graph(mut self, func: &Function) -> Self {
        // Forward sub-pass: collect which side-effect condition (if any) is active at each
        // instruction, so we can add it as a parent during the backward pass below.
        let mut side_effect_at: HashMap<InstructionId, ValueId> = HashMap::new();
        for block_id in self.post_order.iter().copied().rev() {
            let mut current_se: Option<ValueId> = None;
            for instr_id in func.dfg[block_id].instructions() {
                if let Instruction::EnableSideEffectsIf { condition } = &func.dfg[*instr_id] {
                    current_se = (!is_numeric_constant(func, *condition)).then_some(*condition);
                } else if let Some(se) = current_se {
                    side_effect_at.insert(*instr_id, se);
                }
            }
        }

        // Backward pass: build the parent graph.
        //
        // pending_loads[address] = list of tracked load results whose direct parent is `address`.
        // When we later encounter Store { address, value }, we fix those parents up.
        let mut pending_loads: HashMap<ValueId, Vec<ValueId>> = HashMap::new();

        for block_id in self.post_order.iter().copied() {
            for instruction_id in func.dfg[block_id].instructions().iter().rev() {
                let instruction = &func.dfg[*instruction_id];
                let result_ids = func.dfg.instruction_results(*instruction_id);

                // For each tracked result, add its instruction's arguments as direct parents.
                // Compute args lazily — only when we find a tracked result.
                let mut args: Option<Vec<ValueId>> = None;

                for result_id in result_ids {
                    if is_numeric_constant(func, *result_id)
                        || !self.parents.contains_key(result_id)
                    {
                        continue;
                    }

                    let args = args.get_or_insert_with(|| instruction_arguments(func, instruction));

                    self.parents.entry(*result_id).or_default().extend(args.iter().copied());

                    // Ensure each arg is itself tracked so that when we reach the instruction
                    // that produces arg (going backward), we expand its parents too.
                    // This is the equivalent of the who_cares[arg].insert(key) update.
                    for &arg in args.iter() {
                        self.parents.entry(arg).or_default();
                    }

                    // Add the active side-effect condition as an additional parent so that
                    // BFS can reach the condition's ancestors from this result.
                    if let Some(&se) = side_effect_at.get(instruction_id) {
                        self.parents.entry(*result_id).or_default().push(se);
                        // Ensure the condition itself is tracked.
                        self.parents.entry(se).or_default();
                    }

                    // If this is a Load, remember it so Store can fix up the placeholder parent.
                    if let Instruction::Load { address } = instruction {
                        pending_loads.entry(*address).or_default().push(*result_id);
                    }
                }

                // Store resolution: replace the address placeholder with the stored value in
                // all pending load results for this address. By using remove(), only the
                // first Store encountered (going backward) resolves the loads.
                if let Instruction::Store { address, value } = instruction
                    && let Some(pending) = pending_loads.remove(address)
                {
                    for tracked in pending {
                        let parents_of_tracked =
                            self.parents.get_mut(&tracked).expect("was inserted above");
                        parents_of_tracked.retain(|&p| p != *address);
                        if !is_numeric_constant(func, *value) {
                            parents_of_tracked.push(*value);
                            // Start tracking the stored value's own parents.
                            self.parents.entry(*value).or_default();
                        }
                    }
                }

                // Start tracking the direct parents of this instruction's arguments if it is
                // a tainted call, a relevant constraint, or an EnableSideEffectsIf instruction.
                let should_track = self.tainted.contains_key(instruction_id)
                    || self.constraints.contains(instruction_id)
                    || is_side_effect(func, instruction);

                if should_track {
                    let args = args.get_or_insert_with(|| instruction_arguments(func, instruction));
                    for value_id in args.iter() {
                        self.parents.entry(*value_id).or_default();
                    }
                }

                // Collect equivalences from `constrain v1 == v2`.
                // These are followed bidirectionally during BFS so that ancestry flows
                // through equivalent values.
                if let Some((v1, v2)) = as_equivalence(func, instruction) {
                    self.equivalences.entry(v1).or_default().push(v2);
                    self.equivalences.entry(v2).or_default().push(v1);
                }
            }
        }

        // BFS sub-pass: compute arg_ancestors for each tainted Brillig call.
        // arg_ancestors is the union of all values reachable backwards from any argument,
        // including the arguments themselves. This is pre-computed once so that
        // arguments_intersect can check membership in O(1) per constrained value.
        let parents = &self.parents;
        let equivalences = &self.equivalences;
        for tainted in self.tainted.values_mut() {
            tainted.arg_ancestors = bfs_ancestors(&tainted.arguments, parents, equivalences);
        }

        self
    }

    /// Traverse blocks and instructions top-down to build up the descendants of Brillig calls.
    fn build_tainted(
        mut self,
        func: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) -> Self {
        // The distance at which we track constrainable values.
        let mut all_constrainable: HashMap<ValueId, u32> = HashMap::new();

        // Traverse in Reverse Post Order, ie. top-down.
        for block_id in self.post_order.clone().into_iter().rev() {
            // Track the current side effect variable, unless it's a constant.
            let mut side_effects_var: Option<ValueId> = None;
            // No need to look for constraints on calls which originate from the same code location;
            // these are the result of unrolling loops, and it should be enough to cover the first.
            let mut visited_locations = HashSet::new();

            for instruction_id in func.dfg[block_id].instructions() {
                let instruction = &func.dfg[*instruction_id];
                let mut arguments = instruction_arguments(func, instruction);
                let results = instruction_results(func, instruction_id);

                // If we are under a side effect, extend the args.
                if let Some(side_effects_var) = &side_effects_var {
                    arguments.push(*side_effects_var);
                }

                // Extend the descendants of Brillig calls.
                // This is only required for array output; for single outputs we can look at the ancestry.
                if !results.is_empty() {
                    // Look for ArrayGet instructions with a constant index,
                    // and if the array is the result of a tainted call,
                    // then add the result as a descendant of that particular index.
                    if let Instruction::ArrayGet { array, index } = instruction
                        && let Some(index) = func.dfg.get_numeric_constant(*index)
                        && let Some(index) = index.try_to_u32()
                    {
                        for tainted in self.tainted.values_mut() {
                            tainted.extend_array_result(*array, index, &results);
                        }
                    }

                    // Extend the values we are looking to constrain.
                    let min_dist = arguments
                        .iter()
                        .fold(None, |acc, arg| match (acc, all_constrainable.get(arg)) {
                            (None, dist) => dist,
                            (acc, None) => acc,
                            (Some(acc), Some(dist)) => Some(cmp::min(acc, dist)),
                        })
                        .copied();

                    // Only extend if we will not exceed the traversal limit to reach them.
                    if let Some(dist) = min_dist
                        && dist < MAX_ANCESTOR_DISTANCE
                    {
                        all_constrainable.extend(results.iter().map(|r| (*r, dist + 1)));
                        self.extend_constrainable(&arguments, &results);
                    }
                }

                // If this is a Store instruction, then it has no result: instead if the value we store
                // is constrainable, then we can add the address to the constrainable set.
                if let Instruction::Store { address, value } = instruction
                    && let Some(dist) = all_constrainable.get(value)
                {
                    // Keep the same distance as the address is just a handover point for values.
                    all_constrainable.insert(*address, *dist);
                    self.extend_constrainable(&[*value], &[*address]);
                }

                // If we have a constraint that means two values are equal, then we are interested
                // in constraints on the descendants on either of those, even if one of them is
                // not a descendant of Brillig outputs.
                if let Some((v1, v2)) = as_equivalence(func, instruction) {
                    if let Some(dist) = all_constrainable.get(&v1) {
                        all_constrainable.insert(v2, *dist);
                        self.extend_constrainable(&[v1], &[v2]);
                    } else if let Some(dist) = all_constrainable.get(&v2) {
                        all_constrainable.insert(v1, *dist);
                        self.extend_constrainable(&[v2], &[v1]);
                    }
                }

                if is_call_to_brillig(func, all_functions, instruction_id) && !results.is_empty() {
                    // Skip already visited locations (happens often in unrolled functions)
                    let call_stack = func.dfg.get_instruction_call_stack(*instruction_id);
                    let location = call_stack.last();

                    // If there is no call stack (happens for tests), consider unvisited
                    let visited = match location {
                        Some(loc) => {
                            let Instruction::Call { func: callee, .. } = instruction else {
                                unreachable!("ICE: Expected Brillig call");
                            };
                            !visited_locations.insert((*callee, *loc))
                        }
                        None => false,
                    };

                    // Skip if we have a similar one already.
                    if !visited {
                        let tainted = TaintedDescendants::new(func, arguments, &results);
                        self.tainted.insert(*instruction_id, tainted);
                        // Look out for constraints on these outputs.
                        // We don't need to consider the inputs: the constraints which are relevant will have to constrain
                        // at least one output. Then, we will look at whether the other constrained value is related to
                        // the inputs, based on its ancestry, collected later for all inputs of relevant constraints.
                        all_constrainable.extend(results.iter().map(|r| (*r, 0)));
                    }
                } else if is_constraint(func, instruction_id) && !self.tainted.is_empty() {
                    let constrained_values = instruction_arguments(func, instruction);
                    // If this constraint involves a Brillig output, then we can use it later, otherwise it's not interesting.
                    if constrained_values.iter().any(|value| all_constrainable.contains_key(value))
                    {
                        self.constraints.insert(*instruction_id);
                    }
                } else if let Instruction::EnableSideEffectsIf { condition } = instruction {
                    side_effects_var =
                        (!is_numeric_constant(func, *condition)).then_some(*condition);
                }
            }
        }

        self
    }

    /// If any of the `args` is one of the constrainable values of a tainted call,
    /// then extend them with the `results`.
    fn extend_constrainable(&mut self, args: &[ValueId], results: &[ValueId]) {
        for t in self.tainted.values_mut() {
            t.extend_constrainable(args, results);
        }
    }

    /// Traverse blocks and instructions top-down and try to constrain Brillig outputs.
    fn constrain_tainted(
        mut self,
        func: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) -> Self {
        // Constraints on tainted output cannot be used to connect output to input.
        let mut all_tainted = ValueSet::new(&func.dfg);
        // Skip checks until we encounter the tainted instruction.
        let mut active_tainted = HashSet::new();

        // Traverse in Reverse Post Order, ie. top-down.
        for block_id in self.post_order.clone().into_iter().rev() {
            for instruction_id in func.dfg[block_id].instructions() {
                let instruction = &func.dfg[*instruction_id];
                let arguments = instruction_arguments(func, instruction);
                let results = instruction_results(func, instruction_id);

                // Extend the descendants of Brillig calls.
                if !results.is_empty() {
                    // Tainted values cannot be used to constrain Brillig output.
                    if arguments.iter().any(|a| all_tainted.contains(a)) {
                        all_tainted.extend(&results);
                    }
                }

                if is_call_to_brillig(func, all_functions, instruction_id) && !results.is_empty() {
                    // Always keep track of tainted descendants, required for correct constraint checks.
                    all_tainted.extend(&results);
                    if self.tainted.contains_key(instruction_id) {
                        active_tainted.insert(instruction_id);
                    }
                } else if self.constraints.contains(instruction_id)
                    && !self.tainted.is_empty()
                    && !active_tainted.is_empty()
                {
                    let constrained_values = instruction_arguments(func, instruction);
                    // Split borrows: extract parents/equivalences before the closure that
                    // mutably borrows self.tainted.
                    let parents = &self.parents;
                    let equivalences = &self.equivalences;
                    self.tainted.retain(|id, tainted| {
                        if !active_tainted.contains(id) {
                            return true;
                        }

                        let constrained = tainted.try_constrain(
                            &constrained_values,
                            parents,
                            equivalences,
                            &all_tainted,
                        );

                        if constrained {
                            active_tainted.remove(id);
                        }

                        !constrained
                    });
                }
            }
        }

        self
    }

    /// Every Brillig call not properly constrained should remain in the tainted set
    /// at this point. For each, emit a corresponding warning.
    fn into_warnings(self, function: &Function) -> Vec<SsaReport> {
        self.tainted
            .keys()
            .map(|brillig_call| {
                SsaReport::Bug(InternalBug::UncheckedBrilligCall {
                    call_stack: function.dfg.get_instruction_call_stack(*brillig_call),
                })
            })
            .collect()
    }
}

/// Whether there is at least one instruction making a call to a Brillig function with non-empty results.
fn has_call_to_brillig(func: &Function, all_functions: &BTreeMap<FunctionId, Function>) -> bool {
    for block_id in func.reachable_blocks() {
        for instruction_id in func.dfg[block_id].instructions() {
            if is_call_to_brillig(func, all_functions, instruction_id) {
                return true;
            }
        }
    }
    false
}

/// Whether the instruction is a call to a Brillig function with a non-empty results.
fn is_call_to_brillig(
    func: &Function,
    all_functions: &BTreeMap<FunctionId, Function>,
    instruction_id: &InstructionId,
) -> bool {
    let Instruction::Call { func: callee_id, .. } = func.dfg[*instruction_id] else {
        return false;
    };
    let Value::Function(callee_id) = func.dfg[callee_id] else {
        return false;
    };
    if !all_functions[&callee_id].runtime().is_brillig() {
        return false;
    }
    !func.dfg.instruction_results(*instruction_id).is_empty()
}

/// Whether an instruction puts constraints on its inputs.
fn is_constraint(func: &Function, instruction_id: &InstructionId) -> bool {
    let instruction = &func.dfg[*instruction_id];
    if matches!(
        instruction,
        Instruction::Constrain(..)
            | Instruction::ConstrainNotEqual(..)
            | Instruction::RangeCheck { .. }
    ) {
        return true;
    }
    let Instruction::Call { func: callee_id, .. } = instruction else {
        return false;
    };
    let Value::Intrinsic(intrinsic) = &func.dfg[*callee_id] else {
        return false;
    };
    matches!(intrinsic, Intrinsic::ApplyRangeConstraint | Intrinsic::AssertConstant)
}

/// Whether the instruction enables side effects with a non-constant variable.
fn is_side_effect(func: &Function, instruction: &Instruction) -> bool {
    let Instruction::EnableSideEffectsIf { condition } = instruction else {
        return false;
    };
    !is_numeric_constant(func, *condition)
}

/// Whether the instruction is a `constrain v1 == v2` with non-constant variables.
fn as_equivalence(func: &Function, instruction: &Instruction) -> Option<(ValueId, ValueId)> {
    if let Instruction::Constrain(v1, v2, _) = instruction
        && !is_numeric_constant(func, *v1)
        && !is_numeric_constant(func, *v2)
    {
        Some((*v1, *v2))
    } else {
        None
    }
}

/// Collect non-constant arguments of an instruction.
fn instruction_arguments(func: &Function, instruction: &Instruction) -> Vec<ValueId> {
    let mut arguments = Vec::new();
    // Skip the first value of calls, which is the function ID.
    let skip_first = matches!(instruction, Instruction::Call { .. });
    let mut is_first = true;
    instruction.for_each_value(|value_id| {
        if !(skip_first && is_first || is_numeric_constant(func, value_id)) {
            arguments.push(value_id);
        }
        is_first = false;
    });
    arguments
}

/// Collect non-constant results of an instruction.
fn instruction_results(func: &Function, instruction_id: &InstructionId) -> Vec<ValueId> {
    func.dfg
        .instruction_results(*instruction_id)
        .iter()
        .filter(|value| !is_numeric_constant(func, **value))
        .copied()
        .collect()
}

/// Compute the set of all values reachable (inclusive) from any of the `starts` by following
/// `parents` and `equivalences` edges backwards.
///
/// Equivalences are only followed from **intermediate** nodes (not from the starting nodes
/// themselves). This matches the original transitive-closure semantics: `constrain v1 == v2`
/// adds v2 to the ancestor sets of keys that *already* have v1 as an ancestor, but does **not**
/// add v2 to v1's own ancestor set (because v1 is never its own ancestor).
///
/// Calls a function `f` with each value; if `f` returns `true` the traversal continues, otherwise returns.
///
/// Returns the set of visited nodes.
fn bfs_traverse_ancestors(
    starts: &[ValueId],
    parents: &HashMap<ValueId, Vec<ValueId>>,
    equivalences: &HashMap<ValueId, Vec<ValueId>>,
    mut f: impl FnMut(ValueId, u32) -> bool,
) -> HashSet<ValueId> {
    let mut visited: HashSet<ValueId> = HashSet::new();
    let mut queue: VecDeque<(ValueId, u32)> = VecDeque::new();
    for &s in starts {
        visited.insert(s);
        if !f(s, 0) {
            return visited;
        }
        // From start nodes: follow only parent edges, not equivalences.
        for &p in parents.get(&s).into_iter().flatten() {
            if visited.insert(p) {
                queue.push_back((p, 1));
            }
        }
    }
    // From intermediate nodes: follow both parent and equivalence edges.
    while let Some((curr, dist)) = queue.pop_front() {
        if !f(curr, dist) {
            return visited;
        }
        for &next in parents
            .get(&curr)
            .into_iter()
            .flatten()
            .chain(equivalences.get(&curr).into_iter().flatten())
        {
            if visited.insert(next) {
                queue.push_back((next, dist + 1));
            }
        }
    }
    visited
}

/// Compute the set of all values reachable (inclusive) from any of the `starts` by following
/// `parents` and `equivalences` edges backwards.
fn bfs_ancestors(
    starts: &[ValueId],
    parents: &HashMap<ValueId, Vec<ValueId>>,
    equivalences: &HashMap<ValueId, Vec<ValueId>>,
) -> HashSet<ValueId> {
    bfs_traverse_ancestors(starts, parents, equivalences, |_, _| true)
}

/// Returns `true` if `start` itself, or any value reachable from `start` by following
/// `parents` (and `equivalences` from intermediate nodes), equals `target`.
///
/// Equivalences are not followed directly from `start` — only from nodes reached via
/// parent edges. See [`bfs_ancestors`] for the rationale.
fn any_ancestor_is(
    start: ValueId,
    target: ValueId,
    parents: &HashMap<ValueId, Vec<ValueId>>,
    equivalences: &HashMap<ValueId, Vec<ValueId>>,
) -> bool {
    let mut found = false;
    bfs_traverse_ancestors(&[start], parents, equivalences, |a, d| {
        if a == target {
            found = true;
        }
        !found && d <= MAX_ANCESTOR_DISTANCE
    });
    found
}

/// Returns `true` if `start` itself, or any value reachable from `start` by following
/// `parents` (and `equivalences` from intermediate nodes), is contained in `target_set`.
///
/// Equivalences are not followed directly from `start` — only from nodes reached via
/// parent edges. See [`bfs_ancestors`] for the rationale.
fn any_ancestor_in(
    start: ValueId,
    target_set: &HashSet<ValueId>,
    parents: &HashMap<ValueId, Vec<ValueId>>,
    equivalences: &HashMap<ValueId, Vec<ValueId>>,
) -> bool {
    let mut found = false;
    bfs_traverse_ancestors(&[start], parents, equivalences, |a, d| {
        if target_set.contains(&a) {
            found = true;
        }
        !found && d <= MAX_ANCESTOR_DISTANCE
    });
    found
}

#[cfg(test)]
mod tests {
    use crate::ssa::Ssa;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    /// Test where a call to a Brillig function is left unchecked with a later assert,
    /// by example of the program illustrating issue #5425 (simplified variant).
    ///
    /// The crux of this test is the load and store of values leading to the constraint.
    fn test_underconstrained_value_detector_5425() {
        /*
        unconstrained fn maximum_price(options: [u32; 2]) -> u32 {
            let mut maximum_option = options[0];
            if (options[1] > options[0]) {
                maximum_option = options[1];
            }
            maximum_option
        }

        fn main(sandwiches: pub [u32; 2], drinks: pub [u32; 2], best_value: u32) {
            let most_expensive_sandwich = maximum_price(sandwiches);
            let mut sandwich_exists = false;
            sandwich_exists |= (sandwiches[0] == most_expensive_sandwich);
            sandwich_exists |= (sandwiches[1] == most_expensive_sandwich);
            assert(sandwich_exists);

            let most_expensive_drink = maximum_price(drinks);
            assert(
                best_value
                == (most_expensive_sandwich + most_expensive_drink)
            );
        }
        */
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v4: [u32; 2], v5: [u32; 2], v6: u32):
            v8 = call f1(v4) -> u32
            v9 = allocate -> &mut u1
            store u1 0 at v9
            v10 = load v9 -> u1
            v11 = array_get v4, index u32 0 -> u32
            v12 = eq v11, v8
            v13 = or v10, v12
            store v13 at v9
            v14 = load v9 -> u1
            v15 = array_get v4, index u32 1 -> u32
            v16 = eq v15, v8
            v17 = or v14, v16
            store v17 at v9
            v18 = load v9 -> u1
            constrain v18 == u1 1
            v19 = call f1(v5) -> u32
            v20 = add v8, v19
            constrain v6 == v20
            return
        }

        brillig(inline) fn maximum_price f1 {
          b0(v0: [u32; 2]):
            v2 = array_get v0, index u32 0 -> u32
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where a call to a Brillig function returning multiple result values
    /// is left unchecked with a later assert involving all the results
    fn test_unchecked_multiple_results_brillig() {
        // First call is constrained properly, involving both results
        // Second call is insufficiently constrained, involving only one of the results
        // The Brillig function is fake, for simplicity's sake
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2, v3 = call f1(v0) -> (u32, u32)
            v4 = mul v2, v3
            constrain v4 == v0
            v5, v6 = call f1(v0) -> (u32, u32)
            v7 = mul v5, v5
            constrain v7 == v0
            return
        }

        brillig(inline) fn factor f1 {
          b0(v0: u32):
            return u32 0, u32 0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig function is called with a constant argument
    /// (should _not_ lead to a false positive failed check
    /// if all the results are constrained)
    fn test_checked_brillig_with_constant_arguments() {
        // The call is constrained properly, involving both results
        // (but the argument to the Brillig is a constant)
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v3, v4 = call f1(Field 7) -> (u32, u32)
            v5 = mul v3, v4
            constrain v5 == v0
            return
        }

        brillig(inline) fn factor f1 {
          b0(v0: Field):
            return u32 0, u32 0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig function call is constrained with a range check
    /// (should _not_ lead to a false positive failed check)
    fn test_range_checked_brillig() {
        // The call is constrained properly with a range check, involving
        // both Brillig call argument and result
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = call f1(v0) -> u32
            v3 = add v2, v0
            range_check v3 to 32 bits
            return
        }

        brillig(inline) fn dummy f1 {
          b0(v0: u32):
            return u32 0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig nested type result is insufficiently constrained
    /// (with a field constraint missing)
    fn test_nested_type_result_brillig() {
        /*
        struct Animal {
            legs: Field,
            eyes: u8,
            tag: Tag,
        }

        struct Tag {
            no: Field,
        }

        unconstrained fn foo(bar: Field) -> Animal {
            Animal {
                legs: 4,
                eyes: 2,
                tag: Tag { no: bar }
            }
        }

        fn main(x: Field) -> pub Animal {
            let dog = foo(x);
            assert(dog.legs == 4);
            assert(dog.tag.no == x);

            dog
        }
        */
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2, v3, v4 = call f1(v0) -> (Field, u8, Field)
            v6 = eq v2, Field 4
            constrain v2 == Field 4
            v10 = eq v4, v0
            constrain v4 == v0
            return v2, v3, v4
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field):
            return Field 4, u8 2, v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where Brillig calls' root result values are constrained against
    /// each other (covers a false negative edge case)
    /// (https://github.com/noir-lang/noir/pull/6658#pullrequestreview-2482170066)
    fn test_root_result_intersection_false_negative() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = call f1(v0, v1) -> Field
            v5 = call f1(v0, v1) -> Field
            v6 = eq v3, v5
            constrain v3 == v5
            v8 = add v3, v5
            return v8
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 2);
    }

    #[test]
    #[traced_test]
    /// Test EnableSideEffectsIf conditions affecting the dependency graph
    /// (SSA a bit convoluted to work around simplification breaking the flow
    /// of the parsed test code). Note that the side effect variable is a
    /// descendant of the output of the call, and the constraint is on a
    /// variable which is affected by the side effect variable.
    fn test_enable_side_effects_affecting_following_statements() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = call f1(v0, v1) -> Field
            v5 = add v0, v1
            v6 = eq v3, v5
            v7 = add u1 1, u1 0
            enable_side_effects v6
            v8 = add v7, u1 1
            enable_side_effects u1 1
            constrain v8 == u1 2
            return v3
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test call result array elements being underconstrained
    fn test_brillig_result_array_missing_element_constraint() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v16 = call f1(v0) -> [u32; 3]
            v17 = array_get v16, index u32 0 -> u32
            constrain v17 == v0
            v19 = array_get v16, index u32 2 -> u32
            constrain v19 == v0
            return v17
        }

        brillig(inline) fn into_array f1 {
          b0(v0: u32):
            v4 = make_array [v0, v0, v0] : [u32; 3]
            return v4
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test call result array elements being constrained properly
    fn test_brillig_result_array_all_elements_constrained() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v16 = call f1(v0) -> [u32; 3]
            v17 = array_get v16, index u32 0 -> u32
            constrain v17 == v0
            v20 = array_get v16, index u32 1 -> u32
            constrain v20 == v0
            v19 = array_get v16, index u32 2 -> u32
            constrain v19 == v0
            return v17
        }

        brillig(inline) fn into_array f1 {
          b0(v0: u32):
            v4 = make_array [v0, v0, v0] : [u32; 3]
            return v4
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test chained (wrapper) Brillig calls not producing a false positive.
    ///
    /// A wrapper was considered something that passes all the outputs of
    /// one Brillig call as inputs to the next Brillig call.
    fn test_chained_brillig_calls_constrained_wrapped() {
        /*
        struct Animal {
            legs: Field,
            eyes: u8,
            tag: Tag,
        }

        struct Tag {
            no: Field,
        }

        unconstrained fn foo(x: Field) -> Animal {
            Animal {
                legs: 4,
                eyes: 2,
                tag: Tag { no: x }
            }
        }

        unconstrained fn bar(x: Animal) -> Animal {
            Animal {
                legs: x.legs,
                eyes: x.eyes,
                tag: Tag { no: x.tag.no + 1 }
            }
        }

        fn main(x: Field) -> pub Animal {
            let dog = bar(foo(x));
            assert(dog.legs == 4);
            assert(dog.eyes == 2);
            assert(dog.tag.no == x + 1);

            dog
        }
        */
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v27, v28, v29 = call f2(v0) -> (Field, u8, Field)
            v30, v31, v32 = call f1(v27, v28, v29) -> (Field, u8, Field)
            constrain v30 == Field 4
            constrain v31 == u8 2
            v35 = add v0, Field 1
            constrain v32 == v35
            return v30, v31, v32
        }

        brillig(inline) fn foo f2 {
          b0(v0: Field):
            return Field 4, u8 2, v0
        }

        brillig(inline) fn bar f1 {
          b0(v0: Field, v1: u8, v2: Field):
            v7 = add v2, Field 1
            return v0, v1, v7
        }

        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test chained Brillig calls.
    ///
    /// This is based on the diagram from the top of the module.
    fn test_chained_brillig_calls_constrained_mixed() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v3 = mul v0, v1
            v4 = call f1(v1, v2) -> Field
            v5 = call f1(v3, v4) -> Field
            constrain v3 == v5
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Show that the output of two Brillig calls don't constrain each other.
    fn test_brillig_calls_constrained_only_against_each_other() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = call f1(v0, v1) -> Field
            v3 = call f1(v2, v2) -> Field
            constrain v2 == v3
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 2);
    }

    #[test]
    #[traced_test]
    /// Test chained Brillig calls.
    ///
    /// In this one we constrain the output of the first call against a constant,
    /// then we feed it into a second call, and constrain the second call output
    /// against its tainted input. But because the tainted input is constrained,
    /// the second call should be constrained as well.
    fn test_chained_brillig_calls_constrained_against_const() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v3 = mul v0, v1
            v4 = call f1(v1, v2) -> Field
            v5 = call f1(v4, v4) -> Field
            constrain v4 == Field 10
            v6 = mul v4, Field 2
            constrain v5 == v6
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test for the argument descendants coming before Brillig calls themselves being
    /// registered as such
    fn test_brillig_argument_descendants_preceding_call() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = add v0, v1
            v5 = call f1(v0, v1) -> Field
            constrain v3 == v5
            return v3
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// No-result calls (e.g. print) shouldn't trigger the check
    fn test_no_result_brillig_calls() {
        let program = r#"
        acir(inline) fn main f0 {
          b0():
            call f1(Field 1)
            return Field 1
        }
        acir(inline) fn println f1 {
          b0(v0: Field):
            call f2(u1 1, v0)
            return
        }
        brillig(inline) fn print_unconstrained f2 {
          b0(v0: u1, v1: Field):
            return
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test for programs equivalent to the below (#10547):
    ///
    /// ```noir
    /// unconstrained fn identity(input: u64) -> u64 {
    ///     input
    /// }
    ///
    /// pub fn main(input: u32) {
    ///     let casted_input = input as u64;
    ///     let input_copy = unsafe { identity(casted_input) };
    ///     assert_eq(input_copy as Field, casted_input as Field);
    /// }
    /// ```
    fn multiple_casts_on_brillig_input_does_not_result_in_warning() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
            b0(v0: u32):
            v1 = cast v0 as u64
            v3 = call f1(v1) -> u64
            v4 = cast v3 as Field
            v5 = cast v0 as Field
            constrain v4 == v5
            return
        }
        brillig(inline) predicate_pure fn identity f1 {
            b0(v0: u64):
            return v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn truncating_brillig_argument_does_not_result_in_warning() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
            b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = call f1(v1) -> Field
            constrain v2 == v0
            return
        }
        brillig(inline) predicate_pure fn identity32 f1 {
            b0(v0: Field):
            return v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn constrain_on_independent_variable_can_indirectly_clear_results() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = call f1(v0) -> u32
            constrain v3 == v1       // This constraint does not connect the input of f1 to the output, so it doesn't clear.
            v4 = lt v1, u32 1000000  // This is a constraint against a constant, so it would clear if it was directly v3.
            constrain v4 == u1 1     // Since we asserted that v3 equals v1, this should indirectly clear v3.
            return
        }
        brillig(inline) predicate_pure fn f f1 {
          b0(v0: u32):
            return v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn constrain_on_array_element_links_to_input_array() {
        // Regression test for https://github.com/noir-lang/noir/issues/11807
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = make_array [v0] : [Field; 1]
            v3 = call f1(v1) -> Field
            constrain v3 == v0
            return v3
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 1]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0, "Expected no warnings but found some.");
    }

    #[test]
    #[traced_test]
    fn constrain_on_nested_array_element_links_to_input_array() {
        // Nested array variant: [[Field; 1]; 1] wrapping v0
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = make_array [v0] : [Field; 1]
            v2 = make_array [v1] : [[Field; 1]; 1]
            v4 = call f1(v2) -> Field
            constrain v4 == v0
            return v4
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [[Field; 1]; 1]):
            v2 = array_get v0, index u32 0 -> [Field; 1]
            v3 = array_get v2, index u32 0 -> Field
            return v3
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0, "Expected no warnings but found some.");
    }

    #[test]
    #[traced_test]
    fn array_set_with_variable_index_constrain_against_set_value() {
        // Array built from constants, then array_set with a non-constant index
        // inserts v0. Brillig result constrained against v0.
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: u32):
            v2 = make_array [Field 0, Field 0] : [Field; 2]
            v3 = array_set v2, index v1, value v0
            v4 = call f1(v3) -> Field
            constrain v4 == v0
            return v4
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: array_set value should be tracked as a call argument."
        );
    }

    #[test]
    #[traced_test]
    fn array_set_on_param_array_constrain_against_original_element() {
        // make_array [v0, v1], then array_set at non-constant index with v0.
        // Brillig result constrained against v0 (which is both in the original
        // make_array AND the array_set value).
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field, v2: u32):
            v3 = make_array [v0, v1] : [Field; 2]
            v4 = array_set v3, index v2, value v0
            v5 = call f1(v4) -> Field
            constrain v5 == v0
            return v5
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: array_set on make_array with params, constrained against original element."
        );
    }

    #[test]
    #[traced_test]
    fn array_set_constrain_result_array_elements() {
        // Brillig returns an array. We array_get each element and constrain
        // against the values used in the array_set. Since the Brillig call's
        // result is an array, the checker uses array element tracking.
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field, v2: u32):
            v3 = make_array [v0, Field 0] : [Field; 2]
            v4 = array_set v3, index v2, value v1
            v5 = call f1(v4) -> [Field; 2]
            v6 = array_get v5, index u32 0 -> Field
            v7 = array_get v5, index u32 1 -> Field
            constrain v6 == v0
            constrain v7 == v1
            return
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            return v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: both array elements constrained against inputs."
        );
    }

    #[test]
    #[traced_test]
    fn outputs_do_not_trivially_connect_to_inputs() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1, v2 = call f1(v0) -> (u32, u32)
            constrain v1 == v2
            return
        }
        brillig(inline) predicate_pure fn f f1 {
          b0(v0: u32):
            return v0, v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(
            ssa_level_warnings.len(),
            1,
            "We are constraining the outputs, but they are *not* connected to the inputs"
        );
    }

    #[test]
    #[traced_test]
    fn single_call_no_constraint() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v1 = call f1() -> i64
            return v1
        }
        brillig(inline) predicate_pure fn func_1 f1 {
          b0():
            v2 = shl i64 0, i64 -877061792390071735
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    fn array_output_constant_constraint_on_sum() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = call f1(v0) -> [u32; 2]
            v2 = array_get v1, index u32 0 -> u32
            v3 = array_get v1, index u32 1 -> u32
            v4 = unchecked_add v2, v3
            v5 = lt v4, u32 100
            constrain v5 == u1 1
            return
        }
        brillig(inline) predicate_pure fn f f1 {
          b0(v0: u32):
            v1 = make_array [v0, v0] : [u32; 2]
            return v1
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    /// The array returned is longer than MAX_ARRAY_OUTPUT_LENGTH so we don't track it item-by-item,
    /// but the constraint placed on a few items should clear the whole array.
    #[test]
    #[traced_test]
    fn large_array_output_constant_constraint_on_sum() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = call f1(v0) -> [u32; 100]
            v2 = array_get v1, index u32 0 -> u32
            v3 = array_get v1, index u32 1 -> u32
            v4 = unchecked_add v2, v3
            v5 = lt v4, u32 100
            constrain v5 == u1 1
            return
        }
        brillig(inline) predicate_pure fn f f1 {
          b0(v0: u32):
            v1 = make_array [
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
            ] : [u32; 100]
            return v1
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints();
        assert_eq!(ssa_level_warnings.len(), 0);
    }
}
