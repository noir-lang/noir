//! Implementations for VM native [black box functions][acir::brillig::Opcode::BlackBox].
use acir::brillig::{BlackBoxOp, HeapArray};
use acir::{AcirField, BlackBoxFunc};
use acvm_blackbox_solver::{
    BlackBoxFunctionSolver, BlackBoxResolutionError, aes128_encrypt, blake2s, blake3,
    ecdsa_secp256k1_verify, ecdsa_secp256r1_verify, keccakf1600, sha256_compression,
};
use num_bigint::BigUint;
use num_traits::Zero;

use crate::Memory;
use crate::memory::MemoryValue;

/// Reads a fixed-size [array][HeapArray] from memory.
///
/// The data is not expected to contain pointers to nested arrays or vector.
fn read_heap_array<'a, F: AcirField>(
    memory: &'a Memory<F>,
    array: &HeapArray,
) -> &'a [MemoryValue<F>] {
    let items_start = memory.read_ref(array.pointer);
    memory.read_slice(items_start, array.size)
}

/// Write values to a [array][HeapArray] in memory.
fn write_heap_array<F: AcirField>(
    memory: &mut Memory<F>,
    array: &HeapArray,
    values: &[MemoryValue<F>],
) {
    let items_start = memory.read_ref(array.pointer);
    memory.write_slice(items_start, values);
}

/// Extracts the last byte of every value
fn to_u8_vec<F: AcirField>(inputs: &[MemoryValue<F>]) -> Vec<u8> {
    let mut result = Vec::with_capacity(inputs.len());
    for &input in inputs {
        result.push(input.expect_u8().unwrap());
    }
    result
}

/// Converts a slice of u8 values into a Vec<[`MemoryValue<F>`]>,
/// wrapping each byte as a [MemoryValue::U8].
fn to_value_vec<F: AcirField>(input: &[u8]) -> Vec<MemoryValue<F>> {
    input.iter().map(|&x| x.into()).collect()
}

