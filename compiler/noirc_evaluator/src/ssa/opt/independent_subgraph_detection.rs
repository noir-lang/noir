//! This module defines an SSA pass that detects if the final function has any subgraphs independent from inputs and outputs.
//! If this is the case, then part of the final circuit can be completely replaced by any other passing circuit, since there are no constraints ensuring connections.
//! So the compiler informs the developer of this as a bug
use im::HashMap;

use crate::errors::{InternalBug, SsaReport};
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::function::RuntimeType;
use crate::ssa::ir::function::{Function, FunctionId};
use crate::ssa::ir::instruction::{Instruction, InstructionId, Intrinsic};
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::ssa_gen::Ssa;
use std::collections::{BTreeMap, HashSet};

impl Ssa {
    /// Go through each top-level non-brillig function and detect if it has independent subgraphs
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn detect_independent_subgraphs(mut self) -> Ssa {
        for function in self.functions.values() {
            match function.runtime() {
                RuntimeType::Acir { .. } => {
                    let warnings = self.detect_independent_subgraphs_within_function(function);
                    self.warnings.extend(warnings);
                }
                RuntimeType::Brillig => (),
            }
        }
        self
    }

    /// Detect independent subgraphs (not connected to function inputs or outputs) and return a vector of bug reports if some are found
    fn detect_independent_subgraphs_within_function(&self, function: &Function) -> Vec<SsaReport> {
        let mut context = Context::default();

        // Go through each block in the function and create a list of sets of ValueIds connected by instructions
        context.block_queue.push(function.entry_block());
        while let Some(block) = context.block_queue.pop() {
            if context.visited_blocks.contains(&block) {
                continue;
            }
            context.visited_blocks.insert(block);
            context.connect_value_ids_in_block(function, block, &self.functions);
        }

        // Merge ValueIds into sets, where each original small set of ValueIds is merged with another set if they intersect
        context.merge_sets();

        let function_parameters = function.parameters();
        let variable_parameters_and_return_values: HashSet<ValueId> = function_parameters
            .iter()
            .chain(function.returns())
            .filter(|&x| match function.dfg[*x] {
                Value::NumericConstant { .. } => false, // Constant values don't connect elements and can be reused in different subgraphs, so we need to avoid them
                _ => true,
            })
            .copied()
            .collect();

        let mut connected_sets_indices: HashSet<usize> = HashSet::new();

        // Go through each parameter and each set and check if the set contains the parameter
        // If it's the case, then that set doesn't present an issue
        for parameter_or_return_value in variable_parameters_and_return_values.iter() {
            for (set_index, final_set) in context.value_sets.iter().enumerate() {
                if final_set.contains(parameter_or_return_value) {
                    connected_sets_indices.insert(set_index);
                }
            }
        }

        // All the other sets of variables are independent
        let disconnected_sets_indices: Vec<usize> =
            HashSet::from_iter(0..(context.value_sets.len()))
                .difference(&connected_sets_indices)
                .copied()
                .collect();

        let all_brillig_generated_values: HashSet<ValueId> =
            context.brillig_return_to_argument_map.keys().copied().collect();

        // Go through each disconnected set
        for set_index in disconnected_sets_indices.iter() {
            let current_set = &context.value_sets[*set_index];

            // Find brillig-generated values in the set
            let intersection: Vec<ValueId> =
                all_brillig_generated_values.intersection(current_set).cloned().collect();
            if intersection.is_empty() {
                // This is probably a test and the values are optimized in a weird way
                continue;
            }

            // Go through all brillig outputs in the set
            for brillig_output_in_set in intersection.iter() {
                // Get the inputs that correspond to the output
                let inputs: HashSet<ValueId> = HashSet::from_iter(
                    context.brillig_return_to_argument_map[brillig_output_in_set].iter().copied(),
                );

                // Check if any of them are not in the set
                let unused_inputs: Vec<ValueId> = inputs.difference(current_set).copied().collect();

                // There is a value not in the set, which means that the inputs/outputs of this call have not been properly constrained
                if !unused_inputs.is_empty() {
                    context.warnings.push(SsaReport::Bug(InternalBug::IndependentSubgraph {
                        call_stack: function.dfg.get_call_stack(
                            context.brillig_return_to_instruction_id_map[brillig_output_in_set],
                        ),
                    }));
                }
            }
        }
        context.warnings
    }
}

#[derive(Default)]
struct Context {
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
    warnings: Vec<SsaReport>,
    value_sets: Vec<HashSet<ValueId>>,
    brillig_return_to_argument_map: HashMap<ValueId, Vec<ValueId>>,
    brillig_return_to_instruction_id_map: HashMap<ValueId, InstructionId>,
}

