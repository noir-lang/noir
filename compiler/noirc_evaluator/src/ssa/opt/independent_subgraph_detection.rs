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
    fn detect_independent_subgraphs_within_function(&self, function: &Function) -> Vec<SsaReport> {
        let mut context = Context::default();
        let function_parameters = function.parameters();
        let mut nonconstant_parameters_and_return_values: HashSet<ValueId> = function_parameters
            .iter()
            .filter(|&x| match function.dfg[*x] {
                Value::NumericConstant { .. } => false,
                _ => true,
            })
            .copied()
            .collect();
        nonconstant_parameters_and_return_values.extend(function.returns());
        context.block_queue.push(function.entry_block());
        while let Some(block) = context.block_queue.pop() {
            if context.visited_blocks.contains(&block) {
                continue;
            }
            context.visited_blocks.insert(block);
            context.connect_value_ids_in_block(function, block, &self.functions);
        }
        context.merge_sets();
        let mut connected_sets_indices: HashSet<usize> = HashSet::new();
        for parameter_or_return_value in nonconstant_parameters_and_return_values.iter() {
            for (set_index, final_set) in context.value_sets.iter().enumerate() {
                if final_set.contains(parameter_or_return_value) {
                    connected_sets_indices.insert(set_index);
                }
            }
        }
        let disconnected_sets_indices: Vec<usize> =
            HashSet::from_iter(0..(context.value_sets.len()))
                .difference(&connected_sets_indices)
                .copied()
                .collect();
        let all_brillig_generated_values: HashSet<ValueId> =
            context.brillig_return_to_argument_map.keys().copied().collect();
        for set_index in disconnected_sets_indices.iter() {
            let current_set = &context.value_sets[*set_index];
            let intersection: Vec<ValueId> =
                all_brillig_generated_values.intersection(current_set).cloned().collect();
            if intersection.len() == 0 {
                panic!("Somehow produced a disconnected subgraph without any brillig. How is that possible?")
            }
            for brillig_output_in_set in intersection.iter() {
                let inputs: HashSet<ValueId> = HashSet::from_iter(
                    context.brillig_return_to_argument_map[brillig_output_in_set].iter().copied(),
                );
                let unused_inputs: Vec<ValueId> = inputs.difference(current_set).copied().collect();
                if unused_inputs.len() != 0 {
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
    fn connect_value_ids_in_block(
        &mut self,
        function: &Function,
        block: BasicBlockId,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) {
        let instructions = function.dfg[block].instructions();

        for instruction in instructions.iter() {
            let results = function.dfg.instruction_results(*instruction);
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

    fn merge_sets(&mut self) {
        let mut new_set_id: usize = 0;
        let mut updated_sets: HashMap<usize, HashSet<ValueId>> = HashMap::new();
        let mut value_dictionary: HashMap<ValueId, usize> = HashMap::new();
        let mut parse_value_set: HashSet<ValueId> = HashSet::new();
        for set in self.value_sets.iter() {
            let intersection: HashSet<ValueId> =
                set.intersection(&parse_value_set).cloned().collect();
            parse_value_set.extend(set.iter());
            if intersection.is_empty() {
                updated_sets.insert(new_set_id, set.clone());
                for entry in set.iter() {
                    value_dictionary.insert(*entry, new_set_id);
                }
                new_set_id += 1;
                continue;
            }
            let mut joining_sets_ids: HashSet<usize> =
                intersection.iter().map(|x| value_dictionary[x]).collect();
            let mut largest_set_size = usize::MIN;
            let mut largest_set_index = usize::MAX;
            for set_id in joining_sets_ids.iter() {
                if updated_sets[set_id].len() > largest_set_size {
                    (largest_set_index, largest_set_size) = (*set_id, updated_sets[set_id].len());
                }
            }
            joining_sets_ids.remove(&largest_set_index);

            let mut largest_set =
                updated_sets.extract(&largest_set_index).expect("Set should be in the hashmap").0;
            for set_id in joining_sets_ids.iter() {
                for element in updated_sets[set_id].iter() {
                    value_dictionary[element] = largest_set_index;
                    largest_set.insert(*element);
                }
                updated_sets.remove(set_id);
            }
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
    fn test_simple_connected_function() {
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
    fn test_simple_function_with_disconnected_part() {
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
