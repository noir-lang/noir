use super::instruction::{Argument, Instruction};
use super::options::SsaBlockOptions;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, InstructionWithOneArg, InstructionWithTwoArgs},
    typed_value::{TypedValue, ValueType},
};
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use std::collections::{HashMap, VecDeque};

/// Main context for the ssa block containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
#[derive(Debug, Clone)]
pub(crate) struct BlockContext {
    /// Ids of the Program variables stored as TypedValue separated by type
    pub(crate) stored_values: HashMap<ValueType, Vec<TypedValue>>,
    /// Ids of typed addresses of memory (mutable variables)
    pub(crate) memory_addresses: HashMap<ValueType, Vec<TypedValue>>,
    /// ACIR and Brillig last changed value, used to finalize the block with return
    pub(crate) last_value: Option<TypedValue>,
    /// Parent blocks history
    pub(crate) parent_blocks_history: VecDeque<BasicBlockId>,
    /// Children blocks
    pub(crate) children_blocks: Vec<BasicBlockId>,
    /// Options for the block
    pub(crate) options: SsaBlockOptions,
}

/// Returns a typed value from the map
/// Variables are stored in a map with type as key and vector of typed values as value
/// We use modulo to wrap index around the length of the vector, because fuzzer can produce index that is greater than the length of the vector
fn get_typed_value_from_map(
    map: &HashMap<ValueType, Vec<TypedValue>>,
    type_: &ValueType,
    idx: usize,
) -> Option<TypedValue> {
    let arr = map.get(type_);
    arr?;
    let arr = arr.unwrap();
    let value = arr.get(idx % arr.len());
    value?;
    Some(value.unwrap().clone())
}

fn append_typed_value_to_map(
    map: &mut HashMap<ValueType, Vec<TypedValue>>,
    type_: &ValueType,
    value: TypedValue,
) {
    map.entry(*type_).or_default().push(value);
}

impl BlockContext {
    pub(crate) fn new(
        stored_values: HashMap<ValueType, Vec<TypedValue>>,
        memory_addresses: HashMap<ValueType, Vec<TypedValue>>,
        parent_blocks_history: VecDeque<BasicBlockId>,
        options: SsaBlockOptions,
    ) -> Self {
        Self {
            stored_values,
            memory_addresses,
            last_value: None,
            parent_blocks_history,
            children_blocks: Vec::new(),
            options,
        }
    }

    /// Inserts an instruction that takes a single argument
    fn insert_instruction_with_single_arg(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        arg: Argument,
        instruction: InstructionWithOneArg,
    ) {
        let value = get_typed_value_from_map(&self.stored_values, &arg.value_type, arg.index);
        let value = match value {
            Some(value) => value,
            _ => return,
        };
        let acir_result = instruction(acir_builder, value.clone());
        // insert to brillig, assert id is the same
        assert_eq!(acir_result.value_id, instruction(brillig_builder, value).value_id);
        self.last_value = Some(acir_result.clone());
        append_typed_value_to_map(
            &mut self.stored_values,
            &acir_result.to_value_type(),
            acir_result,
        );
    }

    /// Inserts an instruction that takes two arguments
    fn insert_instruction_with_double_args(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        lhs: Argument,
        rhs: Argument,
        instruction: InstructionWithTwoArgs,
    ) {
        let instr_lhs = get_typed_value_from_map(&self.stored_values, &lhs.value_type, lhs.index);
        // We ignore type of the second argument, because all binary instructions must use the same type
        let instr_rhs = get_typed_value_from_map(&self.stored_values, &lhs.value_type, rhs.index);
        let (instr_lhs, instr_rhs) = match (instr_lhs, instr_rhs) {
            (Some(acir_lhs), Some(acir_rhs)) => (acir_lhs, acir_rhs),
            _ => return,
        };
        let result = instruction(acir_builder, instr_lhs.clone(), instr_rhs.clone());
        // insert to brillig, assert id of return is the same
        assert_eq!(result.value_id, instruction(brillig_builder, instr_lhs, instr_rhs).value_id);

        self.last_value = Some(result.clone());
        append_typed_value_to_map(&mut self.stored_values, &result.to_value_type(), result);
    }

