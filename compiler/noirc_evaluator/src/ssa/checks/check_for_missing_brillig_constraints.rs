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
use crate::ssa::ir::function::RuntimeType;
use crate::ssa::ir::function::{Function, FunctionId};
use crate::ssa::ir::instruction::{Instruction, InstructionId, Intrinsic};
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::ssa_gen::Ssa;
use noirc_artifacts::ssa::{InternalBug, SsaReport};
use noirc_errors::Location;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;
use tracing::trace;

impl Ssa {
    /// Detect Brillig calls left unconstrained with manual asserts
    /// and return a vector of bug reports if any have been found
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) fn check_for_missing_brillig_constraints(
        &mut self,
        enable_lookback: bool,
    ) -> Vec<SsaReport> {
        // Skip the check if there are no Brillig functions involved
        if !self.functions.values().any(|func| func.runtime().is_brillig()) {
            return vec![];
        }

        self.functions
            .values()
            .map(|f| f.id())
            .par_bridge()
            .flat_map(|fid| {
                let function_to_process = &self.functions[&fid];
                match function_to_process.runtime() {
                    RuntimeType::Acir { .. } => {
                        let mut context =
                            DependencyContext { enable_lookback, ..Default::default() };
                        context.build(function_to_process, &self.functions);
                        context.collect_warnings(function_to_process)
                    }
                    RuntimeType::Brillig(_) => Vec::new(),
                }
            })
            .collect()
    }
}

#[derive(Default)]
struct DependencyContext {
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
    /// Map keeping track of values stored at memory locations
    memory_slots: im::HashMap<ValueId, ValueId>,
    /// Value currently affecting every instruction (i.e. being
    /// considered a parent of every value id met) because
    /// of its involvement in an EnableSideEffectsIf condition
    side_effects_condition: Option<ValueId>,
    /// Map of Brillig call IDs to sets of the value IDs descending
    /// from their arguments and results
    tainted: BTreeMap<InstructionId, BrilligTaintedIds>,
    /// Map of argument value IDs to the Brillig call IDs employing them
    call_arguments: im::HashMap<ValueId, Vec<InstructionId>>,
    /// The set of calls currently being tracked
    tracking: HashSet<InstructionId>,
    /// Opt-in to use the lookback feature (tracking the argument values
    /// of a Brillig call before the call happens if their usage precedes
    /// it). Can prevent certain false positives, at the cost of
    /// slowing down checking large functions considerably
    enable_lookback: bool,
    /// Code locations of brillig calls already visited (we don't
    /// need to recheck calls happening in the same unrolled functions)
    visited_locations: HashSet<(FunctionId, Location)>,
}

/// Structure keeping track of value IDs descending from Brillig calls'
/// arguments and results, also storing information on results
/// already properly constrained
#[derive(Clone, Debug)]
struct BrilligTaintedIds {
    /// Argument descendant value ids
    arguments: HashSet<ValueId>,
    /// Results status
    results: Vec<ResultStatus>,
    /// Indices of the array elements in the results vector
    array_elements: im::HashMap<ValueId, Vec<usize>>,
    /// Initial result value ids, along with element IDs for arrays
    root_results: HashSet<ValueId>,
}

#[derive(Clone, Debug)]
enum ResultStatus {
    /// Keep track of descendants until found constrained
    Unconstrained {
        descendants: HashSet<ValueId>,
    },
    Constrained,
}

impl BrilligTaintedIds {
    fn new(function: &Function, arguments: &[ValueId], results: &[ValueId]) -> Self {
        // Exclude numeric constants
        let arguments: Vec<ValueId> = arguments
            .iter()
            .filter(|value| !is_numeric_constant(function, **value))
            .copied()
            .collect();
        let results: Vec<ValueId> = results
            .iter()
            .filter(|value| !is_numeric_constant(function, **value))
            .copied()
            .collect();

        let mut results_status: Vec<ResultStatus> = vec![];
        let mut array_elements: im::HashMap<ValueId, Vec<usize>> = im::HashMap::new();

        for result in &results {
            match function.dfg.try_get_array_length(*result) {
                // If the result value is an array, create an empty descendant set for
                // every element to be accessed further on and record the indices
                // of the resulting sets for future reference
                Some(length)
                    if length.0 <= crate::ssa::ir::dfg::MAX_ELEMENTS.try_into().unwrap() =>
                {
                    array_elements.insert(*result, vec![]);
                    for _ in 0..length.0 {
                        array_elements[result].push(results_status.len());
                        results_status
                            .push(ResultStatus::Unconstrained { descendants: HashSet::new() });
                    }
                }
                // For very large arrays or non-arrays, treat the whole result as a single value
                // to avoid memory/time issues when tracking individual elements
                Some(_) | None => {
                    results_status.push(ResultStatus::Unconstrained {
                        descendants: HashSet::from([*result]),
                    });
                }
            }
        }

        BrilligTaintedIds {
            arguments: HashSet::from_iter(arguments.iter().copied()),
            results: results_status,
            array_elements,
            root_results: HashSet::from_iter(results.iter().copied()),
        }
    }

