//! This module defines security SSA passes detecting constraint problems leading to possible
//! soundness vulnerabilities.
//! The compiler informs the developer of these as bugs.
use crate::errors::{InternalBug, SsaReport};
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::function::RuntimeType;
use crate::ssa::ir::function::{Function, FunctionId};
use crate::ssa::ir::instruction::{Hint, Instruction, InstructionId, Intrinsic};
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::ssa_gen::Ssa;
use im::HashMap;
use noirc_errors::Location;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use tracing::trace;

/// The number of instructions that have to be passed to stop
/// following a Brillig call, with assumption it wouldn't get constrained
const BRILLIG_CONSTRAINT_SEARCH_DEPTH: usize = 10_000_000;

impl Ssa {
    /// This function provides an SSA pass that detects if the final function has any subgraphs independent from inputs and outputs.
    /// If this is the case, then part of the final circuit can be completely replaced by any other passing circuit, since there are no constraints ensuring connections.
    /// Go through each top-level non-Brillig function and detect if it has independent subgraphs
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn check_for_underconstrained_values(&mut self) -> Vec<SsaReport> {
        self.functions
            .values()
            .map(|f| f.id())
            .par_bridge()
            .flat_map(|fid| {
                let function_to_process = &self.functions[&fid];
                match function_to_process.runtime() {
                    RuntimeType::Acir { .. } => check_for_underconstrained_values_within_function(
                        function_to_process,
                        &self.functions,
                    ),
                    RuntimeType::Brillig(_) => Vec::new(),
                }
            })
            .collect()
    }

