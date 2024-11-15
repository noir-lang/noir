//! This module defines an SSA pass that detects if the final function has any subgraphs independent from inputs and outputs.
//! If this is the case, then part of the final circuit can be completely replaced by any other passing circuit, since there are no constraints ensuring connections.
//! So the compiler informs the developer of this as a bug
use crate::errors::{InternalBug, SsaReport};
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::function::RuntimeType;
use crate::ssa::ir::function::{Function, FunctionId};
use crate::ssa::ir::instruction::{Instruction, InstructionId, Intrinsic};
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::ssa_gen::Ssa;
use im::HashMap;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashSet};
use tracing::{debug, trace};

impl Ssa {
    /// Go through each top-level non-brillig function and detect if it has independent subgraphs
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn check_for_underconstrained_values(&mut self) -> Vec<SsaReport> {
        let functions_id = self.functions.values().map(|f| f.id().to_usize()).collect::<Vec<_>>();
        functions_id
            .iter()
            .par_bridge()
            .flat_map(|fid| {
                let function_to_process = &self.functions[&FunctionId::new(*fid)];
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

    /// Detect brillig calls left unconstrained with later manual asserts
    /// and return a vector of bug reports if some are found
    pub(crate) fn check_for_missing_brillig_constrains(&mut self) -> Vec<SsaReport> {
        let functions_id = self.functions.values().map(|f| f.id().to_usize()).collect::<Vec<_>>();
        functions_id
            .iter()
            .par_bridge()
            .flat_map(|fid| {
                let function_to_process = &self.functions[&FunctionId::new(*fid)];
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
    value_parents: HashMap<ValueId, Vec<ValueId>>,
    // Map keeping track of values stored at memory locations
    memory_slots: HashMap<ValueId, ValueId>,
    // List of values involved in constrain instructions
    constrained_values: Vec<Vec<ValueId>>,
    // Map of brillig call ids to sets of their arguments and results
    brillig_values: HashMap<InstructionId, (HashSet<ValueId>, HashSet<ValueId>)>,
}

impl DependencyContext {
    /// Build the dependency graph of variable ValueIds, also storing
    /// information on value ids involved in constrain operations
    /// and brillig calls
    fn build(&mut self, function: &Function, all_functions: &BTreeMap<FunctionId, Function>) {
        self.block_queue.push(function.entry_block());
        while let Some(block) = self.block_queue.pop() {
            if self.visited_blocks.contains(&block) {
                continue;
            }
            self.visited_blocks.insert(block);
            self.build_block_value_flow_graph(block, function, all_functions);
        }
    }

    fn build_block_value_flow_graph(
        &mut self,
        block: BasicBlockId,
        function: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) {
        trace!("building block value flow graph for block {} of function {}", block, function);

        for instruction in function.dfg[block].instructions() {
            let mut arguments = Vec::new();
            let mut results = Vec::new();

            // Collect non-constant instruction arguments
            function.dfg[*instruction].for_each_value(|value_id| {
                if function.dfg.get_numeric_constant(value_id).is_none() {
                    arguments.push(function.dfg.resolve(value_id));
                }
            });

            // Assign parent arguments to non-constant results
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
                    self.memory_slots.insert(*address, *value);
                }
                Instruction::Load { address } => {
                    // Remember the value stored at address as parent for the results
                    if let Some(value_id) = self.memory_slots.get(address) {
                        for result in results {
                            self.value_parents.entry(result).or_default().push(*value_id);
                        }
                    } else {
                        debug!("load instruction {} has attempted to access previously unused memory location, skipping",
                            instruction);
                    }
                }
                // Record the constrain instruction arguments to check them against those
                // involved in brillig calls
                Instruction::Constrain(value1, value2, _) => {
                    self.constrained_values.push(vec![*value1, *value2]);
                }
                // Consider range check to also be constraining
                Instruction::RangeCheck { value, .. } => {
                    self.constrained_values.push(vec![*value]);
                }
                Instruction::Call { func: func_id, arguments } => {
                    match &function.dfg[*func_id] {
                        Value::Intrinsic(intrinsic) => match intrinsic {
                            Intrinsic::ApplyRangeConstraint | Intrinsic::AssertConstant => {
                                // Consider these intrinsic arguments constrained
                                self.constrained_values.push(arguments.clone());
                            }
                            Intrinsic::AsWitness | Intrinsic::IsUnconstrained => {
                                // These intrinsics won't affect the dependency graph
                            }
                            Intrinsic::ArrayLen
                            | Intrinsic::ArrayAsStrUnchecked
                            | Intrinsic::AsField
                            | Intrinsic::AsSlice
                            | Intrinsic::BlackBox(..)
                            | Intrinsic::DerivePedersenGenerators
                            | Intrinsic::FromField
                            | Intrinsic::SlicePushBack
                            | Intrinsic::SlicePushFront
                            | Intrinsic::SlicePopBack
                            | Intrinsic::SlicePopFront
                            | Intrinsic::SliceInsert
                            | Intrinsic::SliceRemove
                            | Intrinsic::StaticAssert
                            | Intrinsic::StrAsBytes
                            | Intrinsic::ToBits(..)
                            | Intrinsic::ToRadix(..)
                            | Intrinsic::FieldLessThan => {
                                // Record all the function arguments as parents of the results
                                for result in results {
                                    self.value_parents.entry(result).or_default().extend(arguments);
                                }
                            }
                        },
                        Value::Function(callee) => match all_functions[&callee].runtime() {
                            RuntimeType::Brillig(_) => {
                                // Record arguments/results for each brillig call for the check
                                trace!("brillig function {} called at {}", callee, instruction);
                                self.brillig_values.insert(
                                    *instruction,
                                    (
                                        HashSet::from_iter(arguments.clone()),
                                        HashSet::from_iter(results),
                                    ),
                                );
                            }
                            RuntimeType::Acir(..) => {
                                // Record all the function arguments as parents of the results
                                for result in results {
                                    self.value_parents.entry(result).or_default().extend(arguments);
                                }
                            }
                        },
                        Value::ForeignFunction(..) => {
                            debug!("should not be able to reach foreign function from non-brillig functions, {func_id} in function {}", function.name());
                        }
                        Value::Array { .. }
                        | Value::Instruction { .. }
                        | Value::NumericConstant { .. }
                        | Value::Param { .. } => {
                            debug!(
                                "should not be able to call {func_id} in function {}",
                                function.name()
                            );
                        }
                    }
                }
                Instruction::ArrayGet { .. }
                | Instruction::ArraySet { .. }
                | Instruction::Binary(..)
                | Instruction::Cast(..)
                | Instruction::IfElse { .. }
                | Instruction::Not(..)
                | Instruction::Truncate { .. } => {
                    // Record all the used arguments as parents of the results
                    for result in results {
                        self.value_parents.entry(result).or_default().extend(&arguments);
                    }
                }
                // These instructions won't affect the dependency graph
                Instruction::Allocate { .. }
                | Instruction::DecrementRc { .. }
                | Instruction::EnableSideEffectsIf { .. }
                | Instruction::IncrementRc { .. } => {}
            }
        }

        trace!("resulting value parents map: {:?}", self.value_parents);
        trace!("resulting constrained values: {:?}", self.constrained_values);
        trace!("resulting brillig involved values: {:?}", self.brillig_values);
    }

    /// Check if the constrained values can be traced back to brillig calls.
    /// For every brillig call not properly constrained, emit a corresponding warning.
    fn collect_warnings(&mut self, function: &Function) -> Vec<SsaReport> {
        // the check is unneeded if there are no brillig calls
        if self.brillig_values.is_empty() {
            return vec![];
        }

        let mut covered_brillig_calls: HashSet<InstructionId> = HashSet::new();

        // reverse order of constrains to check the ones with fewest ancestors first
        for constrained_values in self.constrained_values.iter().rev() {
            let constrain_ancestors: HashSet<_> =
                constrained_values.iter().flat_map(|v| self.collect_ancestors(*v)).collect();
            trace!("checking constrain involving values {:?}", constrain_ancestors);
            for (brillig_call, brillig_values) in &self.brillig_values {
                if covered_brillig_calls.contains(brillig_call) {
                    continue;
                }
                // If there is at least one value among the brillig call arguments
                // along with all the results featuring in the constrain value ancestors,
                // consider the call properly covered
                if constrain_ancestors.intersection(&brillig_values.0).next().is_some()
                    && constrain_ancestors.is_superset(&brillig_values.1)
                {
                    trace!(
                        "brillig call at {} covered by constrained values {:?}",
                        brillig_call,
                        constrained_values
                    );
                    covered_brillig_calls.insert(*brillig_call);
                }
            }

            // stop checking if all the brillig calls are already found covered
            if covered_brillig_calls.len() == self.brillig_values.len() {
                break;
            }
        }

        // For each unchecked brillig call, emit a warning
        let unchecked_calls =
            self.brillig_values.keys().filter(|v| !covered_brillig_calls.contains(v));

        let warnings: Vec<SsaReport> = unchecked_calls
            .map(|brillig_call| {
                SsaReport::Bug(InternalBug::UncheckedBrilligCall {
                    call_stack: function.dfg.get_call_stack(*brillig_call),
                })
            })
            .collect();

        trace!("making following reports for function {}: {:?}", function.name(), warnings);
        warnings
    }

    /// Build a set of all ValueIds the given ValueId descends from
    fn collect_ancestors(&self, value_id: ValueId) -> HashSet<ValueId> {
        let mut to_visit = vec![value_id];
        // loops are possible judging by testing on noir-contracts,
        // so don't revisit nodes
        let mut visited = HashSet::new();
        let mut ancestors = HashSet::from([value_id]);
        while let Some(value_id) = to_visit.pop() {
            visited.insert(value_id);
            if let Some(values) = self.value_parents.get(&value_id) {
                for value in values {
                    if !visited.contains(value) {
                        to_visit.push(*value);
                    }
                }
                ancestors.extend(values);
            }
        }

        ancestors
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
    /// Additionally, store information about brillig calls in the context
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

    /// Find which brillig calls separate this set from others and return bug warnings about them
    fn find_disconnecting_brillig_calls_with_results_in_set(
        &self,
        current_set: &HashSet<ValueId>,
        all_brillig_generated_values: &HashSet<ValueId>,
        function: &Function,
    ) -> Vec<SsaReport> {
        let mut warnings = Vec::new();
        // Find brillig-generated values in the set
        let intersection = all_brillig_generated_values.intersection(current_set).copied();

        // Go through all brillig outputs in the set
        for brillig_output_in_set in intersection {
            // Get the inputs that correspond to the output
            let inputs: HashSet<ValueId> =
                self.brillig_return_to_argument[&brillig_output_in_set].iter().copied().collect();

            // Check if any of them are not in the set
            let unused_inputs = inputs.difference(current_set).next().is_some();

            // There is a value not in the set, which means that the inputs/outputs of this call have not been properly constrained
            if unused_inputs {
                warnings.push(SsaReport::Bug(InternalBug::IndependentSubgraph {
                    call_stack: function.dfg.get_call_stack(
                        self.brillig_return_to_instruction_id[&brillig_output_in_set],
                    ),
                }));
            }
        }
        warnings
    }
    /// Go through each instruction in the block and add a set of ValueIds connected through that instruction
    ///
    /// Additionally, this function adds mappings of brillig return values to call arguments and instruction ids from calls to brillig functions in the block
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
                | Instruction::Truncate { .. } => {
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
                            | Intrinsic::AsField
                            | Intrinsic::AsSlice
                            | Intrinsic::BlackBox(..)
                            | Intrinsic::DerivePedersenGenerators
                            | Intrinsic::FromField
                            | Intrinsic::SlicePushBack
                            | Intrinsic::SlicePushFront
                            | Intrinsic::SlicePopBack
                            | Intrinsic::SlicePopFront
                            | Intrinsic::SliceInsert
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
                                // For calls to brillig functions we memorize the mapping of results to argument ValueId's and InstructionId's
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
                            panic!("Should not be able to reach foreign function from non-brillig functions, {func_id} in function {}", function.name());
                        }
                        Value::Array { .. }
                        | Value::Instruction { .. }
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
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            instruction::BinaryOp,
            map::Id,
            types::{NumericType, Type},
        },
    };
    use std::sync::Arc;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    /// Test that a connected function raises no warnings
    fn test_simple_connected_function() {
        // fn main {
        //   b0(v0: Field, v1: Field):
        //      v2 = add v0, 1
        //      v3 = mul v1, 2
        //      v4 = eq v2, v3
        //      return v2
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.add_parameter(Type::field());

        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);

        let v2 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v3 = builder.insert_binary(v1, BinaryOp::Mul, two);
        let _v4 = builder.insert_binary(v2, BinaryOp::Eq, v3);
        builder.terminate_with_return(vec![v2]);

        let mut ssa = builder.finish();
        let ssa_level_warnings = ssa.check_for_underconstrained_values();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where the results of a call to a brillig function are not connected to main function inputs or outputs
    /// This should be detected.
    fn test_simple_function_with_disconnected_part() {
        //  unconstrained fn br(v0: Field, v1: Field){
        //      v2 = add v0, v1
        //      return v2
        //  }
        //
        //  fn main {
        //   b0(v0: Field, v1: Field):
        //      v2 = add v0, 1
        //      v3 = mul v1, 2
        //      v4 = call br(v2, v3)
        //      v5 = add v4, 2
        //      return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.add_parameter(Type::field());

        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);

        let v2 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v3 = builder.insert_binary(v1, BinaryOp::Mul, two);

        let br_function_id = Id::test_new(1);
        let br_function = builder.import_function(br_function_id);
        let v4 = builder.insert_call(br_function, vec![v2, v3], vec![Type::field()])[0];
        let v5 = builder.insert_binary(v4, BinaryOp::Add, two);
        builder.insert_constrain(v5, one, None);
        builder.terminate_with_return(vec![]);

        builder.new_brillig_function("br".into(), br_function_id, InlineType::default());
        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.add_parameter(Type::field());
        let v2 = builder.insert_binary(v0, BinaryOp::Add, v1);
        builder.terminate_with_return(vec![v2]);
        let mut ssa = builder.finish();
        let ssa_level_warnings = ssa.check_for_underconstrained_values();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where a call to a brillig function is left unchecked with a later assert,
    /// by example of the program illustrating issue #5425 (simplified).
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

        fn main f0 {
          b0(v0: [u32; 2], v1: [u32; 2], v2: u32):
            inc_rc v0
            inc_rc v1
            v4 = call f1(v0)
            v6 = allocate
            store u1 0 at v6
            v7 = load v6
            v11 = array_get v0, index u32 0
            v12 = eq v11, v4
            v13 = or v7, v12
            store v13 at v6
            v14 = load v6
            v16 = array_get v0, index u32 1
            v17 = eq v16, v4
            v18 = or v14, v17
            store v18 at v6
            v19 = load v6
            constrain v19 == u1 1
            v22 = call f1(v1)
            v23 = add v4, v22
            v24 = eq v2, v23
            constrain v2 == v23
            dec_rc v0
            dec_rc v1
            return
        }
        */
        let type_u32 = Type::Numeric(NumericType::Unsigned { bit_size: 32 });
        let type_u1 = Type::Numeric(NumericType::Unsigned { bit_size: 1 });

        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let zero = builder.numeric_constant(0u32, type_u32.clone());
        let one = builder.numeric_constant(1u32, type_u32.clone());

        let bool_false = builder.numeric_constant(0u32, type_u1.clone());
        let bool_true = builder.numeric_constant(1u32, type_u1.clone());

        let v0 = builder.add_parameter(Type::Array(Arc::new(vec![type_u32.clone()]), 2));
        let v1 = builder.add_parameter(Type::Array(Arc::new(vec![type_u32.clone()]), 2));
        let v2 = builder.add_parameter(type_u32.clone());

        builder.insert_inc_rc(v0);
        builder.insert_inc_rc(v1);

        let br_function_id = Id::test_new(1);
        let br_function = builder.import_function(br_function_id);

        let v4 = builder.insert_call(br_function, vec![v0], vec![type_u32.clone()])[0];
        let v6 = builder.insert_allocate(type_u32.clone());

        builder.insert_store(v6, bool_false);
        let v7 = builder.insert_load(v6, type_u1.clone());
        let v11 = builder.insert_array_get(v0, zero, type_u32.clone());
        let v12 = builder.insert_binary(v11, BinaryOp::Eq, v4);
        let v13 = builder.insert_binary(v7, BinaryOp::Or, v12);

        builder.insert_store(v6, v13);
        let v14 = builder.insert_load(v6, type_u1.clone());
        let v16 = builder.insert_array_get(v0, one, type_u32.clone());
        let v17 = builder.insert_binary(v16, BinaryOp::Eq, v4);
        let v18 = builder.insert_binary(v14, BinaryOp::Or, v17);

        builder.insert_store(v6, v18);
        let v19 = builder.insert_load(v6, type_u1.clone());

        builder.insert_constrain(v19, bool_true, None);

        let v22 = builder.insert_call(br_function, vec![v1], vec![type_u32.clone()])[0];
        let v23 = builder.insert_binary(v4, BinaryOp::Add, v22);

        builder.insert_constrain(v2, v23, None);

        builder.insert_dec_rc(v0);
        builder.insert_dec_rc(v1);

        builder.terminate_with_return(vec![]);

        // We're faking the brillig function here, for simplicity's sake

        builder.new_brillig_function("maximum_price".into(), br_function_id, InlineType::default());
        let v0 = builder.add_parameter(Type::Array(Arc::new(vec![type_u32.clone()]), 2));
        let zero = builder.numeric_constant(0u32, type_u32.clone());

        let v1 = builder.insert_array_get(v0, zero, type_u32);
        builder.terminate_with_return(vec![v1]);

        let mut ssa = builder.finish();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constrains();
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where a call to a brillig function returning multiple result values
    /// is left unchecked with a later assert involving all the results
    fn test_unchecked_multiple_results_brillig() {
        let type_u32 = Type::Numeric(NumericType::Unsigned { bit_size: 32 });

        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.add_parameter(type_u32.clone());

        let br_function_id = Id::test_new(1);
        let br_function = builder.import_function(br_function_id);

        // First call is constrained properly, involving both results
        let call_results =
            builder.insert_call(br_function, vec![v0], vec![type_u32.clone(), type_u32.clone()]);
        let (v6, v7) = (call_results[0], call_results[1]);
        let v8 = builder.insert_binary(v6, BinaryOp::Mul, v7);
        builder.insert_constrain(v8, v0, None);

        // Second call is insufficiently constrained, involving only one of the results
        let call_results =
            builder.insert_call(br_function, vec![v0], vec![type_u32.clone(), type_u32.clone()]);
        let (v9, _) = (call_results[0], call_results[1]);
        let v11 = builder.insert_binary(v9, BinaryOp::Mul, v9);
        builder.insert_constrain(v11, v0, None);

        builder.terminate_with_return(vec![]);

        // We're faking the brillig function here, for simplicity's sake

        builder.new_brillig_function("factor".into(), br_function_id, InlineType::default());
        builder.add_parameter(type_u32.clone());
        let zero = builder.numeric_constant(0u32, type_u32.clone());

        builder.terminate_with_return(vec![zero, zero]);

        let mut ssa = builder.finish();

        let ssa_level_warnings = ssa.check_for_missing_brillig_constrains();
        assert_eq!(ssa_level_warnings.len(), 1);
    }
}