    /// Check if the call being tracked is a simple wrapper of another call
    fn is_wrapper(&self, other: &BrilligTaintedIds) -> bool {
        other.root_results == self.arguments
    }

    /// Add children of a given parent to the tainted value set
    /// (for arguments one set is enough, for results we keep them
    /// separate as the forthcoming check considers the call covered
    /// if all the results were properly covered)
    fn update_children(&mut self, parents: &HashSet<ValueId>, children: &[ValueId]) {
        if intersecting(&self.arguments, parents) {
            self.arguments.extend(children);
        }

        for result in &mut self.results.iter_mut() {
            match result {
                // Skip updating results already found covered
                ResultStatus::Constrained => {}
                ResultStatus::Unconstrained { descendants } => {
                    if intersecting(descendants, parents) {
                        descendants.extend(children);
                    }
                }
            }
        }
    }

    /// Update children of all the results (helper function for
    /// chained Brillig call handling)
    fn update_results_children(&mut self, children: &[ValueId]) {
        for result in &mut self.results.iter_mut() {
            match result {
                // Skip updating results already found covered
                ResultStatus::Constrained => {}
                ResultStatus::Unconstrained { descendants } => {
                    descendants.extend(children);
                }
            }
        }
    }

    /// If Brillig call is properly constrained by the given ids, return true
    fn check_constrained(&self) -> bool {
        // If every result has now been constrained,
        // consider the call properly constrained
        self.results.iter().all(|result| matches!(result, ResultStatus::Constrained))
    }

    /// Remember partial constraints (involving some of the results and an argument)
    /// along the way to take them into final consideration.
    ///
    /// Generally, a valid partial constraint should link up a result descendant
    /// and an argument descendant, that is, it should establish a relationship
    /// between the inputs and the outputs of an unconstrained call. Notably,
    /// checking the results against an independent variable is _not_ considered
    /// a partial constraint!
    ///
    /// There are two exceptions to this requirement:
    /// * if the unconstrained call had no arguments
    /// * if the value was constrained against some constant, rather than an input
    fn store_partial_constraints(&mut self, constrained_values: &HashSet<ValueId>) {
        let mut results_involved: Vec<usize> = vec![];

        // For a valid partial constraint, a value descending from
        // one of the results should be constrained
        for (i, result_status) in self.results.iter().enumerate() {
            match result_status {
                // Skip checking already covered results
                ResultStatus::Constrained => {}
                ResultStatus::Unconstrained { descendants } => {
                    if intersecting(descendants, constrained_values) {
                        results_involved.push(i);
                    }
                }
            }
        }

        if results_involved.is_empty() {
            return;
        }

        // Along with it, one of the argument descendants should be constrained
        // (skipped if there were no arguments, or if a result descendant
        // has been constrained _alone_, e.g. against a constant).
        let is_arg_constrained = intersecting(&self.arguments, constrained_values);
        let is_against_const = constrained_values.len() == 1;

        if self.arguments.is_empty() || is_arg_constrained || is_against_const {
            // Remember the partial constraint, clearing the sets
            results_involved.iter().for_each(|i| self.results[*i] = ResultStatus::Constrained);
        }
    }

    /// When an ArrayGet instruction occurs, place the resulting ValueId into
    /// the corresponding sets of the call's array element result values
    fn process_array_get(&mut self, array: ValueId, index: usize, element_results: &[ValueId]) {
        if let Some(element_indices) = self.array_elements.get(&array)
            && let Some(result_index) = element_indices.get(index)
            && let Some(ResultStatus::Unconstrained { descendants }) =
                self.results.get_mut(*result_index)
        {
            descendants.extend(element_results);
            self.root_results.extend(element_results);
        }
    }
}

impl DependencyContext {
    /// Build the dependency context of variable ValueIds, storing
    /// information on value ids involved in unchecked Brillig calls
    fn build(&mut self, function: &Function, all_functions: &BTreeMap<FunctionId, Function>) {
        self.block_queue.push(function.entry_block());
        while let Some(block) = self.block_queue.pop() {
            if !self.visited_blocks.insert(block) {
                continue;
            }
            self.process_instructions(block, function, all_functions);
        }
    }