/// Evaluates a black box function inside the VM, performing the actual native computation.
///
/// Delegates the execution to the corresponding cryptographic or arithmetic
/// function, depending on the [BlackBoxOp] variant.
/// Handles input conversion, writing the result to memory, and error propagation.
///
/// # Arguments
/// - op: The black box operation to evaluate.
/// - solver: An implementation of [BlackBoxFunctionSolver] providing external function behavior.
/// - memory: The VM memory from which inputs are read and to which results are written.
/// - bigint_solver: A solver used for big integer operations.
///
/// # Returns
/// - Ok(()) if evaluation succeeds.
/// - Err([BlackBoxResolutionError]) if an error occurs during execution or input is invalid.
///
/// # Panics
/// If any required memory value cannot be converted to the expected type (e.g., [expect_u8][MemoryValue::expect_u8])
/// or if the [radix decomposition][BlackBoxOp::ToRadix] constraints are violated internally, such as an invalid radix range (e.g., radix of 1).
pub(crate) fn evaluate_black_box<F: AcirField, Solver: BlackBoxFunctionSolver<F>>(
    op: &BlackBoxOp,
    solver: &Solver,
    memory: &mut Memory<F>,
) -> Result<(), BlackBoxResolutionError> {
    match op {
        BlackBoxOp::AES128Encrypt { inputs, iv, key, outputs } => {
            let bb_func = black_box_function_from_op(op);

            let inputs = to_u8_vec(read_heap_array(memory, inputs));

            let iv: [u8; 16] = to_u8_vec(read_heap_array(memory, iv)).try_into().map_err(|_| {
                BlackBoxResolutionError::Failed(bb_func, "Invalid iv length".to_string())
            })?;
            let key: [u8; 16] =
                to_u8_vec(read_heap_array(memory, key)).try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(bb_func, "Invalid key length".to_string())
                })?;
            let ciphertext = aes128_encrypt(&inputs, iv, key)?;

            memory.write_slice(memory.read_ref(outputs.pointer), &to_value_vec(&ciphertext));

            Ok(())
        }
        BlackBoxOp::Blake2s { message, output } => {
            let message = to_u8_vec(read_heap_array(memory, message));
            let bytes = blake2s(message.as_slice())?;
            write_heap_array(memory, output, &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Blake3 { message, output } => {
            let message = to_u8_vec(read_heap_array(memory, message));
            let bytes = blake3(message.as_slice())?;
            write_heap_array(memory, output, &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Keccakf1600 { input, output } => {
            let state_vec: Vec<u64> = read_heap_array(memory, input)
                .iter()
                .map(|&memory_value| memory_value.expect_u64().unwrap())
                .collect();
            let state: [u64; 25] = state_vec.try_into().unwrap();

            let new_state = keccakf1600(state)?;

            let new_state: Vec<MemoryValue<F>> = new_state.into_iter().map(|x| x.into()).collect();
            write_heap_array(memory, output, &new_state);
            Ok(())
        }
        BlackBoxOp::EcdsaSecp256k1 {
            hashed_msg,
            public_key_x,
            public_key_y,
            signature,
            result: result_address,
        }
        | BlackBoxOp::EcdsaSecp256r1 {
            hashed_msg,
            public_key_x,
            public_key_y,
            signature,
            result: result_address,
        } => {
            let bb_func = black_box_function_from_op(op);

            let public_key_x: [u8; 32] =
                to_u8_vec(read_heap_array(memory, public_key_x)).try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(
                        bb_func,
                        "Invalid public key x length".to_string(),
                    )
                })?;
            let public_key_y: [u8; 32] =
                to_u8_vec(read_heap_array(memory, public_key_y)).try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(
                        bb_func,
                        "Invalid public key y length".to_string(),
                    )
                })?;
            let signature: [u8; 64] =
                to_u8_vec(read_heap_array(memory, signature)).try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(bb_func, "Invalid signature length".to_string())
                })?;

            let hashed_msg = to_u8_vec(read_heap_array(memory, hashed_msg));

            let result = match op {
                BlackBoxOp::EcdsaSecp256k1 { .. } => ecdsa_secp256k1_verify(
                    &hashed_msg.try_into().unwrap(),
                    &public_key_x,
                    &public_key_y,
                    &signature,
                )?,
                BlackBoxOp::EcdsaSecp256r1 { .. } => ecdsa_secp256r1_verify(
                    &hashed_msg.try_into().unwrap(),
                    &public_key_x,
                    &public_key_y,
                    &signature,
                )?,
                _ => unreachable!("`BlackBoxOp` is guarded against being a non-ecdsa operation"),
            };

            memory.write(*result_address, result.into());
            Ok(())
        }
        BlackBoxOp::MultiScalarMul { points, scalars, outputs: result } => {
            let points: Vec<F> = read_heap_array(memory, points)
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    if i % 3 == 2 {
                        let is_infinite: bool = x.expect_u1().unwrap();
                        F::from(is_infinite)
                    } else {
                        x.expect_field().unwrap()
                    }
                })
                .collect();
            let scalars: Vec<F> = read_heap_array(memory, scalars)
                .iter()
                .map(|x| x.expect_field().unwrap())
                .collect();
            let mut scalars_lo = Vec::with_capacity(scalars.len() / 2);
            let mut scalars_hi = Vec::with_capacity(scalars.len() / 2);
            for (i, scalar) in scalars.iter().enumerate() {
                if i % 2 == 0 {
                    scalars_lo.push(*scalar);
                } else {
                    scalars_hi.push(*scalar);
                }
            }
            let (x, y, is_infinite) = solver.multi_scalar_mul(
                &points,
                &scalars_lo,
                &scalars_hi,
                true, // Predicate is always true as brillig has control flow to handle false case
            )?;
            write_heap_array(
                memory,
                result,
                &[
                    MemoryValue::new_field(x),
                    MemoryValue::new_field(y),
                    MemoryValue::U1(is_infinite != F::zero()),
                ],
            );
            Ok(())
        }
        BlackBoxOp::EmbeddedCurveAdd {
            input1_x,
            input1_y,
            input2_x,
            input2_y,
            result,
            input1_infinite,
            input2_infinite,
        } => {
            let input1_x = memory.read(*input1_x).expect_field().unwrap();
            let input1_y = memory.read(*input1_y).expect_field().unwrap();
            let input1_infinite: bool = memory.read(*input1_infinite).expect_u1().unwrap();
            let input2_x = memory.read(*input2_x).expect_field().unwrap();
            let input2_y = memory.read(*input2_y).expect_field().unwrap();
            let input2_infinite: bool = memory.read(*input2_infinite).expect_u1().unwrap();
            let (x, y, infinite) = solver.ec_add(
                &input1_x,
                &input1_y,
                &input1_infinite.into(),
                &input2_x,
                &input2_y,
                &input2_infinite.into(),
                true, // Predicate is always true as brillig has control flow to handle false case
            )?;

            write_heap_array(
                memory,
                result,
                &[
                    MemoryValue::new_field(x),
                    MemoryValue::new_field(y),
                    MemoryValue::U1(infinite != F::zero()),
                ],
            );
            Ok(())
        }
        BlackBoxOp::Poseidon2Permutation { message, output } => {
            let input = read_heap_array(memory, message);
            let input: Vec<F> = input.iter().map(|x| x.expect_field().unwrap()).collect();
            let result = solver.poseidon2_permutation(&input)?;
            let mut values = Vec::new();
            for i in result {
                values.push(MemoryValue::new_field(i));
            }
            write_heap_array(memory, output, &values);
            Ok(())
        }
        BlackBoxOp::Sha256Compression { input, hash_values, output } => {
            let mut message = [0; 16];
            let inputs = read_heap_array(memory, input);
            if inputs.len() != 16 {
                return Err(BlackBoxResolutionError::Failed(
                    BlackBoxFunc::Sha256Compression,
                    format!("Expected 16 inputs but encountered {}", &inputs.len()),
                ));
            }
            for (i, &input) in inputs.iter().enumerate() {
                message[i] = input.expect_u32().unwrap();
            }
            let mut state = [0; 8];
            let values = read_heap_array(memory, hash_values);
            if values.len() != 8 {
                return Err(BlackBoxResolutionError::Failed(
                    BlackBoxFunc::Sha256Compression,
                    format!("Expected 8 values but encountered {}", &values.len()),
                ));
            }
            for (i, &value) in values.iter().enumerate() {
                state[i] = value.expect_u32().unwrap();
            }

            sha256_compression(&mut state, &message);
            let state = state.map(|x| x.into());

            write_heap_array(memory, output, &state);
            Ok(())
        }
        BlackBoxOp::ToRadix { input, radix, output_pointer, num_limbs, output_bits } => {
            let input: F = memory.read(*input).expect_field().expect("ToRadix input not a field");
            let MemoryValue::U32(radix) = memory.read(*radix) else {
                panic!("ToRadix opcode's radix bit size does not match expected bit size 32")
            };
            let num_limbs = memory.read(*num_limbs).to_usize();
            let MemoryValue::U1(output_bits) = memory.read(*output_bits) else {
                panic!("ToRadix opcode's output_bits size does not match expected bit size 1")
            };

            let output = to_be_radix(input, radix, num_limbs, output_bits)?;

            memory.write_slice(memory.read_ref(*output_pointer), &output);

            Ok(())
        }
    }
}