    /// Detect Brillig calls left unconstrained with manual asserts
    /// and return a vector of bug reports if any have been found
    pub(crate) fn check_for_missing_brillig_constraints(
        &mut self,
        enable_lookback: bool,
    ) -> Vec<SsaReport> {
        // Skip the check if there are no Brillig functions involved
        if !self.functions.values().any(|func| func.runtime().is_brillig()) {
            return vec![];
        };

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

/// Detect independent subgraphs (not connected to function inputs or outputs) and return a vector of bug reports if some are found
fn check_for_underconstrained_values_within_function(
    function: &Function,
    all_functions: &BTreeMap<FunctionId, Function>,
) -> Vec<SsaReport> {
    let mut context = Context::default();
    let mut warnings: Vec<SsaReport> = Vec::new();

    context.compute_sets_of_connected_value_ids(function, all_functions);

    let all_brillig_generated_values: BTreeSet<ValueId> =
        context.brillig_return_to_argument.keys().copied().collect();

    let connected_sets_indices =
        context.find_sets_connected_to_function_inputs_or_outputs(function);

    // Go through each disconnected set, find Brillig calls that caused it and form warnings
    for set_index in
        BTreeSet::from_iter(0..(context.value_sets.len())).difference(&connected_sets_indices)
    {
        let current_set = &context.value_sets[*set_index];
        warnings.append(&mut context.find_disconnecting_brillig_calls_with_results_in_set(
            current_set,
            &all_brillig_generated_values,
            function,
        ));
    }
    warnings
}

#[derive(Default)]
struct DependencyContext {
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
    // Map keeping track of values stored at memory locations
    memory_slots: HashMap<ValueId, ValueId>,
    // Value currently affecting every instruction (i.e. being
    // considered a parent of every value id met) because
    // of its involvement in an EnableSideEffectsIf condition
    side_effects_condition: Option<ValueId>,
    // Map of Brillig call ids to sets of the value ids descending
    // from their arguments and results
    tainted: BTreeMap<InstructionId, BrilligTaintedIds>,
    // Map of argument value ids to the Brillig call ids employing them
    call_arguments: HashMap<ValueId, Vec<InstructionId>>,
    // Maintains count of calls being tracked
    tracking_count: usize,
    // Map of block indices to Brillig call ids that should not be
    // followed after meeting them
    search_limits: HashMap<usize, InstructionId>,
    // Opt-in to use the lookback feature (tracking the argument values
    // of a Brillig call before the call happens if their usage precedes
    // it). Can prevent certain false positives, at the cost of
    // slowing down checking large functions considerably
    enable_lookback: bool,
    // Code locations of brillig calls already visited (we don't
    // need to recheck calls happening in the same unrolled functions)
    visited_locations: HashSet<Location>,
}

/// Structure keeping track of value ids descending from Brillig calls'
/// arguments and results, also storing information on results
/// already properly constrained
#[derive(Clone, Debug)]
struct BrilligTaintedIds {
    // Argument descendant value ids
    arguments: HashSet<ValueId>,
    // Results status
    results: Vec<ResultStatus>,
    // Indices of the array elements in the results vector
    array_elements: HashMap<ValueId, Vec<usize>>,
    // Initial result value ids, along with element ids for arrays
    root_results: HashSet<ValueId>,
    // The flag signaling that the call should be now tracked
    tracking: bool,
}

#[derive(Clone, Debug)]
enum ResultStatus {
    // Keep track of descendants until found constrained
    Unconstrained { descendants: HashSet<ValueId> },
    Constrained,
}

impl BrilligTaintedIds {
    fn new(function: &Function, arguments: &[ValueId], results: &[ValueId]) -> Self {
        // Exclude numeric constants
        let arguments: Vec<ValueId> = arguments
            .iter()
            .filter(|value| function.dfg.get_numeric_constant(**value).is_none())
            .copied()
            .map(|value| function.dfg.resolve(value))
            .collect();
        let results: Vec<ValueId> = results
            .iter()
            .filter(|value| function.dfg.get_numeric_constant(**value).is_none())
            .copied()
            .map(|value| function.dfg.resolve(value))
            .collect();

        let mut results_status: Vec<ResultStatus> = vec![];
        let mut array_elements: HashMap<ValueId, Vec<usize>> = HashMap::new();

        for result in &results {
            match function.dfg.try_get_array_length(*result) {
                // If the result value is an array, create an empty descendant set for
                // every element to be accessed further on and record the indices
                // of the resulting sets for future reference
                Some(length) => {
                    array_elements.insert(*result, vec![]);
                    for _ in 0..length {
                        array_elements[result].push(results_status.len());
                        results_status
                            .push(ResultStatus::Unconstrained { descendants: HashSet::new() });
                    }
                }
                // Otherwise initialize a descendant set with the current value
                None => {
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
            tracking: false,
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
        if self.arguments.intersection(parents).next().is_some() {
            self.arguments.extend(children);
        }

        for result in &mut self.results.iter_mut() {
            match result {
                // Skip updating results already found covered
                ResultStatus::Constrained => {}
                ResultStatus::Unconstrained { descendants } => {
                    if descendants.intersection(parents).next().is_some() {
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
    /// along the way to take them into final consideration
    /// Generally, a valid partial constraint should link up a result descendant
    /// and an argument descendant, although there are also edge cases mentioned below.
    fn store_partial_constraints(&mut self, constrained_values: &HashSet<ValueId>) {
        let mut results_involved: Vec<usize> = vec![];

        // For a valid partial constraint, a value descending from
        // one of the results should be constrained
        for (i, result_status) in self.results.iter().enumerate() {
            match result_status {
                // Skip checking already covered results
                ResultStatus::Constrained => {}
                ResultStatus::Unconstrained { descendants } => {
                    if descendants.intersection(constrained_values).next().is_some() {
                        results_involved.push(i);
                    }
                }
            }
        }

        // Along with it, one of the argument descendants should be constrained
        // (skipped if there were no arguments, or if a result descendant
        // has been constrained _alone_, e.g. against a constant)
        if !results_involved.is_empty()
            && (self.arguments.is_empty()
                || (constrained_values.len() == 1)
                || self.arguments.intersection(constrained_values).next().is_some())
        {
            // Remember the partial constraint, clearing the sets
            results_involved.iter().for_each(|i| self.results[*i] = ResultStatus::Constrained);
        }
    }

    /// When an ArrayGet instruction occurs, place the resulting ValueId into
    /// the corresponding sets of the call's array element result values
    fn process_array_get(&mut self, array: ValueId, index: usize, element_results: &[ValueId]) {
        if let Some(element_indices) = self.array_elements.get(&array) {
            if let Some(result_index) = element_indices.get(index) {
                if let Some(ResultStatus::Unconstrained { descendants }) =
                    self.results.get_mut(*result_index)
                {
                    descendants.extend(element_results);
                    self.root_results.extend(element_results);
                }
            }
        }
    }
}

impl DependencyContext {
    /// Build the dependency context of variable ValueIds, storing
    /// information on value ids involved in unchecked Brillig calls
    fn build(&mut self, function: &Function, all_functions: &BTreeMap<FunctionId, Function>) {
        self.block_queue.push(function.entry_block());
        while let Some(block) = self.block_queue.pop() {
            if self.visited_blocks.contains(&block) {
                continue;
            }
            self.visited_blocks.insert(block);
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
        trace!("processing instructions of block {} of function {}", block, function.id());

        // First, gather information on all Brillig calls in the block
        // to be able to follow their arguments first appearing in the
        // flow graph before the calls themselves
        function.dfg[block].instructions().iter().enumerate().for_each(
            |(block_index, instruction)| {
                if let Instruction::Call { func, arguments } = &function.dfg[*instruction] {
                    if let Value::Function(callee) = &function.dfg[*func] {
                        if all_functions[&callee].runtime().is_brillig() {
                            // Skip already visited locations (happens often in unrolled functions)
                            let call_stack = function.dfg.get_instruction_call_stack(*instruction);
                            let location = call_stack.last();

                            // If there is no call stack (happens for tests), consider unvisited
                            let mut visited = false;
                            if let Some(location) = location {
                                visited = self.visited_locations.contains(location);
                            }

                            if !visited {
                                let results = function.dfg.instruction_results(*instruction);
                                let current_tainted =
                                    BrilligTaintedIds::new(function, arguments, results);

                                // Record arguments/results for each Brillig call for the check.
                                //
                                // Do not track Brillig calls acting as simple wrappers over
                                // another registered Brillig call, update the tainted sets of
                                // the wrapped call instead
                                let mut wrapped_call_found = false;
                                for (_, tainted_call) in self.tainted.iter_mut() {
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
                                        self.call_arguments
                                            .entry(*value)
                                            .or_default()
                                            .push(*instruction);
                                    });

                                    // Set the constraint search limit for the call
                                    self.search_limits.insert(
                                        block_index + BRILLIG_CONSTRAINT_SEARCH_DEPTH,
                                        *instruction,
                                    );
                                }

                                if let Some(location) = location {
                                    self.visited_locations.insert(*location);
                                }
                            }
                        }
                    }
                }
            },
        );

        //Then, go over the instructions
        for (block_index, instruction) in function.dfg[block].instructions().iter().enumerate() {
            let mut arguments = Vec::new();

            // Collect non-constant instruction arguments
            function.dfg[*instruction].for_each_value(|value_id| {
                if function.dfg.get_numeric_constant(value_id).is_none() {
                    arguments.push(function.dfg.resolve(value_id));
                }
            });

            // If the lookback feature is enabled, start tracking calls when
            // their argument value ids first appear, or when their
            // instruction id comes up (in case there were no non-constant arguments)
            if self.enable_lookback {
                for argument in &arguments {
                    if let Some(calls) = self.call_arguments.get(argument) {
                        for call in calls {
                            if let Some(tainted_ids) = self.tainted.get_mut(call) {
                                tainted_ids.tracking = true;
                                self.tracking_count += 1;
                            }
                        }
                    }
                }
            }
            if let Some(tainted_ids) = self.tainted.get_mut(instruction) {
                tainted_ids.tracking = true;
                self.tracking_count += 1;
            }

            // Stop tracking calls when their search limit is hit
            if let Some(call) = self.search_limits.get(&block_index) {
                if let Some(tainted_ids) = self.tainted.get_mut(call) {
                    tainted_ids.tracking = false;
                    self.tracking_count -= 1;
                }
            }

            // We can skip over instructions while nothing is being tracked
            if self.tracking_count > 0 {
                let mut results = Vec::new();

                // Collect non-constant instruction results
                for value_id in function.dfg.instruction_results(*instruction).iter() {
                    if function.dfg.get_numeric_constant(*value_id).is_none() {
                        results.push(function.dfg.resolve(*value_id));
                    }
                }

                match &function.dfg[*instruction] {
                    // For memory operations, we have to link up the stored value as a parent
                    // of one loaded from the same memory slot
                    Instruction::Store { address, value } => {
                        self.memory_slots.insert(*address, function.dfg.resolve(*value));
                    }
                    Instruction::Load { address } => {
                        // Recall the value stored at address as parent for the results
                        if let Some(value_id) = self.memory_slots.get(address) {
                            self.update_children(&[*value_id], &results);
                        } else {
                            panic!("load instruction {} has attempted to access previously unused memory location",
                                instruction);
                        }
                    }
                    // Record the condition to set as future parent for the following values
                    Instruction::EnableSideEffectsIf { condition: value } => {
                        self.side_effects_condition =
                            match function.dfg.get_numeric_constant(*value) {
                                None => Some(function.dfg.resolve(*value)),
                                Some(_) => None,
                            }
                    }
                    // Check the constrain instruction arguments against those
                    // involved in Brillig calls, remove covered calls
                    Instruction::Constrain(value_id1, value_id2, _)
                    | Instruction::ConstrainNotEqual(value_id1, value_id2, _) => {
                        self.clear_constrained(
                            &[function.dfg.resolve(*value_id1), function.dfg.resolve(*value_id2)],
                            function,
                        );
                    }
                    // Consider range check to also be constraining
                    Instruction::RangeCheck { value, .. } => {
                        self.clear_constrained(&[function.dfg.resolve(*value)], function);
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
                                | Intrinsic::AsSlice
                                | Intrinsic::BlackBox(..)
                                | Intrinsic::DerivePedersenGenerators
                                | Intrinsic::Hint(..)
                                | Intrinsic::SlicePushBack
                                | Intrinsic::SlicePushFront
                                | Intrinsic::SlicePopBack
                                | Intrinsic::SlicePopFront
                                | Intrinsic::SliceRefCount
                                | Intrinsic::SliceInsert
                                | Intrinsic::SliceRemove
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
                                panic!("should not be able to reach foreign function from non-Brillig functions, {func_id} in function {}", function.name());
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
                        self.process_array_get(function, *array, *index, &results);
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
                    Instruction::Allocate { .. }
                    | Instruction::DecrementRc { .. }
                    | Instruction::IncrementRc { .. }
                    | Instruction::MakeArray { .. }
                    | Instruction::Noop => {}
                }
            }
        }

        if !self.tainted.is_empty() {
            trace!(
                "number of Brillig calls in function {} left unchecked: {}",
                function,
                self.tainted.len()
            );
        }
    }

    /// Every Brillig call not properly constrained should remain in the tainted set
    /// at this point. For each, emit a corresponding warning.
    fn collect_warnings(&mut self, function: &Function) -> Vec<SsaReport> {
        let warnings: Vec<SsaReport> = self
            .tainted
            .keys()
            .map(|brillig_call| {
                trace!("tainted structure for {}: {:?}", brillig_call, self.tainted[brillig_call]);
                SsaReport::Bug(InternalBug::UncheckedBrilligCall {
                    call_stack: function.dfg.get_instruction_call_stack(*brillig_call),
                })
            })
            .collect();

        trace!(
            "making {} reports on underconstrained Brillig calls for function {}",
            warnings.len(),
            function.name()
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
        for (_, tainted_ids) in self.tainted.iter_mut() {
            if tainted_ids.tracking {
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
            .filter(|v| function.dfg.get_numeric_constant(**v).is_none())
            .copied()
            .collect();

        // Skip untracked calls
        for (_, tainted_ids) in self.tainted.iter_mut() {
            if tainted_ids.tracking {
                tainted_ids.store_partial_constraints(&constrained_values);
            }
        }

        self.tainted.retain(|_, tainted_ids| !tainted_ids.check_constrained());
    }

    /// Process ArrayGet instruction for tracked Brillig calls
    fn process_array_get(
        &mut self,
        function: &Function,
        array: ValueId,
        index: ValueId,
        element_results: &[ValueId],
    ) {
        use acvm::acir::AcirField;

        // Only allow numeric constant indices
        if let Some(value) = function.dfg.get_numeric_constant(index) {
            if let Some(index) = value.try_to_u32() {
                // Skip untracked calls
                for (_, tainted_ids) in self.tainted.iter_mut() {
                    if tainted_ids.tracking {
                        tainted_ids.process_array_get(array, index as usize, element_results);
                    }
                }
            }
        }
    }
}

#[derive(Default)]
struct Context {
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
    value_sets: Vec<BTreeSet<ValueId>>,
    brillig_return_to_argument: HashMap<ValueId, Vec<ValueId>>,
    brillig_return_to_instruction_id: HashMap<ValueId, InstructionId>,
}

impl Context {
    /// Compute sets of variable ValueIds that are connected with constraints
    ///
    /// Additionally, store information about Brillig calls in the context
    fn compute_sets_of_connected_value_ids(
        &mut self,
        function: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) {
        // Go through each block in the function and create a list of sets of ValueIds connected by instructions
        self.block_queue.push(function.entry_block());
        while let Some(block) = self.block_queue.pop() {
            if self.visited_blocks.contains(&block) {
                continue;
            }
            self.visited_blocks.insert(block);
            self.connect_value_ids_in_block(function, block, all_functions);
        }
        // Merge ValueIds into sets, where each original small set of ValueIds is merged with another set if they intersect
        self.value_sets = Self::merge_sets_par(&self.value_sets);
    }

    /// Find sets that contain input or output value of the function
    ///
    /// Goes through each set of connected ValueIds and see if function arguments or return values are in the set
    fn find_sets_connected_to_function_inputs_or_outputs(
        &mut self,
        function: &Function,
    ) -> BTreeSet<usize> {
        let returns = function.returns();
        let variable_parameters_and_return_values = function
            .parameters()
            .iter()
            .chain(returns)
            .filter(|id| function.dfg.get_numeric_constant(**id).is_none())
            .map(|value_id| function.dfg.resolve(*value_id));

        let mut connected_sets_indices: BTreeSet<usize> = BTreeSet::default();

        // Go through each parameter and each set and check if the set contains the parameter
        // If it's the case, then that set doesn't present an issue
        for parameter_or_return_value in variable_parameters_and_return_values {
            for (set_index, final_set) in self.value_sets.iter().enumerate() {
                if final_set.contains(&parameter_or_return_value) {
                    connected_sets_indices.insert(set_index);
                }
            }
        }
        connected_sets_indices
    }

    /// Find which Brillig calls separate this set from others and return bug warnings about them
    fn find_disconnecting_brillig_calls_with_results_in_set(
        &self,
        current_set: &BTreeSet<ValueId>,
        all_brillig_generated_values: &BTreeSet<ValueId>,
        function: &Function,
    ) -> Vec<SsaReport> {
        let mut warnings = Vec::new();
        // Find Brillig-generated values in the set
        let intersection = all_brillig_generated_values.intersection(current_set).copied();

        // Go through all Brillig outputs in the set
        for brillig_output_in_set in intersection {
            // Get the inputs that correspond to the output
            let inputs: BTreeSet<ValueId> =
                self.brillig_return_to_argument[&brillig_output_in_set].iter().copied().collect();

            // Check if any of them are not in the set
            let unused_inputs = inputs.difference(current_set).next().is_some();

            // There is a value not in the set, which means that the inputs/outputs of this call have not been properly constrained
            if unused_inputs {
                warnings.push(SsaReport::Bug(InternalBug::IndependentSubgraph {
                    call_stack: function.dfg.get_instruction_call_stack(
                        self.brillig_return_to_instruction_id[&brillig_output_in_set],
                    ),
                }));
            }
        }
        warnings
    }
    /// Go through each instruction in the block and add a set of ValueIds connected through that instruction
    ///
    /// Additionally, this function adds mappings of Brillig return values to call arguments and instruction ids from calls to Brillig functions in the block
    fn connect_value_ids_in_block(
        &mut self,
        function: &Function,
        block: BasicBlockId,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) {
        let instructions = function.dfg[block].instructions();

        for instruction in instructions.iter() {
            let mut instruction_arguments_and_results = BTreeSet::new();

            // Insert non-constant instruction arguments
            function.dfg[*instruction].for_each_value(|value_id| {
                if function.dfg.get_numeric_constant(value_id).is_none() {
                    instruction_arguments_and_results.insert(function.dfg.resolve(value_id));
                }
            });
            // And non-constant results
            for value_id in function.dfg.instruction_results(*instruction).iter() {
                if function.dfg.get_numeric_constant(*value_id).is_none() {
                    instruction_arguments_and_results.insert(function.dfg.resolve(*value_id));
                }
            }

            // For most instructions we just connect inputs and outputs
            match &function.dfg[*instruction] {
                Instruction::ArrayGet { .. }
                | Instruction::ArraySet { .. }
                | Instruction::Binary(..)
                | Instruction::Cast(..)
                | Instruction::Constrain(..)
                | Instruction::ConstrainNotEqual(..)
                | Instruction::IfElse { .. }
                | Instruction::Load { .. }
                | Instruction::Not(..)
                | Instruction::Store { .. }
                | Instruction::Truncate { .. }
                | Instruction::MakeArray { .. } => {
                    self.value_sets.push(instruction_arguments_and_results);
                }

                Instruction::Call { func: func_id, arguments: argument_ids } => {
                    match &function.dfg[*func_id] {
                        Value::Intrinsic(intrinsic) => match intrinsic {
                            Intrinsic::ApplyRangeConstraint
                            | Intrinsic::AssertConstant
                            | Intrinsic::AsWitness
                            | Intrinsic::IsUnconstrained => {}
                            Intrinsic::ArrayLen
                            | Intrinsic::ArrayAsStrUnchecked
                            | Intrinsic::ArrayRefCount
                            | Intrinsic::AsSlice
                            | Intrinsic::BlackBox(..)
                            | Intrinsic::Hint(Hint::BlackBox)
                            | Intrinsic::DerivePedersenGenerators
                            | Intrinsic::SliceInsert
                            | Intrinsic::SlicePushBack
                            | Intrinsic::SlicePushFront
                            | Intrinsic::SlicePopBack
                            | Intrinsic::SlicePopFront
                            | Intrinsic::SliceRefCount
                            | Intrinsic::SliceRemove
                            | Intrinsic::StaticAssert
                            | Intrinsic::StrAsBytes
                            | Intrinsic::ToBits(..)
                            | Intrinsic::ToRadix(..)
                            | Intrinsic::FieldLessThan => {
                                self.value_sets.push(instruction_arguments_and_results);
                            }
                        },
                        Value::Function(callee) => match all_functions[callee].runtime() {
                            RuntimeType::Brillig(_) => {
                                // For calls to Brillig functions we memorize the mapping of results to argument ValueId's and InstructionId's
                                // The latter are needed to produce the callstack later
                                for result in
                                    function.dfg.instruction_results(*instruction).iter().filter(
                                        |value_id| {
                                            function.dfg.get_numeric_constant(**value_id).is_none()
                                        },
                                    )
                                {
                                    self.brillig_return_to_argument
                                        .insert(*result, argument_ids.clone());
                                    self.brillig_return_to_instruction_id
                                        .insert(*result, *instruction);
                                }
                            }
                            RuntimeType::Acir(..) => {
                                self.value_sets.push(instruction_arguments_and_results);
                            }
                        },
                        Value::ForeignFunction(..) => {
                            panic!("Should not be able to reach foreign function from non-Brillig functions, {func_id} in function {}", function.name());
                        }
                        Value::Instruction { .. }
                        | Value::NumericConstant { .. }
                        | Value::Param { .. }
                        | Value::Global(_) => {
                            panic!("At the point we are running disconnect there shouldn't be any other values as arguments")
                        }
                    }
                }
                Instruction::Allocate { .. }
                | Instruction::DecrementRc { .. }
                | Instruction::EnableSideEffectsIf { .. }
                | Instruction::IncrementRc { .. }
                | Instruction::Noop
                | Instruction::RangeCheck { .. } => {}
            }
        }

        self.block_queue.extend(function.dfg[block].successors());
    }

    /// Merge all small sets into larger ones based on whether the sets intersect or not
    ///
    /// If two small sets have a common ValueId, we merge them into one
    fn merge_sets(current: &[BTreeSet<ValueId>]) -> Vec<BTreeSet<ValueId>> {
        let mut new_set_id: usize = 0;
        let mut updated_sets: BTreeMap<usize, BTreeSet<ValueId>> = BTreeMap::default();
        let mut value_dictionary: HashMap<ValueId, usize> = HashMap::default();
        let mut parsed_value_set: BTreeSet<ValueId> = BTreeSet::default();

        for set in current.iter() {
            // Check if the set has any of the ValueIds we've encountered at previous iterations
            let intersection: BTreeSet<ValueId> =
                set.intersection(&parsed_value_set).copied().collect();
            parsed_value_set.extend(set.iter());

            // If there is no intersection, add the new set to updated sets
            if intersection.is_empty() {
                updated_sets.insert(new_set_id, set.clone());

                // Add each entry to a dictionary for quick lookups of which ValueId is in which updated set
                for entry in set.iter() {
                    value_dictionary.insert(*entry, new_set_id);
                }
                new_set_id += 1;
                continue;
            }

            // If there is an intersection, we have to join the sets
            let mut joining_sets_ids: BTreeSet<usize> =
                intersection.iter().map(|x| value_dictionary[x]).collect();
            let mut largest_set_size = usize::MIN;
            let mut largest_set_index = usize::MAX;

            // We need to find the largest set to move as few elements as possible
            for set_id in joining_sets_ids.iter() {
                if updated_sets[set_id].len() > largest_set_size {
                    (largest_set_index, largest_set_size) = (*set_id, updated_sets[set_id].len());
                }
            }
            joining_sets_ids.remove(&largest_set_index);

            let mut largest_set =
                updated_sets.remove(&largest_set_index).expect("Set should be in the hashmap");

            // For each of other sets that need to be joined
            for set_id in joining_sets_ids.iter() {
                // Map each element to the largest set and insert it
                for element in updated_sets[set_id].iter() {
                    value_dictionary[element] = largest_set_index;
                    largest_set.insert(*element);
                }
                // Remove the old set
                updated_sets.remove(set_id);
            }

            // Join the new set with the largest sets
            for element in set.iter() {
                value_dictionary.insert(*element, largest_set_index);
                largest_set.insert(*element);
            }
            updated_sets.insert(largest_set_index, largest_set);
        }
        updated_sets.values().cloned().collect()
    }

    /// Parallel version of merge_sets
    /// The sets are merged by chunks, and then the chunks are merged together
    fn merge_sets_par(sets: &[BTreeSet<ValueId>]) -> Vec<BTreeSet<ValueId>> {
        let mut sets = sets.to_owned();
        let mut len = sets.len();
        let mut prev_len = len + 1;

        while len > 1000 && len < prev_len {
            sets = sets.par_chunks(1000).flat_map(Self::merge_sets).collect();

            prev_len = len;
            len = sets.len();
        }
        // TODO: if prev_len >= len, this means we cannot effectively merge the sets anymore
        // We should instead partition the sets into disjoint chunks and work on those chunks,
        // but for now we fallback to the non-parallel implementation
        Self::merge_sets(&sets)
    }
}
#[cfg(test)]
mod test {
    use crate::ssa::Ssa;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    /// Test that a connected function raises no warnings
    fn test_simple_connected_function() {
        let program = r#"
        acir(inline) fn main f0 {
            b0(v0: Field, v1: Field):
                v2 = add v0, Field 1
                v3 = mul v1, Field 2
                v4 = eq v2, v3
                return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_underconstrained_values();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where the results of a call to a Brillig function are not connected to main function inputs or outputs
    /// This should be detected.
    fn test_simple_function_with_disconnected_part() {
        let program = r#"
        acir(inline) fn main f0 {
            b0(v0: Field, v1: Field):
                v2 = add v0, Field 1
                v3 = mul v1, Field 2
                v4 = call f1(v2, v3) -> Field
                v5 = add v4, Field 2
                return
        }
        
        brillig(inline) fn br f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_underconstrained_values();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

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
            v9 = allocate -> &mut u32
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
}