    /// Go over the given block tracking Brillig calls and checking them against
    /// following constraints
    fn process_instructions(
        &mut self,
        block: BasicBlockId,
        function: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) {
        trace!(
            "processing instructions of block {} of function {} {}",
            block,
            function.name(),
            function.id()
        );

        // First, gather information on all Brillig calls in the block
        // to be able to follow their arguments first appearing in the
        // flow graph before the calls themselves
        function.dfg[block].instructions().iter().for_each(|instruction| {
            if let Instruction::Call { func, arguments } = &function.dfg[*instruction]
                && let Value::Function(callee) = &function.dfg[*func]
                && all_functions[callee].runtime().is_brillig()
            {
                // Skip already visited locations (happens often in unrolled functions)
                let call_stack = function.dfg.get_instruction_call_stack(*instruction);
                let location = call_stack.last();

                // If there is no call stack (happens for tests), consider unvisited
                let visited =
                    location.is_some_and(|loc| self.visited_locations.contains(&(*callee, *loc)));

                if !visited {
                    let results = function.dfg.instruction_results(*instruction);

                    // Calls with no results (e.g. print) shouldn't be checked
                    if results.is_empty() {
                        return;
                    }

                    let current_tainted = BrilligTaintedIds::new(function, arguments, results);

                    // Record arguments/results for each Brillig call for the check.
                    //
                    // Do not track Brillig calls acting as simple wrappers over
                    // another registered Brillig call, update the tainted sets of
                    // the wrapped call instead
                    let mut wrapped_call_found = false;
                    for tainted_call in self.tainted.values_mut() {
                        if current_tainted.is_wrapper(tainted_call) {
                            tainted_call.update_results_children(results);
                            wrapped_call_found = true;
                            break;
                        }
                    }

                    if !wrapped_call_found {
                        // Record the current call, remember the argument values involved
                        self.tainted.insert(*instruction, current_tainted);
                        arguments.iter().for_each(|value| {
                            self.call_arguments.entry(*value).or_default().push(*instruction);
                        });
                    }

                    if let Some(location) = location {
                        self.visited_locations.insert((*callee, *location));
                    }
                }
            }
        });

        // Then, go over the instructions
        for instruction in function.dfg[block].instructions() {
            let mut arguments = Vec::new();
            let mut results = Vec::new();

            // Collect non-constant instruction arguments
            function.dfg[*instruction].for_each_value(|value_id| {
                if !is_numeric_constant(function, value_id) {
                    arguments.push(value_id);
                }
            });

            // Collect non-constant instruction results
            for value_id in function.dfg.instruction_results(*instruction) {
                if !is_numeric_constant(function, *value_id) {
                    results.push(*value_id);
                }
            }

            // If the lookback feature is enabled, start tracking calls when
            // their value ids first appear in arguments or results, or when their
            // instruction id comes up (in case there were no non-constant arguments)
            if self.enable_lookback {
                for id in arguments.iter().chain(&results) {
                    if let Some(calls) = self.call_arguments.get(id) {
                        for call in calls {
                            if self.tainted.contains_key(call) {
                                self.tracking.insert(*call);

                                // If the instruction is a cast or truncate, also update the
                                // tainted arguments of the call with its argument
                                // (fixes #10547)
                                if matches!(
                                    &function.dfg[*instruction],
                                    Instruction::Cast(..) | Instruction::Truncate { .. }
                                ) && let Some(tainted_ids) = self.tainted.get_mut(call)
                                {
                                    tainted_ids.arguments.extend(&arguments);
                                }
                            }
                        }
                    }
                }
            }
            if self.tainted.contains_key(instruction) {
                self.tracking.insert(*instruction);
            }

            // We can skip over instructions while nothing is being tracked
            if !self.tracking.is_empty() {
                match &function.dfg[*instruction] {
                    // For memory operations, we have to link up the stored value as a parent
                    // of one loaded from the same memory slot
                    Instruction::Store { address, value } => {
                        self.memory_slots.insert(*address, *value);
                    }
                    Instruction::Load { address } => {
                        // Recall the value stored at address as parent for the results
                        if let Some(value_id) = self.memory_slots.get(address) {
                            self.update_children(&[*value_id], &results);
                        } else {
                            panic!(
                                "load instruction {instruction} has attempted to access previously unused memory location"
                            );
                        }
                    }
                    // Record the condition to set as future parent for the following values
                    Instruction::EnableSideEffectsIf { condition: value } => {
                        self.side_effects_condition =
                            (!is_numeric_constant(function, *value)).then_some(*value);
                    }
                    // Check the constrain instruction arguments against those
                    // involved in Brillig calls, remove covered calls
                    Instruction::Constrain(value_id1, value_id2, _) => {
                        let constrained_values = [*value_id1, *value_id2];
                        self.clear_constrained(&constrained_values, function);
                        // When we have `constrain v0 == v1`, then consider any follow up constraints
                        // on v0 or v1 as if it applied on both. This is because some SSA passes use
                        // constraint info to simplify values, and what was a constraint on v0 could
                        // end up being a constraint on v1.
                        self.update_children(&constrained_values, &constrained_values);
                    }
                    Instruction::ConstrainNotEqual(value_id1, value_id2, _) => {
                        self.clear_constrained(&[*value_id1, *value_id2], function);
                    }
                    // Consider range check to also be constraining
                    Instruction::RangeCheck { value, .. } => {
                        self.clear_constrained(&[*value], function);
                    }
                    Instruction::Call { func: func_id, .. } => {
                        // For functions, we remove the first element of arguments,
                        // as .for_each_value() used previously also includes func_id
                        arguments.remove(0);

                        match &function.dfg[*func_id] {
                            Value::Intrinsic(intrinsic) => match intrinsic {
                                Intrinsic::ApplyRangeConstraint | Intrinsic::AssertConstant => {
                                    // Consider these intrinsic arguments constrained
                                    self.clear_constrained(&arguments, function);
                                }
                                Intrinsic::AsWitness | Intrinsic::IsUnconstrained => {
                                    // These intrinsics won't affect the dependency graph
                                }
                                Intrinsic::ArrayLen
                                | Intrinsic::ArrayRefCount
                                | Intrinsic::ArrayAsStrUnchecked
                                | Intrinsic::AsVector
                                | Intrinsic::BlackBox(..)
                                | Intrinsic::DerivePedersenGenerators
                                | Intrinsic::Hint(..)
                                | Intrinsic::VectorPushBack
                                | Intrinsic::VectorPushFront
                                | Intrinsic::VectorPopBack
                                | Intrinsic::VectorPopFront
                                | Intrinsic::VectorRefCount
                                | Intrinsic::VectorInsert
                                | Intrinsic::VectorRemove
                                | Intrinsic::StaticAssert
                                | Intrinsic::StrAsBytes
                                | Intrinsic::ToBits(..)
                                | Intrinsic::ToRadix(..)
                                | Intrinsic::FieldLessThan => {
                                    // Record all the function arguments as parents of the results
                                    self.update_children(&arguments, &results);
                                }
                            },
                            Value::Function(callee) => match all_functions[callee].runtime() {
                                // Only update tainted sets for non-Brillig calls, as
                                // the chained Brillig case should already be covered
                                RuntimeType::Acir(..) => {
                                    self.update_children(&arguments, &results);
                                }
                                RuntimeType::Brillig(..) => {}
                            },
                            Value::ForeignFunction(..) => {
                                panic!(
                                    "should not be able to reach foreign function from non-Brillig functions, {func_id} in function {}",
                                    function.name()
                                );
                            }
                            Value::Instruction { .. }
                            | Value::NumericConstant { .. }
                            | Value::Param { .. }
                            | Value::Global(_) => {
                                panic!(
                                    "calling non-function value with ID {func_id} in function {}",
                                    function.name()
                                );
                            }
                        }
                    }
                    // For array get operations, we check the Brillig calls for
                    // results involving the array in question, to properly
                    // populate the array element tainted sets
                    Instruction::ArrayGet { array, index } => {
                        self.process_array_get(*array, *index, &results, function);
                        // Record all the used arguments as parents of the results
                        self.update_children(&arguments, &results);
                    }
                    Instruction::ArraySet { .. }
                    | Instruction::Binary(..)
                    | Instruction::Cast(..)
                    | Instruction::IfElse { .. }
                    | Instruction::Not(..)
                    | Instruction::Truncate { .. } => {
                        // Record all the used arguments as parents of the results
                        self.update_children(&arguments, &results);
                    }
                    // These instructions won't affect the dependency graph
                    Instruction::Allocate
                    | Instruction::DecrementRc { .. }
                    | Instruction::IncrementRc { .. }
                    | Instruction::MakeArray { .. }
                    | Instruction::Noop => {}
                }
            }
        }

        if !self.tainted.is_empty() {
            trace!(
                "number of Brillig calls in function {} {} left unchecked: {}",
                function.name(),
                function.id(),
                self.tainted.len()
            );
        }
    }

