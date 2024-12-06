//! This module defines an SSA pass that detects if the final function has any subgraphs independent from inputs and outputs.
//! If this is the case, then part of the final circuit can be completely replaced by any other passing circuit, since there are no constraints ensuring connections.
//! So the compiler informs the developer of this as a bug
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
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{instruction::BinaryOp, map::Id, types::Type},
    };

    #[test]
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
}
