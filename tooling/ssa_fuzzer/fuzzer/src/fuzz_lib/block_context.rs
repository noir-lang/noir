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
        builder: &mut FuzzerBuilder,
        arg: NumericArgument,
        instruction: InstructionWithOneArg,
    ) {
        let value = self.get_stored_variable(&Type::Numeric(arg.numeric_type), arg.index);
        let value = match value {
            Some(value) => value,
            _ => return,
        };
        let result = instruction(builder, value.clone());
        self.store_variable(&result);
    }

    /// Inserts an instruction that takes two arguments
    fn insert_instruction_with_double_args(
        &mut self,
        builder: &mut FuzzerBuilder,
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
        let result = instruction(builder, instr_lhs.clone(), instr_rhs.clone());
        self.store_variable(&result);
    }

    /// Inserts an instruction into both ACIR and Brillig programs
    fn insert_instruction(&mut self, builder: &mut FuzzerBuilder, instruction: Instruction) {
        match instruction {
            Instruction::AddChecked { lhs, rhs } => {
                if !self.options.instruction_options.add_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_add_instruction_checked(lhs, rhs)
                });
            }
            Instruction::SubChecked { lhs, rhs } => {
                if !self.options.instruction_options.sub_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_sub_instruction_checked(lhs, rhs)
                });
            }
            Instruction::MulChecked { lhs, rhs } => {
                if !self.options.instruction_options.mul_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_mul_instruction_checked(lhs, rhs)
                });
            }
            Instruction::Div { lhs, rhs } => {
                if !self.options.instruction_options.div_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_div_instruction(lhs, rhs)
                });
            }
            Instruction::Eq { lhs, rhs } => {
                if !self.options.instruction_options.eq_enabled {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_eq_instruction(lhs, rhs)
                });
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
                let result = builder.insert_cast(value.clone(), Type::Numeric(type_));
                self.store_variable(&result);
            }
            Instruction::Mod { lhs, rhs } => {
                if !self.options.instruction_options.mod_enabled {
                    return;
                }
                // optimization: mod is not supported for field type (see ssa_validation)
                if lhs.numeric_type == NumericType::Field {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_mod_instruction(lhs, rhs)
                });
            }
            Instruction::Not { lhs } => {
                if !self.options.instruction_options.not_enabled {
                    return;
                }
                if lhs.numeric_type == NumericType::Field {
                    return;
                }
                self.insert_instruction_with_single_arg(builder, lhs, |builder, lhs| {
                    builder.insert_not_instruction(lhs)
                });
            }
            Instruction::Shl { lhs, rhs } => {
                if !self.options.instruction_options.shl_enabled {
                    return;
                }
                // optimization: not is not supported for field type (see ssa_validation)
                if lhs.numeric_type == NumericType::Field {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_shl_instruction(lhs, rhs)
                });
            }
            Instruction::Shr { lhs, rhs } => {
                if !self.options.instruction_options.shr_enabled {
                    return;
                }
                // optimization: shl is not supported for field type (see ssa_validation)
                if lhs.numeric_type == NumericType::Field {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_shr_instruction(lhs, rhs)
                });
            }
            Instruction::And { lhs, rhs } => {
                if !self.options.instruction_options.and_enabled {
                    return;
                }
                // optimization: and is not supported for field type (see ssa_validation)
                if lhs.numeric_type == NumericType::Field {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_and_instruction(lhs, rhs)
                });
            }
            Instruction::Or { lhs, rhs } => {
                if !self.options.instruction_options.or_enabled {
                    return;
                }
                // optimization: or is not supported for field type (see ssa_validation)
                if lhs.numeric_type == NumericType::Field {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_or_instruction(lhs, rhs)
                });
            }
            Instruction::Xor { lhs, rhs } => {
                if !self.options.instruction_options.xor_enabled {
                    return;
                }
                // optimization: xor is not supported for field type (see ssa_validation)
                if lhs.numeric_type == NumericType::Field {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_xor_instruction(lhs, rhs)
                });
            }
            Instruction::Lt { lhs, rhs } => {
                if !self.options.instruction_options.lt_enabled {
                    return;
                }
                // optimization: lt is not supported for field type (see ssa_validation)
                if lhs.numeric_type == NumericType::Field {
                    return;
                }
                self.insert_instruction_with_double_args(builder, lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_lt_instruction(lhs, rhs)
                });
            }

            Instruction::AddSubConstrain { lhs, rhs } => {
                // inserts lhs' = lhs + rhs
                let lhs_orig = self.get_stored_variable(&Type::Numeric(NumericType::Field), lhs);
                let rhs = self.get_stored_variable(&Type::Numeric(NumericType::Field), rhs);
                let (lhs_orig, rhs) = match (lhs_orig, rhs) {
                    (Some(lhs_orig), Some(rhs)) => (lhs_orig, rhs),
                    _ => return,
                };
                let lhs_add_rhs =
                    builder.insert_add_instruction_checked(lhs_orig.clone(), rhs.clone());
                // inserts lhs'' = lhs' - rhs
                let lhs = lhs_add_rhs;
                let morphed = builder.insert_sub_instruction_checked(lhs.clone(), rhs.clone());

                if !self.options.constrain_idempotent_enabled {
                    return;
                }

                builder.insert_constrain(lhs_orig.clone(), morphed.clone());
            }
            Instruction::MulDivConstrain { lhs, rhs } => {
                let lhs_orig = self.get_stored_variable(&Type::Numeric(NumericType::Field), lhs);
                let rhs = self.get_stored_variable(&Type::Numeric(NumericType::Field), rhs);
                let (lhs_orig, rhs) = match (lhs_orig, rhs) {
                    (Some(lhs_orig), Some(rhs)) => (lhs_orig, rhs),
                    _ => return,
                };
                // inserts lhs' = lhs * rhs
                let lhs_mul_rhs =
                    builder.insert_mul_instruction_checked(lhs_orig.clone(), rhs.clone());
                // lhs'' = lhs' / rhs
                let lhs = lhs_mul_rhs;
                let morphed = builder.insert_div_instruction(lhs.clone(), rhs.clone());

                if !self.options.constrain_idempotent_enabled {
                    return;
                }
                builder.insert_constrain(lhs_orig.clone(), morphed.clone());
            }
            Instruction::AddToMemory { lhs } => {
                if !self.options.instruction_options.alloc_enabled {
                    return;
                }

                let value = match self.get_stored_variable(&lhs.value_type, lhs.index) {
                    Some(value) => value,
                    _ => return,
                };

                let addr = builder.insert_add_to_memory(value.clone());
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
                let value = builder.insert_load_from_memory(address.clone());
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
                    let addr = builder.insert_add_to_memory(value.clone());
                    self.store_variable(&addr);
                    addr
                } else {
                    addresses[memory_addr_index % addresses.len()].clone()
                };
                builder.insert_set_to_memory(address.clone(), value.clone());
            }

            Instruction::CreateArray { elements_indices, element_type } => {
                // insert to both acir and brillig builders
                let array = match self.insert_array(builder, elements_indices, element_type) {
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
                    builder,
                    array_index,
                    index,
                    /*is constant =*/ false,
                    safe_index,
                );
                if let Some(value) = value {
                    self.store_variable(&value);
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
                    builder.insert_cast(index.clone(), Type::Numeric(NumericType::U32));

                // insert array set to both acir and brillig builders
                let new_array = self.insert_array_set(
                    builder,
                    array_index,
                    index_casted,
                    /*index_is_constant =*/ false,
                    value_index,
                    safe_index,
                );
                if let Some(new_array) = new_array {
                    self.store_variable(&new_array);
                }
            }
            Instruction::ArrayGetWithConstantIndex { array_index, index, safe_index } => {
                // Array index should be limited to 32 bits
                // Otherwise it fails at compiler/noirc_evaluator/src/ssa/ir/dfg/simplify.rs:133:40:
                let index_32_bits = index & (u32::MAX as usize);
                // insert constant index
                let index_id = builder.insert_constant(index_32_bits, NumericType::U32);
                let value = self.insert_array_get(
                    builder,
                    array_index,
                    index_id,
                    /*is constant =*/ true,
                    safe_index,
                );
                if let Some(value) = value {
                    self.store_variable(&value);
                }
            }
            Instruction::ArraySetWithConstantIndex {
                array_index,
                index,
                value_index,
                safe_index,
            } => {
                // Array index should be limited to 32 bits
                // Otherwise it fails at compiler/noirc_evaluator/src/ssa/ir/dfg/simplify.rs:133:40:
                let index_32_bits = index & (u32::MAX as usize);
                // insert constant index
                let index_id = builder.insert_constant(index_32_bits, NumericType::U32);
                let new_array = self.insert_array_set(
                    builder,
                    array_index,
                    index_id,
                    /*index_is_constant =*/ true,
                    value_index,
                    safe_index,
                );
                if let Some(new_array) = new_array {
                    self.store_variable(&new_array);
                }
            }
            Instruction::FieldToBytesToField { field_idx } => {
                if !self.options.instruction_options.field_to_bytes_to_field_enabled {
                    return;
                }
                let field = self.get_stored_variable(&Type::Numeric(NumericType::Field), field_idx);
                let field = match field {
                    Some(field) => field,
                    _ => return,
                };
                let bytes = builder.insert_to_le_radix(field.clone(), 256, 32);
                let field = builder.insert_from_le_radix(bytes.clone(), 256);
                self.store_variable(&field);
            }
            Instruction::Blake2sHash { field_idx, limbs_count } => {
                if !self.options.instruction_options.blake2s_hash_enabled {
                    return;
                }
                let input = self.get_stored_variable(&Type::Numeric(NumericType::Field), field_idx);
                let input = match input {
                    Some(input) => input,
                    _ => return,
                };
                if limbs_count == 0 {
                    return;
                }
                let bytes = builder.insert_to_le_radix(input.clone(), 256, limbs_count);
                let hash = builder.insert_blake2s_hash(bytes.clone());
                let hash_as_field = builder.insert_from_le_radix(hash.clone(), 256);
                self.store_variable(&hash_as_field);
            }
            Instruction::Blake3Hash { field_idx, limbs_count } => {
                if !self.options.instruction_options.blake3_hash_enabled {
                    return;
                }
                let input = self.get_stored_variable(&Type::Numeric(NumericType::Field), field_idx);
                let input = match input {
                    Some(input) => input,
                    _ => return,
                };
                if limbs_count == 0 {
                    return;
                }
                let bytes = builder.insert_to_le_radix(input.clone(), 256, limbs_count);
                let hash = builder.insert_blake3_hash(bytes.clone());
                let hash_as_field = builder.insert_from_le_radix(hash.clone(), 256);
                self.store_variable(&hash_as_field);
            }
            Instruction::Keccakf1600Hash { u64_indices, load_elements_of_array } => {
                if !self.options.instruction_options.keccakf1600_hash_enabled {
                    return;
                }
                let input = match self.insert_array(
                    builder,
                    u64_indices.to_vec(),
                    Type::Numeric(NumericType::U64),
                ) {
                    Some(input) => input,
                    _ => return,
                };
                let hash_array_u64 = builder.insert_keccakf1600_permutation(input.clone());
                self.store_variable(&hash_array_u64);
                if load_elements_of_array {
                    for i in 0..25_u32 {
                        let index = builder.insert_constant(i, NumericType::U32);
                        let value = builder.insert_array_get(
                            hash_array_u64.clone(),
                            index.clone(),
                            Type::Numeric(NumericType::U64),
                            /*safe_index =*/ false,
                        );
                        self.store_variable(&value);
                    }
                }
            }
            Instruction::Aes128Encrypt { input_idx, input_limbs_count, key_idx, iv_idx } => {
                if !self.options.instruction_options.aes128_encrypt_enabled {
                    return;
                }
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
                let input_bytes = builder.insert_to_le_radix(input.clone(), 256, input_limbs_count);
                let key_bytes = builder.insert_to_le_radix(key.clone(), 256, 16);
                let iv_bytes = builder.insert_to_le_radix(iv.clone(), 256, 16);
                let encrypted = builder.insert_aes128_encrypt(
                    input_bytes.clone(),
                    key_bytes.clone(),
                    iv_bytes.clone(),
                );
                let encrypted_as_field = builder.insert_from_le_radix(encrypted.clone(), 256);
                self.store_variable(&encrypted_as_field);
            }
            Instruction::Sha256Compression {
                input_indices,
                state_indices,
                load_elements_of_array,
            } => {
                if !self.options.instruction_options.sha256_compression_enabled {
                    return;
                }
                let input = match self.insert_array(
                    builder,
                    input_indices.to_vec(),
                    Type::Numeric(NumericType::U32),
                ) {
                    Some(input) => input,
                    _ => return,
                };
                let state = match self.insert_array(
                    builder,
                    state_indices.to_vec(),
                    Type::Numeric(NumericType::U32),
                ) {
                    Some(state) => state,
                    _ => return,
                };
                let compressed = builder.insert_sha256_compression(input.clone(), state.clone());
                self.store_variable(&compressed);
                if load_elements_of_array {
                    for i in 0..8_u32 {
                        let index = builder.insert_constant(i, NumericType::U32);
                        let value = builder.insert_array_get(
                            compressed.clone(),
                            index.clone(),
                            Type::Numeric(NumericType::U32),
                            /*safe_index =*/ false,
                        );
                        self.store_variable(&value);
                    }
                }
            }
            Instruction::PointAdd { p1, p2, predicate } => {
                if !self.options.instruction_options.point_add_enabled {
                    return;
                }
                let p1 = self.ssa_point_from_instruction_point(builder, p1);
                let p2 = self.ssa_point_from_instruction_point(builder, p2);
                if p1.is_none() || p2.is_none() {
                    return;
                }
                let p1 = p1.unwrap();
                let p2 = p2.unwrap();
                let acir_point = builder.point_add(p1.clone(), p2.clone(), predicate);
                for typed_value in [&acir_point.x, &acir_point.y, &acir_point.is_infinite] {
                    self.store_variable(typed_value);
                }
            }
            Instruction::MultiScalarMul { points_and_scalars, predicate } => {
                if !self.options.instruction_options.multi_scalar_mul_enabled {
                    return;
                }
                let mut points_vec = Vec::new();
                let mut scalars_vec = Vec::new();
                for (p, s) in points_and_scalars.iter() {
                    let point = self.ssa_point_from_instruction_point(builder, *p);
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
                let point =
                    builder.multi_scalar_mul(points_vec.clone(), scalars_vec.clone(), predicate);
                for typed_value in [&point.x, &point.y, &point.is_infinite] {
                    self.store_variable(typed_value);
                }
            }
            Instruction::EcdsaSecp256r1 {
                msg,
                corrupt_hash,
                corrupt_pubkey_x,
                corrupt_pubkey_y,
                corrupt_signature,
                predicate,
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
                let result = builder.ecdsa_secp256r1(
                    prepared_signature.public_key_x.clone(),
                    prepared_signature.public_key_y.clone(),
                    prepared_signature.hash.clone(),
                    prepared_signature.signature.clone(),
                    predicate,
                );
                self.store_variable(&result);
            }
            Instruction::EcdsaSecp256k1 {
                msg,
                corrupt_hash,
                corrupt_pubkey_x,
                corrupt_pubkey_y,
                corrupt_signature,
                predicate,
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
                let result = builder.ecdsa_secp256k1(
                    prepared_signature.public_key_x.clone(),
                    prepared_signature.public_key_y.clone(),
                    prepared_signature.hash.clone(),
                    prepared_signature.signature.clone(),
                    predicate,
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
        builder: &mut FuzzerBuilder,
        point: InstructionPoint,
    ) -> Option<Point> {
        let scalar = self.ssa_scalar_from_instruction_scalar(point.scalar);
        scalar.as_ref()?; // wtf clippy forbid me to write if scalar.is_none() {return None}
        let scalar = scalar.unwrap();
        let is_infinite = builder.insert_constant(point.is_infinite, NumericType::Boolean);

        let point = if point.derive_from_scalar_mul {
            builder.base_scalar_mul(scalar.clone(), is_infinite.clone())
        } else {
            builder.create_point_from_scalar(scalar.clone(), is_infinite.clone())
        };
        Some(point)
    }

    fn insert_array(
        &mut self,
        builder: &mut FuzzerBuilder,
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
        let array = builder.insert_array(elements.clone());
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
    /// * TypedValue
    /// * None if the instruction is not enabled or the array is not stored
    fn insert_array_get(
        &mut self,
        builder: &mut FuzzerBuilder,
        array_index: usize,
        index: TypedValue,
        index_is_constant: bool,
        safe_index: bool,
    ) -> Option<TypedValue> {
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
        if array.type_of_variable.type_contains_reference() && !index_is_constant {
            return None;
        }
        // cast the index to u32
        let index_casted = builder.insert_cast(index.clone(), Type::Numeric(NumericType::U32));
        let value = builder.insert_array_get(
            array.clone(),
            index_casted.clone(),
            array.type_of_variable.unwrap_array_element_type(),
            safe_index,
        );
        Some(value)
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
    /// * TypedValue referencing the new array
    /// * None if the instruction is not enabled or the array is not stored
    fn insert_array_set(
        &mut self,
        builder: &mut FuzzerBuilder,
        array_index: usize,
        index: TypedValue,
        index_is_constant: bool,
        value_index: usize,
        safe_index: bool,
    ) -> Option<TypedValue> {
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
            array.type_of_variable.unwrap_array_element_type().type_contains_reference();
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
            builder.insert_array_set(array.clone(), index.clone(), value.clone(), safe_index);
        Some(new_array)
    }

    pub(crate) fn insert_instructions(
        &mut self,
        builder: &mut FuzzerBuilder,
        instructions: &Vec<Instruction>,
    ) {
        for instruction in instructions {
            self.insert_instruction(builder, instruction.clone());
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
        builder: &mut FuzzerBuilder,
        type_: &Type,
        index: Option<usize>,
    ) -> TypedValue {
        log::debug!("Finding values with type: {type_:?}");
        if let Some(values_of_such_type) = self.stored_variables.get(type_) {
            if let Some(index) = index {
                let length = values_of_such_type.len();
                return values_of_such_type.get(index % length).cloned().unwrap();
            } else {
                return values_of_such_type.last().cloned().unwrap();
            }
        }

        match type_ {
            // On numeric simple cast from boolean
            Type::Numeric(_) => {
                let boolean_value =
                    self.get_stored_variable(&Type::Numeric(NumericType::Boolean), 0).unwrap();
                let acir_value = builder.insert_cast(boolean_value.clone(), type_.clone());
                self.store_variable(&acir_value);
                acir_value
            }
            // On reference, try to find value with reference type,
            // allocate and store it in memory
            Type::Reference(reference_type) => {
                let value = self.find_values_with_type(builder, reference_type.as_ref(), None);
                let value = builder.insert_add_to_memory(value.clone());
                self.store_variable(&value);
                value
            }
            Type::Array(array_type, array_size) => {
                let mut values = Vec::with_capacity((*array_size as usize) * array_type.len());
                for _ in 0..*array_size {
                    let value = array_type.iter().map(|element_type| {
                        self.find_values_with_type(builder, element_type, None)
                    });
                    values.extend(value);
                }
                let value = builder.insert_array(values.clone());
                self.store_variable(&value);
                value
            }
            Type::Vector(vector_type) => {
                let values = vector_type
                    .iter()
                    .map(|element_type| self.find_values_with_type(builder, element_type, None))
                    .collect::<Vec<TypedValue>>();
                let value = builder.insert_vector(values.clone());
                self.store_variable(&value);
                value
            }
        }
    }

    /// Finalizes the function by setting the return value
    pub(crate) fn finalize_block_with_return(
        &mut self,
        builder: &mut FuzzerBuilder,
        return_type: Type,
    ) {
        let return_value = self.find_values_with_type(builder, &return_type, None);
        builder.finalize_function(&return_value);
    }

    pub(crate) fn finalize_block_with_jmp(
        &mut self,
        builder: &mut FuzzerBuilder,
        jmp_destination: BasicBlockId,
        args: Vec<TypedValue>,
    ) {
        builder.insert_jmp_instruction(jmp_destination, args.clone());
        self.children_blocks.push(jmp_destination);
    }

    pub(crate) fn finalize_block_with_jmp_if(
        &mut self,
        builder: &mut FuzzerBuilder,
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

        builder.insert_jmpif_instruction(condition, then_destination, else_destination);
        self.children_blocks.push(then_destination);
        self.children_blocks.push(else_destination);
    }

    /// Inserts a function call to the given function with the given arguments and result type
    pub(crate) fn process_function_call(
        &mut self,
        builder: &mut FuzzerBuilder,
        function_id: Id<Function>,
        function_signature: FunctionInfo,
        args: &[usize],
    ) {
        // On SSA level you cannot just call a function by its id, you need to import it first
        let func_as_value_id = builder.insert_import(function_id);

        // Get values from stored_values map by indices
        let mut values = vec![];

        // if the length of args is less than the number of input types, we fill it with random values
        // don't really like this, but there is no other way to predict it on mutation level
        let mut args_to_use = args.to_vec();
        if args.len() < function_signature.input_types.len() {
            args_to_use.extend(vec![0; function_signature.input_types.len() - args.len()]);
        }
        for (value_type, index) in zip(function_signature.input_types, args_to_use) {
            let value = self.find_values_with_type(builder, &value_type, Some(index));
            values.push(value);
        }

        // Insert a call to the function with the given arguments and result type
        let ret_val =
            builder.insert_call(func_as_value_id, &values, function_signature.return_type.clone());
        let typed_ret_val = TypedValue {
            value_id: ret_val,
            type_of_variable: function_signature.return_type.clone(),
        };
        // Append the return value to stored_values map
        self.store_variable(&typed_ret_val);
    }
}