    /// Every Brillig call not properly constrained should remain in the tainted set
    /// at this point. For each, emit a corresponding warning.
    fn collect_warnings(&self, function: &Function) -> Vec<SsaReport> {
        let warnings: Vec<SsaReport> = self
            .tainted
            .keys()
            .map(|brillig_call| {
                trace!(
                    "tainted structure for {:?}: {:?}",
                    brillig_call, self.tainted[brillig_call]
                );
                SsaReport::Bug(InternalBug::UncheckedBrilligCall {
                    call_stack: function.dfg.get_instruction_call_stack(*brillig_call),
                })
            })
            .collect();

        trace!(
            "making {} reports on underconstrained Brillig calls for function {} {}",
            warnings.len(),
            function.name(),
            function.id()
        );
        warnings
    }

    /// Update sets of value ids that can be traced back to the Brillig calls being tracked
    fn update_children(&mut self, parents: &[ValueId], children: &[ValueId]) {
        let mut parents: HashSet<_> = HashSet::from_iter(parents.iter().copied());

        // Also include the current EnableSideEffectsIf condition in parents
        // (as it would affect every following statement)
        self.side_effects_condition.map(|v| parents.insert(v));

        // Don't update sets for the calls not yet being tracked
        for call in &self.tracking {
            if let Some(tainted_ids) = self.tainted.get_mut(call) {
                tainted_ids.update_children(&parents, children);
            }
        }
    }

