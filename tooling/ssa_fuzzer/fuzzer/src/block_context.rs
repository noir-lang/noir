use crate::instruction::{Argument, Instruction};
use crate::options::SsaBlockOptions;
use acvm::acir::native_types::Witness;
use acvm::{FieldElement, acir};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError, InstructionWithOneArg, InstructionWithTwoArgs},
    config::NUMBER_OF_VARIABLES_INITIAL,
    typed_value::{TypedValue, ValueType},
};
use noirc_driver::CompiledProgram;
use noirc_evaluator::ssa::ir::{basic_block::BasicBlockId, types::Type};
use std::collections::{HashMap, VecDeque};

/// Main context for the ssa block containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
#[derive(Debug, Clone)]
pub(crate) struct BlockContext {
    /// Ids of the Program variables stored as TypedValue separated by type
    pub(crate) stored_values: HashMap<ValueType, Vec<TypedValue>>,
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
    log::debug!("value: {:?}", value);
    log::debug!("index_mod_len: {:?}, idx: {:?}, arr_len: {:?}", idx % arr.len(), idx, arr.len());
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
        parent_blocks_history: VecDeque<BasicBlockId>,
        options: SsaBlockOptions,
    ) -> Self {
        Self {
            stored_values,
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
        let instr_rhs = get_typed_value_from_map(&self.stored_values, &rhs.value_type, rhs.index);
        let (instr_lhs, instr_rhs) = match (instr_lhs, instr_rhs) {
            (Some(acir_lhs), Some(acir_rhs)) => (acir_lhs, acir_rhs),
            _ => return,
        };
        let result = instruction(acir_builder, instr_lhs.clone(), instr_rhs.clone());
        // insert to brillig, assert id of return is the same
        assert_eq!(result.value_id, instruction(brillig_builder, instr_lhs, instr_rhs).value_id);

        //
        if self.stored_values.get(&result.to_value_type()).unwrap().contains(&result) {
            return;
        }
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
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_add_instruction_checked(lhs, rhs),
                );
            }
            Instruction::SubChecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_sub_instruction_checked(lhs, rhs),
                );
            }
            Instruction::MulChecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_mul_instruction_checked(lhs, rhs),
                );
            }
            Instruction::Div { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_div_instruction(lhs, rhs),
                );
            }
            Instruction::Eq { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_eq_instruction(lhs, rhs),
                );
            }
            Instruction::Cast { lhs, type_ } => {
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
                // TODO COMMENTS WHY
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
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_mod_instruction(lhs, rhs),
                );
            }
            Instruction::Not { lhs } => {
                self.insert_instruction_with_single_arg(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    |builder, lhs| builder.insert_not_instruction(lhs),
                );
            }
            Instruction::Shl { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_shl_instruction(lhs, rhs),
                );
            }
            Instruction::Shr { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_shr_instruction(lhs, rhs),
                );
            }
            Instruction::And { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_and_instruction(lhs, rhs),
                );
            }
            Instruction::Or { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_or_instruction(lhs, rhs),
                );
            }
            Instruction::Xor { lhs, rhs } => {
                self.insert_instruction_with_double_args(
                    acir_builder,
                    brillig_builder,
                    lhs,
                    rhs,
                    |builder, lhs, rhs| builder.insert_xor_instruction(lhs, rhs),
                );
            }
            _ => {}
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
                // If no last value was set, use the first value from the first type in each map
                let first_type =
                    self.stored_values.keys().next().expect("Should have at least one type");

                let function_result = self
                    .stored_values
                    .get(first_type)
                    .and_then(|values| values.first().cloned())
                    .expect("Should have at least one value");
                acir_builder.finalize_function(&function_result);
                brillig_builder.finalize_function(&function_result);
            }
        }
    }

    pub(crate) fn finalize_block_with_jmp(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        jmp_destination: BasicBlockId,
    ) {
        acir_builder.insert_jmp_instruction(jmp_destination);
        brillig_builder.insert_jmp_instruction(jmp_destination);
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