/// Maps a [BlackBoxOp] variant to its corresponding [BlackBoxFunc].
/// Used primarily for error reporting and resolution purposes.
///
/// # Panics
/// If called with a [BlackBoxOp::ToRadix] operation, which is not part of the [BlackBoxFunc] enum.
fn black_box_function_from_op(op: &BlackBoxOp) -> BlackBoxFunc {
    match op {
        BlackBoxOp::AES128Encrypt { .. } => BlackBoxFunc::AES128Encrypt,
        BlackBoxOp::Blake2s { .. } => BlackBoxFunc::Blake2s,
        BlackBoxOp::Blake3 { .. } => BlackBoxFunc::Blake3,
        BlackBoxOp::Keccakf1600 { .. } => BlackBoxFunc::Keccakf1600,
        BlackBoxOp::EcdsaSecp256k1 { .. } => BlackBoxFunc::EcdsaSecp256k1,
        BlackBoxOp::EcdsaSecp256r1 { .. } => BlackBoxFunc::EcdsaSecp256r1,
        BlackBoxOp::MultiScalarMul { .. } => BlackBoxFunc::MultiScalarMul,
        BlackBoxOp::EmbeddedCurveAdd { .. } => BlackBoxFunc::EmbeddedCurveAdd,
        BlackBoxOp::Poseidon2Permutation { .. } => BlackBoxFunc::Poseidon2Permutation,
        BlackBoxOp::Sha256Compression { .. } => BlackBoxFunc::Sha256Compression,
        BlackBoxOp::ToRadix { .. } => unreachable!("ToRadix is not an ACIR BlackBoxFunc"),
    }
}