    /// Check if any of the recorded Brillig calls have been properly constrained
    /// by given values after recording partial constraints, if so stop tracking them
    fn clear_constrained(&mut self, constrained_values: &[ValueId], function: &Function) {
        // Remove numeric constants
        let constrained_values: HashSet<_> = constrained_values
            .iter()
            .filter(|v| !is_numeric_constant(function, **v))
            .copied()
            .collect();

        // Skip untracked calls
        for call in &self.tracking {
            if let Some(tainted_ids) = self.tainted.get_mut(call) {
                tainted_ids.store_partial_constraints(&constrained_values);
            }
        }

        self.tainted.retain(|call, tainted_ids| {
            if tainted_ids.check_constrained() {
                self.tracking.remove(call);
                false
            } else {
                true
            }
        });
    }

    /// Process ArrayGet instruction for tracked Brillig calls
    fn process_array_get(
        &mut self,
        array: ValueId,
        index: ValueId,
        element_results: &[ValueId],
        function: &Function,
    ) {
        use acvm::acir::AcirField;

        // Only allow numeric constant indices
        if let Some(value) = function.dfg.get_numeric_constant(index)
            && let Some(index) = value.try_to_u32()
        {
            // Skip untracked calls
            for call in &self.tracking {
                if let Some(tainted_ids) = self.tainted.get_mut(call) {
                    tainted_ids.process_array_get(array, index as usize, element_results);
                }
            }
        }
    }
}

/// Return `true` if two sets have a non-empty intersection.
fn intersecting<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> bool {
    a.intersection(b).next().is_some()
}

#[cfg(test)]
mod tests {
    use crate::ssa::Ssa;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    /// Test where a call to a Brillig function is left unchecked with a later assert,
    /// by example of the program illustrating issue #5425 (simplified variant).
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
            inc_rc v4
            inc_rc v5
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
            dec_rc v4
            dec_rc v5
            return
        }

        brillig(inline) fn maximum_price f1 {
          b0(v0: [u32; 2]):
            v2 = array_get v0, index u32 0 -> u32
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 2);
    }

    #[test]
    #[traced_test]
    /// Test EnableSideEffectsIf conditions affecting the dependency graph
    /// (SSA a bit convoluted to work around simplification breaking the flow
    /// of the parsed test code)
    fn test_enable_side_effects_if_affecting_following_statements() {
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test chained (wrapper) Brillig calls not producing a false positive
    fn test_chained_brillig_calls_constrained() {
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
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
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    #[should_panic = "Expected no warnings but found some."]
    fn constrain_on_array_element_links_to_input_array() {
        // Regression test for https://github.com/noir-lang/noir/issues/11807
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = make_array [v0] : [Field; 1]
            v3 = call f1(v1) -> [Field; 1]          // We pass v1 = [v0] to the Brillig function.
            v5 = array_get v3, index u32 0 -> Field
            constrain v5 == v0                      // We constrain the result directly against v0, which should clear the Brillig call.
            return v3
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 1]):
            v2 = array_get v0, index u32 1 minus 1 -> Field
            v3 = make_array [v2] : [Field; 1]
            return v3
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(ssa_level_warnings.len(), 0, "Expected no warnings but found some.");
    }
}