    /// Inserts an instruction into both ACIR and Brillig programs
    fn insert_instruction(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        instruction: Instruction,
    ) {
        match instruction {
            Instruction::AddChecked { lhs, rhs } => {
                if !self.options.instruction_options.add_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_add_instruction_checked(lhs, rhs),
                );
            }
            Instruction::SubChecked { lhs, rhs } => {
                if !self.options.instruction_options.sub_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_sub_instruction_checked(lhs, rhs),
                );
            }
            Instruction::MulChecked { lhs, rhs } => {
                if !self.options.instruction_options.mul_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_mul_instruction_checked(lhs, rhs),
                );
            }
            Instruction::Div { lhs, rhs } => {
                if !self.options.instruction_options.div_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_div_instruction(lhs, rhs),
                );
            }
            Instruction::Eq { lhs, rhs } => {
                if !self.options.instruction_options.eq_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_eq_instruction(lhs, rhs),
                );
            }
            Instruction::Cast { lhs, type_ } => {
                if !self.options.instruction_options.cast_enabled {
                    return;
                }
                let value =
                    get_typed_value_from_map(&self.stored_values, &lhs.value_type, lhs.index);
                let value = match value {
                    Some(value) => value,
                    _ => return,
                };
                let acir_result = acir_builder.insert_cast(value.clone(), type_);
                assert_eq!(
                    acir_result.value_id,
                    brillig_builder.insert_cast(value.clone(), type_).value_id
                );
                // Cast can return the same value as the original value, if cast type is forbidden, so we skip it
                if self.stored_values.get(&value.to_value_type()).unwrap().contains(&acir_result) {
                    return;
                }
                self.last_value = Some(acir_result.clone());
                append_typed_value_to_map(
                    &mut self.stored_values,
                    &acir_result.to_value_type(),
                    acir_result,
                );
            }
            Instruction::Mod { lhs, rhs } => {
                if !self.options.instruction_options.mod_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_mod_instruction(lhs, rhs),
                );
            }
            Instruction::Not { lhs } => {
                if !self.options.instruction_options.not_enabled {
                    return;
                }
                self.insert_instruction_with_single_arg(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    |builder, lhs| builder.insert_not_instruction(lhs),
                );
            }
            Instruction::Shl { lhs, rhs } => {
                if !self.options.instruction_options.shl_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_shl_instruction(lhs, rhs),
                );
            }
            Instruction::Shr { lhs, rhs } => {
                if !self.options.instruction_options.shr_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_shr_instruction(lhs, rhs),
                );
            }
            Instruction::And { lhs, rhs } => {
                if !self.options.instruction_options.and_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_and_instruction(lhs, rhs),
                );
            }
            Instruction::Or { lhs, rhs } => {
                if !self.options.instruction_options.or_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_or_instruction(lhs, rhs),
                );
            }
            Instruction::Xor { lhs, rhs } => {
                if !self.options.instruction_options.xor_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_xor_instruction(lhs, rhs),
                );
            }
            Instruction::Lt { lhs, rhs } => {
                if !self.options.instruction_options.lt_enabled {
                    return;
                }
                // TODO: prevent in builder
                if lhs.value_type == ValueType::Field {
                    return;
                }
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_lt_instruction(lhs, rhs),
                );
            }

            Instruction::AddSubConstrain { lhs, rhs } => {
                // inserts lhs' = lhs + rhs
                let lhs_orig =
                    get_typed_value_from_map(&self.stored_values, &ValueType::Field, lhs);
                let rhs = get_typed_value_from_map(&self.stored_values, &ValueType::Field, rhs);
                let (lhs_orig, rhs) = match (lhs_orig, rhs) {
                    (Some(lhs_orig), Some(rhs)) => (lhs_orig, rhs),
                    _ => return,
                };
                // assert ids of add are the same for both builders
                let lhs_add_rhs =
                    acir_builder.insert_add_instruction_checked(lhs_orig.clone(), rhs.clone());
                assert_eq!(
                    lhs_add_rhs.value_id,
                    brillig_builder
                        .insert_add_instruction_checked(lhs_orig.clone(), rhs.clone())
                        .value_id,
                );
                // inserts lhs'' = lhs' - rhs
                let lhs = lhs_add_rhs;
                let morphed = acir_builder.insert_sub_instruction_checked(lhs.clone(), rhs.clone());

                // assert ids of sub are the same for both builders
                assert_eq!(
                    morphed.value_id,
                    brillig_builder
                        .insert_sub_instruction_checked(lhs.clone(), rhs.clone())
                        .value_id,
                );

                if !self.options.constrain_idempotent_enabled {
                    return;
                }

                acir_builder.insert_constrain(lhs_orig.clone(), morphed.clone());
                brillig_builder.insert_constrain(lhs_orig.clone(), morphed.clone());
            }
            Instruction::MulDivConstrain { lhs, rhs } => {
                let lhs_orig =
                    get_typed_value_from_map(&self.stored_values, &ValueType::Field, lhs);
                let rhs = get_typed_value_from_map(&self.stored_values, &ValueType::Field, rhs);
                let (lhs_orig, rhs) = match (lhs_orig, rhs) {
                    (Some(lhs_orig), Some(rhs)) => (lhs_orig, rhs),
                    _ => return,
                };
                // inserts lhs' = lhs * rhs
                // assert ids of mul are the same for both builders
                let lhs_mul_rhs =
                    acir_builder.insert_mul_instruction_checked(lhs_orig.clone(), rhs.clone());
                assert_eq!(
                    lhs_mul_rhs.value_id,
                    brillig_builder
                        .insert_mul_instruction_checked(lhs_orig.clone(), rhs.clone())
                        .value_id,
                );
                // lhs'' = lhs' / rhs
                let lhs = lhs_mul_rhs;
                // insert to both builders, assert ids of div are the same
                let morphed = acir_builder.insert_div_instruction(lhs.clone(), rhs.clone());
                assert_eq!(
                    morphed.value_id,
                    brillig_builder.insert_div_instruction(lhs.clone(), rhs.clone()).value_id,
                );

                if !self.options.constrain_idempotent_enabled {
                    return;
                }
                acir_builder.insert_constrain(lhs_orig.clone(), morphed.clone());
                brillig_builder.insert_constrain(lhs_orig.clone(), morphed.clone());
            }
            Instruction::AddToMemory { lhs } => {
                if !self.options.instruction_options.alloc_enabled {
                    return;
                }

                let value =
                    match get_typed_value_from_map(&self.stored_values, &lhs.value_type, lhs.index)
                    {
                        Some(value) => value,
                        _ => return,
                    };

                let addr = acir_builder.insert_add_to_memory(value.clone());
                assert_eq!(
                    addr.clone().value_id,
                    brillig_builder.insert_add_to_memory(value.clone()).value_id,
                    "add to memory differs in ACIR and Brillig"
                );
                // Append the memory address to stored_values with the type of the result
                append_typed_value_to_map(&mut self.memory_addresses, &addr.to_value_type(), addr);
            }
            Instruction::LoadFromMemory { memory_addr } => {
                if !self.options.instruction_options.load_enabled {
                    return;
                }
                let addr = get_typed_value_from_map(
                    &self.memory_addresses,
                    &memory_addr.value_type,
                    memory_addr.index,
                );
                let addr = match addr {
                    Some(addr) => addr,
                    _ => return,
                };
                let value = acir_builder.insert_load_from_memory(addr.clone());
                assert_eq!(
                    value.value_id,
                    brillig_builder.insert_load_from_memory(addr.clone()).value_id,
                    "load from memory differs in ACIR and Brillig"
                );
                append_typed_value_to_map(
                    &mut self.stored_values,
                    &value.to_value_type(),
                    value.clone(),
                );
                self.last_value = Some(value.clone());
            }
            Instruction::SetToMemory { memory_addr_index, value } => {
                if !self.options.instruction_options.store_enabled {
                    return;
                }
                let addr = get_typed_value_from_map(
                    &self.memory_addresses,
                    &value.value_type,
                    memory_addr_index,
                );
                let addr = match addr {
                    Some(addr) => addr,
                    _ => return,
                };
                let value =
                    get_typed_value_from_map(&self.stored_values, &value.value_type, value.index);
                let value = match value {
                    Some(value) => value,
                    _ => return,
                };
                acir_builder.insert_set_to_memory(addr.clone(), value.clone());
                brillig_builder.insert_set_to_memory(addr.clone(), value);
            }
        }
    }

    pub(crate) fn insert_instructions(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        instructions: &Vec<Instruction>,
    ) {
        for instruction in instructions {
            self.insert_instruction(acir_builder, brillig_builder, *instruction);
        }
    }

    /// Finalizes the function by setting the return value
    pub(crate) fn finalize_block_with_return(
        self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
    ) {
        match self.last_value {
            Some(last_value) => {
                acir_builder.finalize_function(&last_value);
                brillig_builder.finalize_function(&last_value);
            }
            _ => {
                // If no last value was set, we return boolean, that definitely  set
                let last_value =
                    get_typed_value_from_map(&self.stored_values, &ValueType::Boolean, 0).unwrap();
                acir_builder.finalize_function(&last_value);
                brillig_builder.finalize_function(&last_value);
            }
        }
    }

    pub(crate) fn finalize_block_with_jmp(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        jmp_destination: BasicBlockId,
    ) {
        acir_builder.insert_jmp_instruction(jmp_destination, vec![]);
        brillig_builder.insert_jmp_instruction(jmp_destination, vec![]);
        self.children_blocks.push(jmp_destination);
    }

    pub(crate) fn finalize_block_with_jmp_if(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
    ) {
        // takes last boolean variable as condition
        let condition = self
            .stored_values
            .get(&ValueType::Boolean)
            .and_then(|values| values.last().cloned())
            .expect("Should have at least one boolean")
            .value_id;

        acir_builder.insert_jmpif_instruction(condition, then_destination, else_destination);
        brillig_builder.insert_jmpif_instruction(condition, then_destination, else_destination);
        self.children_blocks.push(then_destination);
        self.children_blocks.push(else_destination);
    }
}