fn to_be_radix<F: AcirField>(
    input: F,
    radix: u32,
    num_limbs: usize,
    output_bits: bool,
) -> Result<Vec<MemoryValue<F>>, BlackBoxResolutionError> {
    assert!(
        (2u32..=256u32).contains(&radix),
        "Radix out of the valid range [2,256]. Value: {radix}"
    );

    assert!(
        num_limbs >= 1 || input.is_zero(),
        "Input value {input} is not zero but number of limbs is zero."
    );

    assert!(
        !output_bits || radix == 2u32,
        "Radix {radix} is not equal to 2 and bit mode is activated."
    );

    let mut input = BigUint::from_bytes_be(&input.to_be_bytes());
    let radix = BigUint::from(radix);

    let mut limbs: Vec<MemoryValue<F>> = vec![MemoryValue::default(); num_limbs];
    for i in (0..num_limbs).rev() {
        let limb = &input % &radix;
        limbs[i] = if output_bits {
            MemoryValue::U1(!limb.is_zero())
        } else {
            let limb: u8 = limb.try_into().unwrap();
            MemoryValue::U8(limb)
        };
        input /= &radix;
    }

    // In order for a successful decomposition, we require that after `num_limbs` divisions by `radix` then `input` should be zero.
    // If `input` is non-zero then that implies that we have additional limbs which are not handled.
    if !input.is_zero() {
        return Err(BlackBoxResolutionError::AssertFailed(format!(
            "Field failed to decompose into specified {num_limbs} limbs"
        )));
    }

    Ok(limbs)
}

#[cfg(test)]
mod to_be_radix_tests {
    use crate::black_box::to_be_radix;

    use acir::{AcirField, FieldElement};

    use proptest::prelude::*;

    // Define a wrapper around field so we can implement `Arbitrary`.
    // NB there are other methods like `arbitrary_field_elements` around the codebase,
    // but for `proptest_derive::Arbitrary` we need `F: AcirField + Arbitrary`.
    acir::acir_field::field_wrapper!(TestField, FieldElement);

    impl Arbitrary for TestField {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any::<u128>().prop_map(|v| Self(FieldElement::from(v))).boxed()
        }
    }

    proptest! {
        #[test]
        fn matches_byte_decomposition(param: TestField) {
            let bytes: Vec<u8> = to_be_radix(param.0, 256, 32, false).unwrap().into_iter().map(|byte| byte.expect_u8().unwrap()).collect();
            let expected_bytes = param.0.to_be_bytes();
            prop_assert_eq!(bytes, expected_bytes);
        }
    }

    #[test]
    fn correctly_handles_unusual_radices() {
        let value = FieldElement::from(65024u128);
        let expected_limbs = vec![254, 254];

        let limbs: Vec<u8> = to_be_radix(value, 255, 2, false)
            .unwrap()
            .into_iter()
            .map(|byte| byte.expect_u8().unwrap())
            .collect();
        assert_eq!(limbs, expected_limbs);
    }

    #[test]
    fn matches_decimal_decomposition() {
        let value = FieldElement::from(123456789u128);
        let expected_limbs = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let limbs: Vec<u8> = to_be_radix(value, 10, 9, false)
            .unwrap()
            .into_iter()
            .map(|byte| byte.expect_u8().unwrap())
            .collect();
        assert_eq!(limbs, expected_limbs);
    }
}
