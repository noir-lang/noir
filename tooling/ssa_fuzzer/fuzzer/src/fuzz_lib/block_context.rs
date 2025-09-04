use super::instruction::Point as InstructionPoint;
use super::instruction::Scalar as InstructionScalar;
use super::{
    ecdsa::{Curve, generate_ecdsa_signature_and_corrupt_it},
    instruction::{FunctionInfo, Instruction, NumericArgument},
    options::SsaBlockOptions,
};
use noir_ssa_fuzzer::builder::{FuzzerBuilder, InstructionWithOneArg, InstructionWithTwoArgs};
use noir_ssa_fuzzer::typed_value::{NumericType, Point, Scalar, Type, TypedValue};
use noirc_evaluator::ssa::ir::{basic_block::BasicBlockId, function::Function, map::Id};
use std::collections::{HashMap, VecDeque};
use std::iter::zip;

/// Main context for the ssa block containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
#[derive(Debug, Clone)]
pub(crate) struct BlockContext {
    /// Ids of the Program variables stored as TypedValue separated by type
    pub(crate) stored_variables: HashMap<Type, Vec<TypedValue>>,
    /// Parent blocks history
    pub(crate) parent_blocks_history: VecDeque<BasicBlockId>,
    /// Children blocks
    pub(crate) children_blocks: Vec<BasicBlockId>,
    /// Options for the block
    pub(crate) options: SsaBlockOptions,
}

impl BlockContext {
    pub(crate) fn new(
        stored_variables: HashMap<Type, Vec<TypedValue>>,
        parent_blocks_history: VecDeque<BasicBlockId>,
        options: SsaBlockOptions,
    ) -> Self {
        Self { stored_variables, parent_blocks_history, children_blocks: Vec::new(), options }
    }

    fn store_variable(&mut self, value: &TypedValue) {
        self.stored_variables
            .entry(value.type_of_variable.clone())
            .or_default()
            .push(value.clone());
    }

    fn get_stored_variable(&self, type_: &Type, index: usize) -> Option<TypedValue> {
        let arr = self.stored_variables.get(type_)?;
        let value = arr.get(index % arr.len())?;
        Some(value.clone())
    }

    fn get_stored_arrays(&self) -> Vec<TypedValue> {
        self.stored_variables
            .iter()
            .filter(|(key, _)| key.is_array())
            .flat_map(|(_, arr)| arr.clone())
            .collect()
    }

    fn get_stored_references_to_type(&self, type_: &Type) -> Vec<TypedValue> {
        self.stored_variables
            .iter()
            .filter(|(key, _)| key.is_reference() && key.unwrap_reference() == *type_)
            .flat_map(|(_, arr)| arr.clone())
            .collect()
    }

    /// Inserts an instruction that takes a single argument
    fn insert_instruction_with_single_arg(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        arg: NumericArgument,
        instruction: InstructionWithOneArg,
    ) {
        let value = self.get_stored_variable(&Type::Numeric(arg.numeric_type), arg.index);
        let value = match value {
            Some(value) => value,
            _ => return,
        };
        let acir_result = instruction(acir_builder, value.clone());
        // insert to brillig, assert id is the same
        assert_eq!(acir_result, instruction(brillig_builder, value));
        self.store_variable(&acir_result);
    }

