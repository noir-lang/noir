use super::instruction::Point as InstructionPoint;
use super::instruction::Scalar as InstructionScalar;
use super::{
    ecdsa::{Curve, generate_ecdsa_signature_and_corrupt_it},
    instruction::{Argument, FunctionInfo, Instruction},
    options::SsaBlockOptions,
};
use noir_ssa_fuzzer::typed_value::{Point, Scalar};
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, InstructionWithOneArg, InstructionWithTwoArgs},
    typed_value::{TypedValue, ValueType},
};
use noirc_evaluator::ssa::ir::{basic_block::BasicBlockId, function::Function, map::Id};
use std::collections::{HashMap, VecDeque};
use std::iter::zip;

#[derive(Debug, Clone)]
pub(crate) struct StoredArray {
    array_id: TypedValue,
    element_type: ValueType,
    is_references: bool,
}

/// Main context for the ssa block containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
#[derive(Debug, Clone)]
pub(crate) struct BlockContext {
    /// Ids of the Program variables stored as TypedValue separated by type
    pub(crate) stored_variables: HashMap<ValueType, Vec<TypedValue>>,
    /// Ids of typed addresses of memory (mutable variables)
    pub(crate) memory_addresses: HashMap<ValueType, Vec<TypedValue>>,
    /// Parent blocks history
    pub(crate) parent_blocks_history: VecDeque<BasicBlockId>,
    /// Children blocks
    pub(crate) children_blocks: Vec<BasicBlockId>,
    /// Arrays stored in the block
    pub(crate) stored_arrays: Vec<StoredArray>,
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
        stored_variables: HashMap<ValueType, Vec<TypedValue>>,
        memory_addresses: HashMap<ValueType, Vec<TypedValue>>,
        parent_blocks_history: VecDeque<BasicBlockId>,
        options: SsaBlockOptions,
    ) -> Self {
        Self {
            stored_variables,
            memory_addresses,
            parent_blocks_history,
            children_blocks: Vec::new(),
            stored_arrays: Vec::new(),
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
        let value = get_typed_value_from_map(&self.stored_variables, &arg.value_type, arg.index);
        let value = match value {
            Some(value) => value,
            _ => return,
        };
        let acir_result = instruction(acir_builder, value.clone());
        // insert to brillig, assert id is the same
        assert_eq!(acir_result.value_id, instruction(brillig_builder, value).value_id);
        append_typed_value_to_map(
            &mut self.stored_variables,
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
        let instr_lhs =
            get_typed_value_from_map(&self.stored_variables, &lhs.value_type, lhs.index);
        // We ignore type of the second argument, because all binary instructions must use the same type
        let instr_rhs =
            get_typed_value_from_map(&self.stored_variables, &lhs.value_type, rhs.index);
        let (instr_lhs, instr_rhs) = match (instr_lhs, instr_rhs) {
            (Some(acir_lhs), Some(acir_rhs)) => (acir_lhs, acir_rhs),
            _ => return,
        };
        let result = instruction(acir_builder, instr_lhs.clone(), instr_rhs.clone());
        // insert to brillig, assert id of return is the same
        assert_eq!(result.value_id, instruction(brillig_builder, instr_lhs, instr_rhs).value_id);
        append_typed_value_to_map(&mut self.stored_variables, &result.to_value_type(), result);
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
                    get_typed_value_from_map(&self.stored_variables, &lhs.value_type, lhs.index);
                let value = match value {
                    Some(value) => value,
                    _ => return,
                };
                let acir_result = acir_builder.insert_cast(value.clone(), type_);
                assert_eq!(
                    acir_result.value_id,
                    brillig_builder.insert_cast(value.clone(), type_).value_id
                );
                append_typed_value_to_map(
                    &mut self.stored_variables,
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
                    get_typed_value_from_map(&self.stored_variables, &ValueType::Field, lhs);
                let rhs = get_typed_value_from_map(&self.stored_variables, &ValueType::Field, rhs);
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
                    get_typed_value_from_map(&self.stored_variables, &ValueType::Field, lhs);
                let rhs = get_typed_value_from_map(&self.stored_variables, &ValueType::Field, rhs);
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

                let value = match get_typed_value_from_map(
                    &self.stored_variables,
                    &lhs.value_type,
                    lhs.index,
                ) {
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
                    &mut self.stored_variables,
                    &value.to_value_type(),
                    value.clone(),
                );
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
                let value = get_typed_value_from_map(
                    &self.stored_variables,
                    &value.value_type,
                    value.index,
                );
                let value = match value {
                    Some(value) => value,
                    _ => return,
                };
                acir_builder.insert_set_to_memory(addr.clone(), value.clone());
                brillig_builder.insert_set_to_memory(addr.clone(), value);
            }

            Instruction::CreateArray { elements_indices, element_type, is_references } => {
                // insert to both acir and brillig builders
                let array = match self.insert_array(
                    acir_builder,
                    brillig_builder,
                    elements_indices,
                    element_type,
                    is_references,
                ) {
                    Some(array) => array,
                    _ => return,
                };
                self.stored_arrays.push(StoredArray {
                    array_id: array,
                    element_type,
                    is_references,
                });
            }
            Instruction::ArrayGet { array_index, index, safe_index } => {
                // insert array get to both acir and brillig builders
                let index = match get_typed_value_from_map(
                    &self.stored_variables,
                    &index.value_type,
                    index.index,
                ) {
                    Some(index) => index,
                    _ => return,
                };
                let value = self.insert_array_get(
                    acir_builder,
                    brillig_builder,
                    array_index,
                    index,
                    /*is constant =*/ false,
                    safe_index,
                );
                if let Some((value, is_references)) = value {
                    if !is_references {
                        append_typed_value_to_map(
                            &mut self.stored_variables,
                            &value.to_value_type(),
                            value.clone(),
                        );
                    } else {
                        panic!("References are not supported for array get with dynamic index");
                    }
                }
            }
            Instruction::ArraySet { array_index, index, value_index, safe_index } => {
                // get the index from the stored variables
                let index = get_typed_value_from_map(
                    &self.stored_variables,
                    &index.value_type,
                    index.index,
                );
                let index = match index {
                    Some(index) => index,
                    _ => return,
                };
                // cast the index to u32
                let index_casted = acir_builder.insert_cast(index.clone(), ValueType::U32);
                assert_eq!(
                    index_casted.value_id,
                    brillig_builder.insert_cast(index.clone(), ValueType::U32).value_id
                );

                // get the value from the stored variables

                // insert array set to both acir and brillig builders
                let new_array = self.insert_array_set(
                    acir_builder,
                    brillig_builder,
                    array_index,
                    index_casted,
                    /*index_is_constant =*/ false,
                    value_index,
                    safe_index,
                );
                if let Some((new_array, is_references)) = new_array {
                    if is_references {
                        panic!("References are not supported for array set with dynamic index");
                    }
                    self.stored_arrays.push(StoredArray {
                        array_id: new_array.clone(),
                        element_type: new_array.to_value_type(),
                        is_references,
                    });
                }
            }
            Instruction::ArrayGetWithConstantIndex { array_index, index, safe_index } => {
                // insert constant index
                let index_id = acir_builder.insert_constant(index, ValueType::U32);
                assert_eq!(
                    index_id.value_id,
                    brillig_builder.insert_constant(index, ValueType::U32).value_id
                );
                let value = self.insert_array_get(
                    acir_builder,
                    brillig_builder,
                    array_index,
                    index_id,
                    /*is constant =*/ true,
                    safe_index,
                );
                if let Some((value, is_references)) = value {
                    if !is_references {
                        append_typed_value_to_map(
                            &mut self.stored_variables,
                            &value.to_value_type(),
                            value.clone(),
                        );
                    } else {
                        append_typed_value_to_map(
                            &mut self.memory_addresses,
                            &value.to_value_type(),
                            value.clone(),
                        );
                    }
                }
            }
            Instruction::ArraySetWithConstantIndex {
                array_index,
                index,
                value_index,
                safe_index,
            } => {
                // insert constant index
                let index_id = acir_builder.insert_constant(index, ValueType::U32);
                assert_eq!(
                    index_id.value_id,
                    brillig_builder.insert_constant(index, ValueType::U32).value_id
                );
                let new_array = self.insert_array_set(
                    acir_builder,
                    brillig_builder,
                    array_index,
                    index_id,
                    /*index_is_constant =*/ true,
                    value_index,
                    safe_index,
                );
                if let Some((new_array, is_references)) = new_array {
                    self.stored_arrays.push(StoredArray {
                        array_id: new_array.clone(),
                        element_type: new_array.to_value_type(),
                        is_references,
                    });
                }
            }
            Instruction::FieldToBytesToField { field_idx } => {
                let field =
                    get_typed_value_from_map(&self.stored_variables, &ValueType::Field, field_idx);
                let field = match field {
                    Some(field) => field,
                    _ => return,
                };
                let bytes = acir_builder.insert_to_le_radix(field.clone(), 256, 32);
                assert_eq!(
                    bytes.value_id,
                    brillig_builder.insert_to_le_radix(field.clone(), 256, 32).value_id
                );
                let field = acir_builder.insert_from_le_radix(bytes.clone(), 256);
                assert_eq!(
                    field.value_id,
                    brillig_builder.insert_from_le_radix(bytes.clone(), 256).value_id
                );
                append_typed_value_to_map(
                    &mut self.stored_variables,
                    &field.to_value_type(),
                    field.clone(),
                );
            }
            Instruction::Blake2sHash { field_idx, limbs_count } => {
                let input =
                    get_typed_value_from_map(&self.stored_variables, &ValueType::Field, field_idx);
                let input = match input {
                    Some(input) => input,
                    _ => return,
                };
                if limbs_count == 0 {
                    return;
                }
                let bytes = acir_builder.insert_to_le_radix(input.clone(), 256, limbs_count);
                assert_eq!(
                    bytes.value_id,
                    brillig_builder.insert_to_le_radix(input.clone(), 256, limbs_count).value_id
                );
                let hash = acir_builder.insert_blake2s_hash(bytes.clone());
                assert_eq!(
                    hash.value_id,
                    brillig_builder.insert_blake2s_hash(bytes.clone()).value_id
                );
                let hash_as_field = acir_builder.insert_from_le_radix(hash.clone(), 256);
                assert_eq!(
                    hash_as_field.value_id,
                    brillig_builder.insert_from_le_radix(hash.clone(), 256).value_id
                );
                append_typed_value_to_map(
                    &mut self.stored_variables,
                    &hash_as_field.to_value_type(),
                    hash_as_field.clone(),
                );
            }
            Instruction::Blake3Hash { field_idx, limbs_count } => {
                let input =
                    get_typed_value_from_map(&self.stored_variables, &ValueType::Field, field_idx);
                let input = match input {
                    Some(input) => input,
                    _ => return,
                };
                if limbs_count == 0 {
                    return;
                }
                let bytes = acir_builder.insert_to_le_radix(input.clone(), 256, limbs_count);
                assert_eq!(
                    bytes.value_id,
                    brillig_builder.insert_to_le_radix(input.clone(), 256, limbs_count).value_id
                );
                let hash = acir_builder.insert_blake3_hash(bytes.clone());
                assert_eq!(
                    hash.value_id,
                    brillig_builder.insert_blake3_hash(bytes.clone()).value_id
                );
                let hash_as_field = acir_builder.insert_from_le_radix(hash.clone(), 256);
                assert_eq!(
                    hash_as_field.value_id,
                    brillig_builder.insert_from_le_radix(hash.clone(), 256).value_id
                );
                append_typed_value_to_map(
                    &mut self.stored_variables,
                    &hash_as_field.to_value_type(),
                    hash_as_field.clone(),
                );
            }
            Instruction::Keccakf1600Hash { u64_indices, load_elements_of_array } => {
                let input = match self.insert_array(
                    acir_builder,
                    brillig_builder,
                    u64_indices.to_vec(),
                    ValueType::U64,
                    false,
                ) {
                    Some(input) => input,
                    _ => return,
                };
                let hash_array_u64 = acir_builder.insert_keccakf1600_permutation(input.clone());
                assert_eq!(
                    hash_array_u64.value_id,
                    brillig_builder.insert_keccakf1600_permutation(input.clone()).value_id
                );
                self.stored_arrays.push(StoredArray {
                    array_id: hash_array_u64.clone(),
                    element_type: ValueType::U64,
                    is_references: false,
                });
                if load_elements_of_array {
                    for i in 0..25_u32 {
                        let index = acir_builder.insert_constant(i, ValueType::U32);
                        assert_eq!(
                            index.value_id,
                            brillig_builder.insert_constant(i, ValueType::U32).value_id
                        );
                        let value = acir_builder.insert_array_get(
                            hash_array_u64.clone(),
                            index.clone(),
                            ValueType::U64.to_ssa_type(),
                            false,
                        );
                        assert_eq!(
                            value.value_id,
                            brillig_builder
                                .insert_array_get(
                                    hash_array_u64.clone(),
                                    index.clone(),
                                    ValueType::U64.to_ssa_type(),
                                    false,
                                )
                                .value_id
                        );
                        append_typed_value_to_map(
                            &mut self.stored_variables,
                            &value.to_value_type(),
                            value.clone(),
                        );
                    }
                }
            }
            Instruction::Aes128Encrypt { input_idx, input_limbs_count, key_idx, iv_idx } => {
                if input_limbs_count == 0 {
                    return;
                }
                let input = match get_typed_value_from_map(
                    &self.stored_variables,
                    &ValueType::Field,
                    input_idx,
                ) {
                    Some(input) => input,
                    _ => return,
                };
                let key = match get_typed_value_from_map(
                    &self.stored_variables,
                    &ValueType::Field,
                    key_idx,
                ) {
                    Some(key) => key,
                    _ => return,
                };
                let iv = match get_typed_value_from_map(
                    &self.stored_variables,
                    &ValueType::Field,
                    iv_idx,
                ) {
                    Some(iv) => iv,
                    _ => return,
                };
                let input_bytes =
                    acir_builder.insert_to_le_radix(input.clone(), 256, input_limbs_count);
                assert_eq!(
                    input_bytes.value_id,
                    brillig_builder
                        .insert_to_le_radix(input.clone(), 256, input_limbs_count)
                        .value_id
                );
                let key_bytes = acir_builder.insert_to_le_radix(key.clone(), 256, 16);
                assert_eq!(
                    key_bytes.value_id,
                    brillig_builder.insert_to_le_radix(key.clone(), 256, 16).value_id
                );
                let iv_bytes = acir_builder.insert_to_le_radix(iv.clone(), 256, 16);
                assert_eq!(
                    iv_bytes.value_id,
                    brillig_builder.insert_to_le_radix(iv.clone(), 256, 16).value_id
                );
                let encrypted = acir_builder.insert_aes128_encrypt(
                    input_bytes.clone(),
                    key_bytes.clone(),
                    iv_bytes.clone(),
                );
                assert_eq!(
                    encrypted.value_id,
                    brillig_builder
                        .insert_aes128_encrypt(input_bytes, key_bytes, iv_bytes)
                        .value_id
                );
                let encrypted_as_field = acir_builder.insert_from_le_radix(encrypted.clone(), 256);
                assert_eq!(
                    encrypted_as_field.value_id,
                    brillig_builder.insert_from_le_radix(encrypted.clone(), 256).value_id
                );
                append_typed_value_to_map(
                    &mut self.stored_variables,
                    &encrypted_as_field.to_value_type(),
                    encrypted_as_field.clone(),
                );
            }
            Instruction::Sha256Compression {
                input_indices,
                state_indices,
                load_elements_of_array,
            } => {
                let input = match self.insert_array(
                    acir_builder,
                    brillig_builder,
                    input_indices.to_vec(),
                    ValueType::U32,
                    false,
                ) {
                    Some(input) => input,
                    _ => return,
                };
                let state = match self.insert_array(
                    acir_builder,
                    brillig_builder,
                    state_indices.to_vec(),
                    ValueType::U32,
                    false,
                ) {
                    Some(state) => state,
                    _ => return,
                };
                let compressed =
                    acir_builder.insert_sha256_compression(input.clone(), state.clone());
                assert_eq!(
                    compressed.value_id,
                    brillig_builder.insert_sha256_compression(input, state).value_id
                );
                self.stored_arrays.push(StoredArray {
                    array_id: compressed.clone(),
                    element_type: ValueType::U32,
                    is_references: false,
                });
                if load_elements_of_array {
                    for i in 0..8_u32 {
                        let index = acir_builder.insert_constant(i, ValueType::U32);
                        assert_eq!(
                            index.value_id,
                            brillig_builder.insert_constant(i, ValueType::U32).value_id
                        );
                        let value = acir_builder.insert_array_get(
                            compressed.clone(),
                            index.clone(),
                            ValueType::U32.to_ssa_type(),
                            false,
                        );
                        assert_eq!(
                            value.value_id,
                            brillig_builder
                                .insert_array_get(
                                    compressed.clone(),
                                    index,
                                    ValueType::U32.to_ssa_type(),
                                    false
                                )
                                .value_id
                        );
                        append_typed_value_to_map(
                            &mut self.stored_variables,
                            &value.to_value_type(),
                            value.clone(),
                        );
                    }
                }
            }
            Instruction::PointAdd { p1, p2 } => {
                if !self.options.instruction_options.point_add_enabled {
                    return;
                }
                let p1 = self.ssa_point_from_instruction_point(acir_builder, brillig_builder, p1);
                let p2 = self.ssa_point_from_instruction_point(acir_builder, brillig_builder, p2);
                if p1.is_none() || p2.is_none() {
                    return;
                }
                let p1 = p1.unwrap();
                let p2 = p2.unwrap();
                let acir_point = acir_builder.point_add(p1.clone(), p2.clone());
                let brillig_point = brillig_builder.point_add(p1, p2);
                assert_eq!(acir_point, brillig_point);
                for typed_value in [&acir_point.x, &acir_point.y, &acir_point.is_infinite] {
                    append_typed_value_to_map(
                        &mut self.stored_variables,
                        &typed_value.to_value_type(),
                        typed_value.clone(),
                    );
                }
            }
            Instruction::MultiScalarMul { points_and_scalars } => {
                if !self.options.instruction_options.multi_scalar_mul_enabled {
                    return;
                }
                let mut points_vec = Vec::new();
                let mut scalars_vec = Vec::new();
                for (p, s) in points_and_scalars.iter() {
                    let point =
                        self.ssa_point_from_instruction_point(acir_builder, brillig_builder, *p);
                    let scalar = self.ssa_scalar_from_instruction_scalar(*s);
                    if point.is_none() || scalar.is_none() {
                        continue;
                    }
                    points_vec.push(point.unwrap());
                    scalars_vec.push(scalar.unwrap());
                }
                if points_vec.is_empty() || scalars_vec.is_empty() {
                    return;
                }
                if points_vec.len() != scalars_vec.len() {
                    unreachable!("points_vec.len() != scalars_vec.len()");
                }
                let acir_point =
                    acir_builder.multi_scalar_mul(points_vec.clone(), scalars_vec.clone());
                let brillig_point = brillig_builder.multi_scalar_mul(points_vec, scalars_vec);
                assert_eq!(acir_point, brillig_point);
                for typed_value in [&acir_point.x, &acir_point.y, &acir_point.is_infinite] {
                    append_typed_value_to_map(
                        &mut self.stored_variables,
                        &typed_value.to_value_type(),
                        typed_value.clone(),
                    );
                }
            }
            Instruction::EcdsaSecp256r1 {
                msg,
                hash_size,
                corrupt_hash,
                corrupt_pubkey_x,
                corrupt_pubkey_y,
                corrupt_signature,
            } => {
                if !self.options.instruction_options.ecdsa_secp256r1_enabled {
                    return;
                }
                let prepared_signature = generate_ecdsa_signature_and_corrupt_it(
                    &msg,
                    Curve::Secp256r1,
                    corrupt_hash,
                    corrupt_pubkey_x,
                    corrupt_pubkey_y,
                    corrupt_signature,
                );
                let result = acir_builder.ecdsa_secp256r1(
                    prepared_signature.public_key_x.clone(),
                    prepared_signature.public_key_y.clone(),
                    prepared_signature.hash.clone(),
                    hash_size,
                    prepared_signature.signature.clone(),
                );
                assert_eq!(
                    result.value_id,
                    brillig_builder
                        .ecdsa_secp256r1(
                            prepared_signature.public_key_x,
                            prepared_signature.public_key_y,
                            prepared_signature.hash,
                            hash_size,
                            prepared_signature.signature,
                        )
                        .value_id
                );
                append_typed_value_to_map(
                    &mut self.stored_variables,
                    &result.to_value_type(),
                    result.clone(),
                );
            }
            Instruction::EcdsaSecp256k1 {
                msg,
                hash_size,
                corrupt_hash,
                corrupt_pubkey_x,
                corrupt_pubkey_y,
                corrupt_signature,
            } => {
                if !self.options.instruction_options.ecdsa_secp256k1_enabled {
                    return;
                }
                let prepared_signature = generate_ecdsa_signature_and_corrupt_it(
                    &msg,
                    Curve::Secp256k1,
                    corrupt_hash,
                    corrupt_pubkey_x,
                    corrupt_pubkey_y,
                    corrupt_signature,
                );
                let result = acir_builder.ecdsa_secp256k1(
                    prepared_signature.public_key_x.clone(),
                    prepared_signature.public_key_y.clone(),
                    prepared_signature.hash.clone(),
                    hash_size,
                    prepared_signature.signature.clone(),
                );
                assert_eq!(
                    result.value_id,
                    brillig_builder
                        .ecdsa_secp256k1(
                            prepared_signature.public_key_x,
                            prepared_signature.public_key_y,
                            prepared_signature.hash,
                            hash_size,
                            prepared_signature.signature,
                        )
                        .value_id
                );
                append_typed_value_to_map(
                    &mut self.stored_variables,
                    &result.to_value_type(),
                    result.clone(),
                );
            }
        }
    }

    /// Takes scalar from [`super::instruction::Scalar`] and converts it to [`noir_ssa_fuzzer::typed_value::Scalar`]
    fn ssa_scalar_from_instruction_scalar(&mut self, scalar: InstructionScalar) -> Option<Scalar> {
        let lo = get_typed_value_from_map(
            &self.stored_variables,
            &ValueType::Field,
            scalar.field_lo_idx,
        );
        let hi = get_typed_value_from_map(
            &self.stored_variables,
            &ValueType::Field,
            scalar.field_hi_idx,
        );
        match (lo, hi) {
            (Some(lo), Some(hi)) => Some(Scalar { lo, hi }),
            _ => None,
        }
    }

    /// Takes point from [`super::instruction::Point`] and converts it to [`noir_ssa_fuzzer::typed_value::Point`]
    fn ssa_point_from_instruction_point(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        point: InstructionPoint,
    ) -> Option<Point> {
        let scalar = self.ssa_scalar_from_instruction_scalar(point.scalar);
        scalar.as_ref()?; // wtf clippy forbid me to write if scalar.is_none() {return None}
        let scalar = scalar.unwrap();
        let is_infinite = acir_builder.insert_constant(point.is_infinite, ValueType::Boolean);
        assert_eq!(
            is_infinite.value_id,
            brillig_builder.insert_constant(point.is_infinite, ValueType::Boolean).value_id
        );

        let point = if point.derive_from_scalar_mul {
            let acir_point = acir_builder.base_scalar_mul(scalar.clone(), is_infinite.clone());
            let brillig_point = brillig_builder.base_scalar_mul(scalar, is_infinite);
            assert_eq!(acir_point, brillig_point);
            acir_point
        } else {
            let acir_point =
                acir_builder.create_point_from_scalar(scalar.clone(), is_infinite.clone());
            let brillig_point = brillig_builder.create_point_from_scalar(scalar, is_infinite);
            assert_eq!(acir_point, brillig_point);
            acir_point
        };
        Some(point)
    }

    fn insert_array(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        elements_indices: Vec<usize>,
        element_type: ValueType,
        is_references: bool,
    ) -> Option<TypedValue> {
        if !self.options.instruction_options.create_array_enabled {
            return None;
        }
        // if we storing references, take values from memory addresses, otherwise from stored variables
        let elements = if !is_references {
            elements_indices
                .iter()
                .map(|index| {
                    get_typed_value_from_map(&self.stored_variables, &element_type, *index)
                })
                .collect::<Option<Vec<TypedValue>>>()
        } else {
            elements_indices
                .iter()
                .map(|index| {
                    get_typed_value_from_map(&self.memory_addresses, &element_type, *index)
                })
                .collect::<Option<Vec<TypedValue>>>()
        };

        let elements = match elements {
            Some(elements) => elements,
            _ => return None,
        };
        if elements.is_empty() {
            return None;
        }
        let array = acir_builder.insert_array(elements.clone(), is_references);
        assert_eq!(array.value_id, brillig_builder.insert_array(elements, is_references).value_id);
        Some(array)
    }

    /// Inserts an array get instruction
    ///
    /// # Arguments
    ///
    /// * `array_index` - Index of the array in the stored arrays
    /// * `index` - Index of the element in the array
    /// * `index_is_constant` - If true, the index is created from a constant
    /// * `safe_index` - If true, the index will be taken modulo the array length
    ///
    /// # Returns
    /// * (TypedValue, is_references)
    /// * None if the instruction is not enabled or the array is not stored
    fn insert_array_get(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        array_index: usize,
        index: TypedValue,
        index_is_constant: bool,
        safe_index: bool,
    ) -> Option<(TypedValue, bool)> {
        if !self.options.instruction_options.array_get_enabled
            || !self.options.instruction_options.create_array_enabled
        {
            return None;
        }
        if !safe_index && !self.options.instruction_options.unsafe_get_set_enabled {
            return None;
        }
        if self.stored_arrays.is_empty() {
            return None;
        }
        // get the array from the stored arrays
        let stored_array = self.stored_arrays.get(array_index % self.stored_arrays.len());
        let stored_array = match stored_array {
            Some(stored_array) => stored_array,
            _ => return None,
        };
        // references are not supported for array get with dynamic index
        if stored_array.is_references && !index_is_constant {
            return None;
        }
        let array_id = stored_array.array_id.clone();
        // cast the index to u32
        let index_casted = acir_builder.insert_cast(index.clone(), ValueType::U32);
        assert_eq!(
            index_casted.value_id,
            brillig_builder.insert_cast(index.clone(), ValueType::U32).value_id
        );
        let value = acir_builder.insert_array_get(
            array_id.clone(),
            index_casted.clone(),
            stored_array.element_type.to_ssa_type(),
            safe_index,
        );
        assert_eq!(
            value.value_id,
            brillig_builder
                .insert_array_get(
                    array_id,
                    index_casted,
                    stored_array.element_type.to_ssa_type(),
                    safe_index
                )
                .value_id
        );
        Some((value, stored_array.is_references))
    }

    /// Inserts an array set instruction
    ///
    /// # Arguments
    ///
    /// * `array_index` - Index of the array in the stored arrays
    /// * `index` - Index of the element in the array
    /// * `index_is_constant` - If true, the index is created from a constant
    /// * `safe_index` - If true, the index will be taken modulo the array length
    ///
    /// # Returns
    /// * (TypedValue referencing the new array, is_references)
    /// * None if the instruction is not enabled or the array is not stored
    #[allow(clippy::too_many_arguments)] // I don't want this refactoring
    fn insert_array_set(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        array_index: usize,
        index: TypedValue,
        index_is_constant: bool,
        value_index: usize,
        safe_index: bool,
    ) -> Option<(TypedValue, bool)> {
        if !self.options.instruction_options.array_set_enabled {
            return None;
        }
        if !safe_index && !self.options.instruction_options.unsafe_get_set_enabled {
            return None;
        }
        if self.stored_arrays.is_empty() {
            return None;
        }
        // get the array from the stored arrays
        let stored_array = self.stored_arrays.get(array_index % self.stored_arrays.len());
        let stored_array = match stored_array {
            Some(stored_array) => stored_array,
            _ => return None,
        };
        // references are not supported for array set with dynamic index
        if stored_array.is_references && !index_is_constant {
            return None;
        }
        let array_id = stored_array.array_id.clone();
        // get the value from the stored variables if not references, otherwise from memory addresses
        let value = if !stored_array.is_references {
            get_typed_value_from_map(
                &self.stored_variables,
                &stored_array.element_type,
                value_index,
            )
        } else {
            get_typed_value_from_map(
                &self.memory_addresses,
                &stored_array.element_type,
                value_index,
            )
        };
        let value = match value {
            Some(value) => value,
            _ => return None,
        };
        let new_array = acir_builder.insert_array_set(
            array_id.clone(),
            index.clone(),
            value.clone(),
            safe_index,
        );
        assert_eq!(
            new_array.value_id,
            brillig_builder.insert_array_set(array_id, index, value, safe_index).value_id
        );
        Some((new_array, stored_array.is_references))
    }

    pub(crate) fn insert_instructions(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        instructions: &Vec<Instruction>,
    ) {
        for instruction in instructions {
            self.insert_instruction(acir_builder, brillig_builder, instruction.clone());
        }
    }

    /// Finalizes the function by setting the return value
    pub(crate) fn finalize_block_with_return(
        self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        return_type: ValueType,
    ) {
        let array_of_values_with_return_type = self.stored_variables.get(&return_type);
        let return_value = match array_of_values_with_return_type {
            Some(arr) => arr.iter().last(),
            _ => None,
        };
        match return_value {
            Some(return_value) => {
                acir_builder.finalize_function(return_value);
                brillig_builder.finalize_function(return_value);
            }
            _ => {
                // If no last value was set, we take a boolean that is definitely set and cast it to the return type
                let boolean_value =
                    get_typed_value_from_map(&self.stored_variables, &ValueType::Boolean, 0)
                        .unwrap();
                let return_value = acir_builder.insert_cast(boolean_value.clone(), return_type);
                assert_eq!(brillig_builder.insert_cast(boolean_value, return_type), return_value);
                acir_builder.finalize_function(&return_value);
                brillig_builder.finalize_function(&return_value);
            }
        }
    }

    pub(crate) fn finalize_block_with_jmp(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        jmp_destination: BasicBlockId,
        args: Vec<TypedValue>,
    ) {
        acir_builder.insert_jmp_instruction(jmp_destination, args.clone());
        brillig_builder.insert_jmp_instruction(jmp_destination, args);
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
            .stored_variables
            .get(&ValueType::Boolean)
            .and_then(|values| values.last().cloned())
            .expect("Should have at least one boolean")
            .value_id;

        acir_builder.insert_jmpif_instruction(condition, then_destination, else_destination);
        brillig_builder.insert_jmpif_instruction(condition, then_destination, else_destination);
        self.children_blocks.push(then_destination);
        self.children_blocks.push(else_destination);
    }

    /// Inserts a function call to the given function with the given arguments and result type
    pub(crate) fn process_function_call(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        function_id: Id<Function>,
        function_signature: FunctionInfo,
        args: &[usize],
    ) {
        // On SSA level you cannot just call a function by its id, you need to import it first
        let func_as_value_id = acir_builder.insert_import(function_id);
        assert_eq!(func_as_value_id, brillig_builder.insert_import(function_id));

        // Get values from stored_values map by indices
        // If we don't have some value of type of the argument, we skip the function call
        let mut values = vec![];
        for (value_type, index) in zip(function_signature.input_types, args) {
            let value = match get_typed_value_from_map(&self.stored_variables, &value_type, *index)
            {
                Some(value) => value,
                None => return,
            };

            values.push(value);
        }

        // Insert a call to the function with the given arguments and result type
        let ret_val =
            acir_builder.insert_call(func_as_value_id, &values, function_signature.return_type);
        assert_eq!(
            ret_val,
            brillig_builder.insert_call(func_as_value_id, &values, function_signature.return_type)
        );
        let typed_ret_val = TypedValue {
            value_id: ret_val,
            type_of_variable: function_signature.return_type.to_ssa_type(),
        };
        // Append the return value to stored_values map
        append_typed_value_to_map(
            &mut self.stored_variables,
            &function_signature.return_type,
            typed_ret_val,
        );
    }
}
