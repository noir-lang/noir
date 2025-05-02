use crate::base_context::{Argument, Instruction};
use acvm::FieldElement;
use acvm::acir::native_types::Witness;
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
    /// Ids of ACIR witnesses stored as TypedValue separated by type
    pub(crate) acir_ids: HashMap<ValueType, Vec<TypedValue>>,
    /// Ids of Brillig witnesses stored as TypedValue separated by type
    pub(crate) brillig_ids: HashMap<ValueType, Vec<TypedValue>>,
    /// ACIR and Brillig last changed value
    pub(crate) last_value_acir: Option<TypedValue>,
    pub(crate) last_value_brillig: Option<TypedValue>,
    /// Depth of the block in the CFG
    pub(crate) depth: usize,
    /// Parent blocks history
    pub(crate) parent_blocks_history: VecDeque<BasicBlockId>,
    /// Children blocks
    pub(crate) children_blocks: Vec<BasicBlockId>,
}

/// Returns a typed value from the map
/// Variables are stored in a map with type as key and vector of typed values as value
/// We use modulo to wrap index around the length of the vector, because fuzzer can produce index that is greater than the length of the vector
fn get_typed_value_from_map(
    map: &HashMap<ValueType, Vec<TypedValue>>,
    type_: ValueType,
    idx: usize,
) -> Option<TypedValue> {
    let arr = map.get(&type_);
    arr?;
    let arr = arr.unwrap();
    let value = arr.get(idx % arr.len());
    value?;
    Some(value.unwrap().clone())
}

fn append_typed_value_to_map(
    map: &mut HashMap<ValueType, Vec<TypedValue>>,
    type_: ValueType,
    value: TypedValue,
) {
    map.entry(type_.clone()).or_default().push(value);
}