impl Context {
    /// Go through each instruction in the block and add a set of ValueIds connected through that instruction
    fn connect_value_ids_in_block(
        &mut self,
        function: &Function,
        block: BasicBlockId,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) {
        let instructions = function.dfg[block].instructions();

        for instruction in instructions.iter() {
            let results = function.dfg.instruction_results(*instruction);
            // For most instructions we just connect inputs and outputs
            match &function.dfg[*instruction] {
                Instruction::Binary(binary) => {
                    let mut value_ids = vec![binary.lhs, binary.rhs];
                    value_ids.extend_from_slice(results);
                    self.connect_values(function, &value_ids);
                }
                Instruction::Cast(value_id, ..)
                | Instruction::Truncate { value: value_id, .. }
                | Instruction::Not(value_id) => {
                    let mut value_ids = vec![*value_id];
                    value_ids.extend_from_slice(results);
                    self.connect_values(function, &value_ids);
                }
                Instruction::Constrain(value_id1, value_id2, ..) => {
                    let value_ids = &[*value_id1, *value_id2];
                    self.connect_values(function, value_ids);
                }
                Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                    let mut value_ids =
                        vec![*then_condition, *then_value, *else_condition, *else_value];

                    value_ids.extend_from_slice(results);
                    self.connect_values(function, &value_ids);
                }
                Instruction::Load { address } => {
                    let mut value_ids = vec![*address];
                    value_ids.extend_from_slice(results);
                    self.connect_values(function, &value_ids);
                }
                Instruction::Store { address, value } => {
                    self.connect_values(function, &[*address, *value])
                }
                Instruction::ArrayGet { array, index } => {
                    let mut value_ids = vec![*array, *index];
                    value_ids.extend_from_slice(results);
                    self.connect_values(function, &value_ids);
                }
                Instruction::ArraySet { array, index, value, .. } => {
                    self.connect_values(function, &[*array, *index, *value])
                }

                Instruction::Call { func: func_id, arguments: argument_ids } => {
                    match &function.dfg[*func_id] {
                        Value::Intrinsic(intrinsic) => match intrinsic {
                            Intrinsic::IsUnconstrained
                            | Intrinsic::AsWitness
                            | Intrinsic::ApplyRangeConstraint
                            | Intrinsic::AssertConstant => {}
                            _ => {
                                let mut value_ids = argument_ids.clone();
                                value_ids.extend_from_slice(results);
                                self.connect_values(function, &value_ids);
                            }
                        },
                        Value::Function(callee) => match all_functions[&callee].runtime() {
                            RuntimeType::Brillig => {
                                // For calls to brillig functions we memorize the mapping of results to argument ValueId's and InstructionId's
                                // The latter are needed to produce the callstack later
                                for result in results.iter() {
                                    self.brillig_return_to_argument_map
                                        .insert(*result, argument_ids.clone());
                                    self.brillig_return_to_instruction_id_map
                                        .insert(*result, instruction.clone());
                                }
                            }
                            _ => {
                                let mut value_ids = argument_ids.clone();
                                value_ids.extend_from_slice(results);
                                self.connect_values(function, &value_ids);
                            }
                        },
                        Value::ForeignFunction(..) => {
                            panic!("Should not be able to reach foreign function from non-brillig functions");
                        }
                        _ => {
                            panic!("At the point we are running disconnect there shouldn't be any other values as arguments")
                        }
                    }
                }
                _ => {}
            }
        }

        self.block_queue.extend(function.dfg[block].successors());
    }

    /// Add a set of ValueIds to the vector of connected values while ignoring constants
    fn connect_values(&mut self, function: &Function, values: &[ValueId]) {
        self.value_sets.push(HashSet::from_iter(
            values
                .iter()
                .filter(|value_id| match function.dfg.get_value(**value_id) {
                    Value::NumericConstant { .. } => false,
                    _ => true,
                })
                .cloned(),
        ));
    }

    /// Merge all small sets into larger ones based on whether the sets intersect or not
    ///
    /// If two small sets have a common ValueId, we merge them into one
    fn merge_sets(&mut self) {
        let mut new_set_id: usize = 0;
        let mut updated_sets: HashMap<usize, HashSet<ValueId>> = HashMap::new();
        let mut value_dictionary: HashMap<ValueId, usize> = HashMap::new();
        let mut parsed_value_set: HashSet<ValueId> = HashSet::new();

        // Go through each set
        for set in self.value_sets.iter() {
            // Check if the set has any of the ValueIds we've encountered at previous iterations
            let intersection: HashSet<ValueId> =
                set.intersection(&parsed_value_set).cloned().collect();
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
        self.value_sets = updated_sets.values().cloned().collect();
    }
}
#[cfg(test)]
mod test {
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
        ssa = ssa.detect_independent_subgraphs();
        assert_eq!(ssa.warnings.len(), 0);
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

        builder.new_brillig_function("br".into(), br_function_id);
        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.add_parameter(Type::field());
        let v2 = builder.insert_binary(v0, BinaryOp::Add, v1);
        builder.terminate_with_return(vec![v2]);
        let mut ssa = builder.finish();
        ssa = ssa.detect_independent_subgraphs();
        assert_eq!(ssa.warnings.len(), 1);
    }
}
