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
use rayon::prelude::*;
use std::collections::{BTreeMap, HashSet};
use tracing::trace;

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
    pub(crate) fn check_for_missing_brillig_constraints(&mut self) -> Vec<SsaReport> {
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
                        let mut context = DependencyContext::default();
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

    let all_brillig_generated_values: HashSet<ValueId> =
        context.brillig_return_to_argument.keys().copied().collect();

    let connected_sets_indices =
        context.find_sets_connected_to_function_inputs_or_outputs(function);

    // Go through each disconnected set, find brillig calls that caused it and form warnings
    for set_index in
        HashSet::from_iter(0..(context.value_sets.len())).difference(&connected_sets_indices)
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
    // Map of values resulting from array get instructions
    // to the actual array values
    array_elements: HashMap<ValueId, ValueId>,
    // Map of brillig call ids to sets of the value ids descending
    // from their arguments and results
    tainted: HashMap<InstructionId, BrilligTaintedIds>,
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
    // Initial result value ids
    root_results: HashSet<ValueId>,
}

#[derive(Clone, Debug)]
enum ResultStatus {
    // Keep track of descendants until found constrained
    Unconstrained { descendants: HashSet<ValueId> },
    Constrained,
}

impl BrilligTaintedIds {
    fn new(arguments: &[ValueId], results: &[ValueId]) -> Self {
        BrilligTaintedIds {
            arguments: HashSet::from_iter(arguments.iter().copied()),
            results: results
                .iter()
                .map(|result| ResultStatus::Unconstrained { descendants: HashSet::from([*result]) })
                .collect(),
            root_results: HashSet::from_iter(results.iter().copied()),
        }
    }