    /// Inserts an instruction that takes two arguments
    fn insert_instruction_with_double_args(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        lhs: NumericArgument,
        rhs: NumericArgument,
        instruction: InstructionWithTwoArgs,
    ) {
        let instr_lhs = self.get_stored_variable(&Type::Numeric(lhs.numeric_type), lhs.index);
        // We ignore type of the second argument, because all binary instructions must use the same type
        let instr_rhs = self.get_stored_variable(&Type::Numeric(lhs.numeric_type), rhs.index);
        let (instr_lhs, instr_rhs) = match (instr_lhs, instr_rhs) {
            (Some(acir_lhs), Some(acir_rhs)) => (acir_lhs, acir_rhs),
            _ => return,
        };
        let result = instruction(acir_builder, instr_lhs.clone(), instr_rhs.clone());
        // insert to brillig, assert id of return is the same
        assert_eq!(result, instruction(brillig_builder, instr_lhs, instr_rhs));
        self.store_variable(&result);
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
                let value = self.get_stored_variable(&Type::Numeric(lhs.numeric_type), lhs.index);
                let value = match value {
                    Some(value) => value,
                    _ => return,
                };
                let acir_result = acir_builder.insert_cast(value.clone(), Type::Numeric(type_));
                assert_eq!(
                    acir_result,
                    brillig_builder.insert_cast(value.clone(), Type::Numeric(type_))
                );
                self.store_variable(&acir_result);
            }
            Instruction::Mod { lhs, rhs } => {
                if !self.options.instruction_options.mod_enabled {
                    return;
                }
                if lhs.numeric_type == NumericType::Field {
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
                if lhs.numeric_type == NumericType::Field {
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
                if lhs.numeric_type == NumericType::Field {
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
                if lhs.numeric_type == NumericType::Field {
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
                if lhs.numeric_type == NumericType::Field {
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
                if lhs.numeric_type == NumericType::Field {
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
                let lhs_orig = self.get_stored_variable(&Type::Numeric(NumericType::Field), lhs);
                let rhs = self.get_stored_variable(&Type::Numeric(NumericType::Field), rhs);
                let (lhs_orig, rhs) = match (lhs_orig, rhs) {
                    (Some(lhs_orig), Some(rhs)) => (lhs_orig, rhs),
                    _ => return,
                };
                // assert ids of add are the same for both builders
                let lhs_add_rhs =
                    acir_builder.insert_add_instruction_checked(lhs_orig.clone(), rhs.clone());
                assert_eq!(
                    lhs_add_rhs,
                    brillig_builder.insert_add_instruction_checked(lhs_orig.clone(), rhs.clone())
                );
                // inserts lhs'' = lhs' - rhs
                let lhs = lhs_add_rhs;
                let morphed = acir_builder.insert_sub_instruction_checked(lhs.clone(), rhs.clone());

                // assert ids of sub are the same for both builders
                assert_eq!(
                    morphed,
                    brillig_builder.insert_sub_instruction_checked(lhs.clone(), rhs.clone())
                );

                if !self.options.constrain_idempotent_enabled {
                    return;
                }

                acir_builder.insert_constrain(lhs_orig.clone(), morphed.clone());
                brillig_builder.insert_constrain(lhs_orig.clone(), morphed.clone());
            }
            Instruction::MulDivConstrain { lhs, rhs } => {
                let lhs_orig = self.get_stored_variable(&Type::Numeric(NumericType::Field), lhs);
                let rhs = self.get_stored_variable(&Type::Numeric(NumericType::Field), rhs);
                let (lhs_orig, rhs) = match (lhs_orig, rhs) {
                    (Some(lhs_orig), Some(rhs)) => (lhs_orig, rhs),
                    _ => return,
                };
                // inserts lhs' = lhs * rhs
                // assert ids of mul are the same for both builders
                let lhs_mul_rhs =
                    acir_builder.insert_mul_instruction_checked(lhs_orig.clone(), rhs.clone());
                assert_eq!(
                    lhs_mul_rhs,
                    brillig_builder.insert_mul_instruction_checked(lhs_orig.clone(), rhs.clone())
                );
                // lhs'' = lhs' / rhs
                let lhs = lhs_mul_rhs;
                // insert to both builders, assert ids of div are the same
                let morphed = acir_builder.insert_div_instruction(lhs.clone(), rhs.clone());
                assert_eq!(
                    morphed,
                    brillig_builder.insert_div_instruction(lhs.clone(), rhs.clone())
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

                let value = match self.get_stored_variable(&lhs.value_type, lhs.index) {
                    Some(value) => value,
                    _ => return,
                };

                let addr = acir_builder.insert_add_to_memory(value.clone());
                assert_eq!(addr, brillig_builder.insert_add_to_memory(value.clone()));
                self.store_variable(&addr);
            }
            Instruction::LoadFromMemory { memory_addr } => {
                if !self.options.instruction_options.load_enabled {
                    return;
                }
                let addresses = self.get_stored_references_to_type(&memory_addr.value_type);
                if addresses.is_empty() {
                    return;
                }
                let address = addresses[memory_addr.index % addresses.len()].clone();
                let value = acir_builder.insert_load_from_memory(address.clone());
                assert_eq!(value, brillig_builder.insert_load_from_memory(address.clone()));
                self.store_variable(&value);
            }
            Instruction::SetToMemory { memory_addr_index, value } => {
                if !self.options.instruction_options.store_enabled {
                    return;
                }
                let addresses = self.get_stored_references_to_type(&value.value_type);
                let value = match self.get_stored_variable(&value.value_type, value.index) {
                    Some(value) => value,
                    _ => return,
                };
                let address = if addresses.is_empty() {
                    let addr = acir_builder.insert_add_to_memory(value.clone());
                    assert_eq!(addr, brillig_builder.insert_add_to_memory(value.clone()));
                    self.store_variable(&addr);
                    addr
                } else {
                    addresses[memory_addr_index % addresses.len()].clone()
                };
                acir_builder.insert_set_to_memory(address.clone(), value.clone());
                brillig_builder.insert_set_to_memory(address, value);
            }

            Instruction::CreateArray { elements_indices, element_type } => {
                // insert to both acir and brillig builders
                let array = match self.insert_array(
                    acir_builder,
                    brillig_builder,
                    elements_indices,
                    element_type,
                ) {
                    Some(array) => array,
                    _ => return,
                };
                self.store_variable(&array);
            }
            Instruction::ArrayGet { array_index, index, safe_index } => {
                // insert array get to both acir and brillig builders
                let index = match self
                    .get_stored_variable(&Type::Numeric(index.numeric_type), index.index)
                {
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
                        self.store_variable(&value);
                    } else {
                        panic!("References are not supported for array get with dynamic index");
                    }
                }
            }
            Instruction::ArraySet { array_index, index, value_index, safe_index } => {
                // get the index from the stored variables
                let index =
                    self.get_stored_variable(&Type::Numeric(index.numeric_type), index.index);
                let index = match index {
                    Some(index) => index,
                    _ => return,
                };
                // cast the index to u32
                let index_casted =
                    acir_builder.insert_cast(index.clone(), Type::Numeric(NumericType::U32));
                assert_eq!(
                    index_casted,
                    brillig_builder.insert_cast(index.clone(), Type::Numeric(NumericType::U32))
                );

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
                    let element_type = new_array.type_of_variable.unwrap_array_element_type();
                    match element_type {
                        Type::Numeric(_element_type) => {
                            self.store_variable(&new_array);
                        }
                        _ => panic!("Expected NumericType, found {element_type:?}"),
                    }
                }
            }
            Instruction::ArrayGetWithConstantIndex { array_index, index, safe_index } => {
                // insert constant index
                let index_id = acir_builder.insert_constant(index, NumericType::U32);
                assert_eq!(index_id, brillig_builder.insert_constant(index, NumericType::U32));
                let value = self.insert_array_get(
                    acir_builder,
                    brillig_builder,
                    array_index,
                    index_id,
                    /*is constant =*/ true,
                    safe_index,
                );
                if let Some((value, _is_references)) = value {
                    self.store_variable(&value);
                }
            }
            Instruction::ArraySetWithConstantIndex {
                array_index,
                index,
                value_index,
                safe_index,
            } => {
                // insert constant index
                let index_id = acir_builder.insert_constant(index, NumericType::U32);
                assert_eq!(index_id, brillig_builder.insert_constant(index, NumericType::U32));
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
                    let element_type = match new_array.type_of_variable.clone() {
                        Type::Array(elements_type, _) => elements_type[0].clone(),
                        _ => panic!("Expected ArrayType, found {:?}", new_array.type_of_variable),
                    };
                    match element_type {
                        Type::Numeric(_element_type) => {
                            assert!(
                                !is_references,
                                "Encountered numeric element in an array with references"
                            );
                            self.store_variable(&new_array);
                        }
                        Type::Reference(type_ref) => {
                            assert!(
                                is_references,
                                "Encountered reference element in an array without references"
                            );
                            assert!(
                                type_ref.is_numeric(),
                                "Expected reference to a numeric type, found {type_ref:?}"
                            );
                            self.store_variable(&new_array);
                        }
                        _ => {
                            panic!("Expected NumericType or ReferenceType, found {element_type:?}")
                        }
                    }
                }
            }
            Instruction::FieldToBytesToField { field_idx } => {
                let field = self.get_stored_variable(&Type::Numeric(NumericType::Field), field_idx);
                let field = match field {
                    Some(field) => field,
                    _ => return,
                };
                let bytes = acir_builder.insert_to_le_radix(field.clone(), 256, 32);
                assert_eq!(bytes, brillig_builder.insert_to_le_radix(field.clone(), 256, 32));
                let field = acir_builder.insert_from_le_radix(bytes.clone(), 256);
                assert_eq!(field, brillig_builder.insert_from_le_radix(bytes.clone(), 256));
                self.store_variable(&field);
            }
            Instruction::Blake2sHash { field_idx, limbs_count } => {
                let input = self.get_stored_variable(&Type::Numeric(NumericType::Field), field_idx);
                let input = match input {
                    Some(input) => input,
                    _ => return,
                };
                if limbs_count == 0 {
                    return;
                }
                let bytes = acir_builder.insert_to_le_radix(input.clone(), 256, limbs_count);
                assert_eq!(
                    bytes,
                    brillig_builder.insert_to_le_radix(input.clone(), 256, limbs_count)
                );
                let hash = acir_builder.insert_blake2s_hash(bytes.clone());
                assert_eq!(hash, brillig_builder.insert_blake2s_hash(bytes.clone()));
                let hash_as_field = acir_builder.insert_from_le_radix(hash.clone(), 256);
                assert_eq!(hash_as_field, brillig_builder.insert_from_le_radix(hash.clone(), 256));
                self.store_variable(&hash_as_field);
            }
            Instruction::Blake3Hash { field_idx, limbs_count } => {
                let input = self.get_stored_variable(&Type::Numeric(NumericType::Field), field_idx);
                let input = match input {
                    Some(input) => input,
                    _ => return,
                };
                if limbs_count == 0 {
                    return;
                }
                let bytes = acir_builder.insert_to_le_radix(input.clone(), 256, limbs_count);
                assert_eq!(
                    bytes,
                    brillig_builder.insert_to_le_radix(input.clone(), 256, limbs_count)
                );
                let hash = acir_builder.insert_blake3_hash(bytes.clone());
                assert_eq!(hash, brillig_builder.insert_blake3_hash(bytes.clone()));
                let hash_as_field = acir_builder.insert_from_le_radix(hash.clone(), 256);
                assert_eq!(hash_as_field, brillig_builder.insert_from_le_radix(hash.clone(), 256));
                self.store_variable(&hash_as_field);
            }
            Instruction::Keccakf1600Hash { u64_indices, load_elements_of_array } => {
                let input = match self.insert_array(
                    acir_builder,
                    brillig_builder,
                    u64_indices.to_vec(),
                    Type::Numeric(NumericType::U64),
                ) {
                    Some(input) => input,
                    _ => return,
                };
                let hash_array_u64 = acir_builder.insert_keccakf1600_permutation(input.clone());
                assert_eq!(
                    hash_array_u64,
                    brillig_builder.insert_keccakf1600_permutation(input.clone())
                );
                self.store_variable(&hash_array_u64);
                if load_elements_of_array {
                    for i in 0..25_u32 {
                        let index = acir_builder.insert_constant(i, NumericType::U32);
                        assert_eq!(index, brillig_builder.insert_constant(i, NumericType::U32));
                        let value = acir_builder.insert_array_get(
                            hash_array_u64.clone(),
                            index.clone(),
                            Type::Numeric(NumericType::U64),
                            /*safe_index =*/ false,
                        );
                        assert_eq!(
                            value,
                            brillig_builder.insert_array_get(
                                hash_array_u64.clone(),
                                index.clone(),
                                Type::Numeric(NumericType::U64),
                                /*safe_index =*/ false
                            )
                        );
                        self.store_variable(&value);
                    }
                }
            }
            Instruction::Aes128Encrypt { input_idx, input_limbs_count, key_idx, iv_idx } => {
                if input_limbs_count == 0 {
                    return;
                }
                let input =
                    match self.get_stored_variable(&Type::Numeric(NumericType::Field), input_idx) {
                        Some(input) => input,
                        _ => return,
                    };
                let key =
                    match self.get_stored_variable(&Type::Numeric(NumericType::Field), key_idx) {
                        Some(key) => key,
                        _ => return,
                    };
                let iv = match self.get_stored_variable(&Type::Numeric(NumericType::Field), iv_idx)
                {
                    Some(iv) => iv,
                    _ => return,
                };
                let input_bytes =
                    acir_builder.insert_to_le_radix(input.clone(), 256, input_limbs_count);
                assert_eq!(
                    input_bytes,
                    brillig_builder.insert_to_le_radix(input.clone(), 256, input_limbs_count)
                );
                let key_bytes = acir_builder.insert_to_le_radix(key.clone(), 256, 16);
                assert_eq!(key_bytes, brillig_builder.insert_to_le_radix(key.clone(), 256, 16));
                let iv_bytes = acir_builder.insert_to_le_radix(iv.clone(), 256, 16);
                assert_eq!(iv_bytes, brillig_builder.insert_to_le_radix(iv.clone(), 256, 16));
                let encrypted = acir_builder.insert_aes128_encrypt(
                    input_bytes.clone(),
                    key_bytes.clone(),
                    iv_bytes.clone(),
                );
                assert_eq!(
                    encrypted,
                    brillig_builder.insert_aes128_encrypt(input_bytes, key_bytes, iv_bytes)
                );
                let encrypted_as_field = acir_builder.insert_from_le_radix(encrypted.clone(), 256);
                assert_eq!(
                    encrypted_as_field,
                    brillig_builder.insert_from_le_radix(encrypted.clone(), 256)
                );
                self.store_variable(&encrypted_as_field);
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
                    Type::Numeric(NumericType::U32),
                ) {
                    Some(input) => input,
                    _ => return,
                };
                let state = match self.insert_array(
                    acir_builder,
                    brillig_builder,
                    state_indices.to_vec(),
                    Type::Numeric(NumericType::U32),
                ) {
                    Some(state) => state,
                    _ => return,
                };
                let compressed =
                    acir_builder.insert_sha256_compression(input.clone(), state.clone());
                assert_eq!(compressed, brillig_builder.insert_sha256_compression(input, state));
                self.store_variable(&compressed);
                if load_elements_of_array {
                    for i in 0..8_u32 {
                        let index = acir_builder.insert_constant(i, NumericType::U32);
                        assert_eq!(index, brillig_builder.insert_constant(i, NumericType::U32));
                        let value = acir_builder.insert_array_get(
                            compressed.clone(),
                            index.clone(),
                            Type::Numeric(NumericType::U32),
                            /*safe_index =*/ false,
                        );
                        assert_eq!(
                            value,
                            brillig_builder.insert_array_get(
                                compressed.clone(),
                                index,
                                Type::Numeric(NumericType::U32),
                                /*safe_index =*/ false
                            )
                        );
                        self.store_variable(&value);
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
                    self.store_variable(typed_value);
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
                    self.store_variable(typed_value);
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
                    result,
                    brillig_builder.ecdsa_secp256r1(
                        prepared_signature.public_key_x,
                        prepared_signature.public_key_y,
                        prepared_signature.hash,
                        hash_size,
                        prepared_signature.signature,
                    )
                );
                self.store_variable(&result);
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
                    result,
                    brillig_builder.ecdsa_secp256k1(
                        prepared_signature.public_key_x,
                        prepared_signature.public_key_y,
                        prepared_signature.hash,
                        hash_size,
                        prepared_signature.signature,
                    )
                );
                self.store_variable(&result);
            }
        }
    }

    /// Takes scalar from [`super::instruction::Scalar`] and converts it to [`noir_ssa_fuzzer::typed_value::Scalar`]
    fn ssa_scalar_from_instruction_scalar(&mut self, scalar: InstructionScalar) -> Option<Scalar> {
        let lo = self.get_stored_variable(&Type::Numeric(NumericType::Field), scalar.field_lo_idx);
        let hi = self.get_stored_variable(&Type::Numeric(NumericType::Field), scalar.field_hi_idx);
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
        let is_infinite = acir_builder.insert_constant(point.is_infinite, NumericType::Boolean);
        assert_eq!(
            is_infinite,
            brillig_builder.insert_constant(point.is_infinite, NumericType::Boolean)
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
        element_type: Type,
    ) -> Option<TypedValue> {
        if !self.options.instruction_options.create_array_enabled {
            return None;
        }
        // if we storing references, take values from memory addresses, otherwise from stored variables
        let elements = elements_indices
            .iter()
            .map(|index| self.get_stored_variable(&element_type, *index))
            .collect::<Option<Vec<TypedValue>>>();
        let elements = match elements {
            Some(elements) => elements,
            _ => return None,
        };
        if elements.is_empty() {
            return None;
        }
        let array = acir_builder.insert_array(elements.clone());
        assert_eq!(array, brillig_builder.insert_array(elements));
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
        let arrays = self.get_stored_arrays();
        if arrays.is_empty() {
            return None;
        }
        let array = arrays.get(array_index % arrays.len());
        let array = match array {
            Some(array) => array,
            _ => return None,
        };
        // references are not supported for array get with dynamic index
        if array.type_of_variable.is_reference() && !index_is_constant {
            return None;
        }
        // cast the index to u32
        let index_casted = acir_builder.insert_cast(index.clone(), Type::Numeric(NumericType::U32));
        assert_eq!(
            index_casted,
            brillig_builder.insert_cast(index.clone(), Type::Numeric(NumericType::U32))
        );
        let value = acir_builder.insert_array_get(
            array.clone(),
            index_casted.clone(),
            array.type_of_variable.unwrap_array_element_type(),
            safe_index,
        );
        assert_eq!(
            value,
            brillig_builder.insert_array_get(
                array.clone(),
                index_casted,
                array.type_of_variable.unwrap_array_element_type(),
                safe_index
            )
        );
        Some((value, array.type_of_variable.is_reference()))
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
        let arrays = self.get_stored_arrays();
        if arrays.is_empty() {
            return None;
        }
        let array = arrays.get(array_index % arrays.len());
        let array = match array {
            Some(array) => array,
            _ => return None,
        };

        let is_array_of_references =
            array.type_of_variable.unwrap_array_element_type().is_reference();
        // get the array from the stored arrays
        // references are not supported for array set with dynamic index
        if is_array_of_references && !index_is_constant {
            return None;
        }
        let value = self
            .get_stored_variable(&array.type_of_variable.unwrap_array_element_type(), value_index);
        let value = match value {
            Some(value) => value,
            _ => return None,
        };
        let new_array =
            acir_builder.insert_array_set(array.clone(), index.clone(), value.clone(), safe_index);
        assert_eq!(
            new_array,
            brillig_builder.insert_array_set(array.clone(), index, value, safe_index)
        );
        Some((new_array, is_array_of_references))
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

    /// Finds values with the given type and index
    ///
    /// # Arguments
    ///
    /// * `type_` - Type of the value to find
    /// * `index` - Index of the value to find
    ///
    /// # Returns
    /// * TypedValue with the given type and index, if index is provided, otherwise the last value
    /// * If no value is found, we create it from predefined boolean
    ///
    /// # Examples
    ///
    /// If we want to proceed function call, e.g. f1(&mut a: Field, b: [Field; 3])
    /// We have to find values with type &mut Field and [Field; 3]
    /// If variables of such type are not defined, uh ohh.. we need to create it
    /// The only thing we know is that one boolean is defined
    fn find_values_with_type(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        type_: &Type,
        index: Option<usize>,
    ) -> TypedValue {
        log::debug!("Finding values with type: {type_:?}");
        let values_of_such_type = self.stored_variables.get(type_);
        if values_of_such_type.is_some() {
            if let Some(index) = index {
                let length = values_of_such_type.unwrap().len();
                return values_of_such_type.unwrap().get(index % length).cloned().unwrap();
            } else {
                return values_of_such_type.unwrap().last().cloned().unwrap();
            }
        }

        match type_ {
            // On numeric simple cast from boolean
            Type::Numeric(_) => {
                let boolean_value =
                    self.get_stored_variable(&Type::Numeric(NumericType::Boolean), 0).unwrap();
                let acir_value = acir_builder.insert_cast(boolean_value.clone(), type_.clone());
                let brillig_value = brillig_builder.insert_cast(boolean_value, type_.clone());
                assert_eq!(acir_value, brillig_value);
                self.store_variable(&acir_value);
                acir_value
            }
            // On reference, try to find value with reference type,
            // allocate and store it in memory
            Type::Reference(reference_type) => {
                let value = self.find_values_with_type(
                    acir_builder,
                    brillig_builder,
                    reference_type.as_ref(),
                    None,
                );
                let acir_value = acir_builder.insert_add_to_memory(value.clone());
                let brillig_value = brillig_builder.insert_add_to_memory(value);
                assert_eq!(acir_value, brillig_value);
                self.store_variable(&acir_value);
                acir_value
            }
            Type::Array(array_type, array_size) => {
                let mut values = Vec::with_capacity((*array_size as usize) * array_type.len());
                for _ in 0..*array_size {
                    let value = array_type.iter().map(|element_type| {
                        self.find_values_with_type(
                            acir_builder,
                            brillig_builder,
                            element_type,
                            None,
                        )
                    });
                    values.extend(value);
                }
                let acir_value = acir_builder.insert_array(values.clone());
                let brillig_value = brillig_builder.insert_array(values);
                assert_eq!(acir_value, brillig_value);
                self.store_variable(&acir_value);
                acir_value
            }
            Type::Slice(slice_type) => {
                let values = slice_type
                    .iter()
                    .map(|element_type| {
                        self.find_values_with_type(
                            acir_builder,
                            brillig_builder,
                            element_type,
                            None,
                        )
                    })
                    .collect::<Vec<TypedValue>>();
                let acir_value = acir_builder.insert_slice(values.clone());
                let brillig_value = brillig_builder.insert_slice(values);
                assert_eq!(acir_value, brillig_value);
                self.store_variable(&acir_value);
                acir_value
            }
        }
    }

    /// Finalizes the function by setting the return value
    pub(crate) fn finalize_block_with_return(
        &mut self,
        acir_builder: &mut FuzzerBuilder,
        brillig_builder: &mut FuzzerBuilder,
        return_type: Type,
    ) {
        let return_value =
            self.find_values_with_type(acir_builder, brillig_builder, &return_type, None);
        acir_builder.finalize_function(&return_value);
        brillig_builder.finalize_function(&return_value);
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
            .get(&Type::Numeric(NumericType::Boolean))
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
        let mut values = vec![];

        // if the length of args is less than the number of input types, we fill it with random values
        // don't really like this, but there is no other way to predict it on mutation level
        let mut args_to_use = args.to_vec();
        if args.len() < function_signature.input_types.len() {
            args_to_use.extend(vec![0; function_signature.input_types.len() - args.len()]);
        }
        for (value_type, index) in zip(function_signature.input_types, args_to_use) {
            let value =
                self.find_values_with_type(acir_builder, brillig_builder, &value_type, Some(index));
            values.push(value);
        }

        // Insert a call to the function with the given arguments and result type
        let ret_val = acir_builder.insert_call(
            func_as_value_id,
            &values,
            function_signature.return_type.clone(),
        );
        assert_eq!(
            ret_val,
            brillig_builder.insert_call(
                func_as_value_id,
                &values,
                function_signature.return_type.clone()
            )
        );
        let typed_ret_val = TypedValue {
            value_id: ret_val,
            type_of_variable: function_signature.return_type.clone(),
        };
        // Append the return value to stored_values map
        self.store_variable(&typed_ret_val);
    }
}