impl BlockContext {
    pub(crate) fn new(
        acir_ids: HashMap<ValueType, Vec<TypedValue>>,
        brillig_ids: HashMap<ValueType, Vec<TypedValue>>,
        parent_blocks_history: VecDeque<BasicBlockId>,
        depth: usize,
    ) -> Self {
        Self {
            acir_ids,
            brillig_ids,
            last_value_acir: None,
            last_value_brillig: None,
            parent_blocks_history,
            children_blocks: Vec::new(),
            depth,
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
        let acir_arg = get_typed_value_from_map(&self.acir_ids, arg.value_type.clone(), arg.index);
        let brillig_arg =
            get_typed_value_from_map(&self.brillig_ids, arg.value_type.clone(), arg.index);
        let (acir_arg, brillig_arg) = match (acir_arg, brillig_arg) {
            (Some(acir_arg), Some(brillig_arg)) => (acir_arg, brillig_arg),
            _ => return,
        };
        let acir_result = instruction(acir_builder, acir_arg);
        let brillig_result = instruction(brillig_builder, brillig_arg);
        self.last_value_acir = Some(acir_result.clone());
        self.last_value_brillig = Some(brillig_result.clone());
        append_typed_value_to_map(&mut self.acir_ids, acir_result.to_value_type(), acir_result);
        append_typed_value_to_map(
            &mut self.brillig_ids,
            brillig_result.to_value_type(),
            brillig_result,
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
        let acir_lhs = get_typed_value_from_map(&self.acir_ids, lhs.value_type.clone(), lhs.index);
        let acir_rhs = get_typed_value_from_map(&self.acir_ids, rhs.value_type.clone(), rhs.index);
        let (acir_lhs, acir_rhs) = match (acir_lhs, acir_rhs) {
            (Some(acir_lhs), Some(acir_rhs)) => (acir_lhs, acir_rhs),
            _ => return,
        };
        let acir_result = instruction(acir_builder, acir_lhs, acir_rhs);
        let brillig_lhs =
            get_typed_value_from_map(&self.brillig_ids, lhs.value_type.clone(), lhs.index);
        let brillig_rhs =
            get_typed_value_from_map(&self.brillig_ids, rhs.value_type.clone(), rhs.index);
        let (brillig_lhs, brillig_rhs) = match (brillig_lhs, brillig_rhs) {
            (Some(brillig_lhs), Some(brillig_rhs)) => (brillig_lhs, brillig_rhs),
            _ => return,
        };
        let brillig_result = instruction(brillig_builder, brillig_lhs, brillig_rhs);
        self.last_value_acir = Some(acir_result.clone());
        self.last_value_brillig = Some(brillig_result.clone());
        append_typed_value_to_map(&mut self.acir_ids, acir_result.to_value_type(), acir_result);
        append_typed_value_to_map(
            &mut self.brillig_ids,
            brillig_result.to_value_type(),
            brillig_result,
        );
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
                let acir_lhs =
                    get_typed_value_from_map(&self.acir_ids, lhs.value_type.clone(), lhs.index);
                let brillig_lhs =
                    get_typed_value_from_map(&self.brillig_ids, lhs.value_type.clone(), lhs.index);
                let (acir_lhs, brillig_lhs) = match (acir_lhs, brillig_lhs) {
                    (Some(acir_lhs), Some(brillig_lhs)) => (acir_lhs, brillig_lhs),
                    _ => return,
                };
                let acir_result = acir_builder.insert_cast(acir_lhs, type_.clone());
                let brillig_result = brillig_builder.insert_cast(brillig_lhs, type_.clone());
                self.last_value_acir = Some(acir_result.clone());
                self.last_value_brillig = Some(brillig_result.clone());
                append_typed_value_to_map(
                    &mut self.acir_ids,
                    acir_result.to_value_type(),
                    acir_result,
                );
                append_typed_value_to_map(
                    &mut self.brillig_ids,
                    brillig_result.to_value_type(),
                    brillig_result,
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
        }
    }

    pub(crate) fn insert_instructions(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        instructions: Vec<Instruction>,
    ) {
        for instruction in instructions {
            self.insert_instruction(acir_builder, brillig_builder, instruction);
        }
    }

    /// Finalizes the function by setting the return value
    pub(crate) fn finalize_block_with_return(
        self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
    ) {
        match (self.last_value_acir.clone(), self.last_value_brillig.clone()) {
            (Some(acir_result), Some(brillig_result)) => {
                acir_builder.finalize_function(acir_result);
                brillig_builder.finalize_function(brillig_result);
            }
            _ => {
                // If no last value was set, use the first value from the first type in each map
                let first_type =
                    self.acir_ids.keys().next().expect("Should have at least one type");

                let acir_result = self
                    .acir_ids
                    .get(first_type)
                    .and_then(|values| values.first().cloned())
                    .expect("Should have at least one value");

                let brillig_result = self
                    .brillig_ids
                    .get(first_type)
                    .and_then(|values| values.first().cloned())
                    .expect("Should have at least one value");

                acir_builder.finalize_function(acir_result);
                brillig_builder.finalize_function(brillig_result);
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
        let acir_condition = self
            .acir_ids
            .get(&ValueType::Boolean)
            .and_then(|values| values.last().cloned())
            .expect("Should have at least one boolean")
            .value_id;
        let brillig_condition = self
            .brillig_ids
            .get(&ValueType::Boolean)
            .and_then(|values| values.last().cloned())
            .expect("Should have at least one boolean")
            .value_id;

        acir_builder.insert_jmpif_instruction(acir_condition, then_destination, else_destination);
        brillig_builder.insert_jmpif_instruction(
            brillig_condition,
            then_destination,
            else_destination,
        );
        self.children_blocks.push(then_destination);
        self.children_blocks.push(else_destination);
    }

    pub(crate) fn get_last_variables(self) -> (Option<TypedValue>, Option<TypedValue>) {
        (self.last_value_acir, self.last_value_brillig)
    }
}