    /// Add children of a given parent to the tainted value set
    /// (for arguments one set is enough, for results we keep them
    /// separate as the forthcoming check considers the call covered
    /// if all the results were properly covered)
    fn update_children(&mut self, parents: &HashSet<ValueId>, children: &[ValueId]) {
        if self.arguments.intersection(parents).next().is_some() {
            self.arguments.extend(children);
        }
        for result_status in &mut self.results.iter_mut() {
            match result_status {
                // Skip updating results already found covered
                ResultStatus::Constrained => {
                    continue;
                }
                ResultStatus::Unconstrained { descendants } => {
                    if descendants.intersection(parents).next().is_some() {
                        descendants.extend(children);
                    }
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
                ResultStatus::Constrained => {
                    continue;
                }
                ResultStatus::Unconstrained { descendants } => {
                    if descendants.intersection(constrained_values).next().is_some() {
                        results_involved.push(i);
                    }
                }
            }
        }

        // Along with it, one of the argument descendants should be constrained
        // (skipped if there were no arguments, or if an actual result and not a
        // descendant has been constrained _alone_, e.g. against a constant)
        if !results_involved.is_empty()
            && (self.arguments.is_empty()
                || (constrained_values.len() == 1
                    && self.root_results.intersection(constrained_values).next().is_some())
                || self.arguments.intersection(constrained_values).next().is_some())
        {
            // Remember the partial constraint, clearing the sets
            results_involved.iter().for_each(|i| self.results[*i] = ResultStatus::Constrained);
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

        for instruction in function.dfg[block].instructions() {
            let mut arguments = Vec::new();
            let mut results = Vec::new();

            // Collect non-constant instruction arguments
            function.dfg[*instruction].for_each_value(|value_id| {
                if function.dfg.get_numeric_constant(value_id).is_none() {
                    arguments.push(function.dfg.resolve(value_id));
                }
            });

            // Collect non-constant instruction results
            for value_id in function.dfg.instruction_results(*instruction).iter() {
                if function.dfg.get_numeric_constant(*value_id).is_none() {
                    results.push(function.dfg.resolve(*value_id));
                }
            }

            // Process instructions
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
                // Check the constrain instruction arguments against those
                // involved in Brillig calls, remove covered calls
                Instruction::Constrain(value_id1, value_id2, _) => {
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
                            | Intrinsic::AsField
                            | Intrinsic::AsSlice
                            | Intrinsic::BlackBox(..)
                            | Intrinsic::DerivePedersenGenerators
                            | Intrinsic::FromField
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
                        Value::Function(callee) => match all_functions[&callee].runtime() {
                            RuntimeType::Brillig(_) => {
                                // Record arguments/results for each Brillig call for the check

                                self.tainted.insert(
                                    *instruction,
                                    BrilligTaintedIds::new(&arguments, &results),
                                );
                            }
                            RuntimeType::Acir(..) => {
                                // Record all the function arguments as parents of the results
                                self.update_children(&arguments, &results);
                            }
                        },
                        Value::ForeignFunction(..) => {
                            panic!("should not be able to reach foreign function from non-Brillig functions, {func_id} in function {}", function.name());
                        }
                        Value::Instruction { .. }
                        | Value::NumericConstant { .. }
                        | Value::Param { .. } => {
                            panic!(
                                "calling non-function value with ID {func_id} in function {}",
                                function.name()
                            );
                        }
                    }
                }
                // For array get operations, we link the resulting values to
                // the corresponding array value ids
                // (this is required later because for now we consider array elements
                // being constrained as valid as the whole arrays being constrained)
                Instruction::ArrayGet { array, .. } => {
                    for result in &results {
                        self.array_elements.insert(*result, function.dfg.resolve(*array));
                    }
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
                | Instruction::EnableSideEffectsIf { .. }
                | Instruction::IncrementRc { .. }
                | Instruction::MakeArray { .. } => {}
            }
        }

        trace!("Number tainted Brillig calls: {}", self.tainted.len());
    }

    /// Every Brillig call not properly constrained should remain in the tainted set
    /// at this point. For each, emit a corresponding warning.
    fn collect_warnings(&mut self, function: &Function) -> Vec<SsaReport> {
        let warnings: Vec<SsaReport> = self
            .tainted
            .keys()
            .map(|brillig_call| {
                SsaReport::Bug(InternalBug::UncheckedBrilligCall {
                    call_stack: function.dfg.get_instruction_call_stack(*brillig_call),
                })
            })
            .collect();

        trace!(
            "making {} under constrained reports for function {}",
            warnings.len(),
            function.name()
        );
        warnings
    }

    /// Update sets of value ids that can be traced back to the Brillig calls being tracked
    fn update_children(&mut self, parents: &[ValueId], children: &[ValueId]) {
        let parents: HashSet<_> = HashSet::from_iter(parents.iter().copied());
        for (_, tainted_ids) in self.tainted.iter_mut() {
            tainted_ids.update_children(&parents, children);
        }
    }

    /// Check if any of the recorded Brillig calls have been properly constrained
    /// by given values after recording partial constraints, if so stop tracking them
    fn clear_constrained(&mut self, constrained_values: &[ValueId], function: &Function) {
        // Remove numeric constants
        let constrained_values =
            constrained_values.iter().filter(|v| function.dfg.get_numeric_constant(**v).is_none());

        // For now, consider array element constraints to be array constraints
        // TODO(https://github.com/noir-lang/noir/issues/6698):
        // This probably has to be further looked into, to ensure _every_ element
        // of an array result of a Brillig call has been constrained
        let constrained_values: HashSet<_> = constrained_values
            .map(|v| {
                if let Some(parent_array) = self.array_elements.get(v) {
                    *parent_array
                } else {
                    *v
                }
            })
            .collect();

        self.tainted.iter_mut().for_each(|(_, tainted_ids)| {
            tainted_ids.store_partial_constraints(&constrained_values);
        });
        self.tainted.retain(|_, tainted_ids| !tainted_ids.check_constrained());
    }
}

#[derive(Default)]
struct Context {
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
    value_sets: Vec<HashSet<ValueId>>,
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
    ) -> HashSet<usize> {
        let variable_parameters_and_return_values = function
            .parameters()
            .iter()
            .chain(function.returns())
            .filter(|id| function.dfg.get_numeric_constant(**id).is_none())
            .map(|value_id| function.dfg.resolve(*value_id));

        let mut connected_sets_indices: HashSet<usize> = HashSet::new();

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
        current_set: &HashSet<ValueId>,
        all_brillig_generated_values: &HashSet<ValueId>,
        function: &Function,
    ) -> Vec<SsaReport> {
        let mut warnings = Vec::new();
        // Find brillig-generated values in the set
        let intersection = all_brillig_generated_values.intersection(current_set).copied();

        // Go through all Brillig outputs in the set
        for brillig_output_in_set in intersection {
            // Get the inputs that correspond to the output
            let inputs: HashSet<ValueId> =
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
            let mut instruction_arguments_and_results = HashSet::new();

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
                            | Intrinsic::AsField
                            | Intrinsic::AsSlice
                            | Intrinsic::BlackBox(..)
                            | Intrinsic::Hint(Hint::BlackBox)
                            | Intrinsic::DerivePedersenGenerators
                            | Intrinsic::FromField
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
                        Value::Function(callee) => match all_functions[&callee].runtime() {
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
                        | Value::Param { .. } => {
                            panic!("At the point we are running disconnect there shouldn't be any other values as arguments")
                        }
                    }
                }
                Instruction::Allocate { .. }
                | Instruction::DecrementRc { .. }
                | Instruction::EnableSideEffectsIf { .. }
                | Instruction::IncrementRc { .. }
                | Instruction::RangeCheck { .. } => {}
            }
        }

        self.block_queue.extend(function.dfg[block].successors());
    }

    /// Merge all small sets into larger ones based on whether the sets intersect or not
    ///
    /// If two small sets have a common ValueId, we merge them into one
    fn merge_sets(current: &[HashSet<ValueId>]) -> Vec<HashSet<ValueId>> {
        let mut new_set_id: usize = 0;
        let mut updated_sets: HashMap<usize, HashSet<ValueId>> = HashMap::new();
        let mut value_dictionary: HashMap<ValueId, usize> = HashMap::new();
        let mut parsed_value_set: HashSet<ValueId> = HashSet::new();

        for set in current.iter() {
            // Check if the set has any of the ValueIds we've encountered at previous iterations
            let intersection: HashSet<ValueId> =
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
            let mut joining_sets_ids: HashSet<usize> =
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
                updated_sets.extract(&largest_set_index).expect("Set should be in the hashmap").0;

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
    fn merge_sets_par(sets: &[HashSet<ValueId>]) -> Vec<HashSet<ValueId>> {
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
}
