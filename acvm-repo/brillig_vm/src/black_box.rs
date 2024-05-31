use acir::brillig::{BlackBoxOp, HeapArray, HeapVector};
use acir::{AcirField, BlackBoxFunc};
use acvm_blackbox_solver::BigIntSolver;
use acvm_blackbox_solver::{
    aes128_encrypt, blake2s, blake3, ecdsa_secp256k1_verify, ecdsa_secp256r1_verify, keccak256,
    keccakf1600, sha256, sha256compression, BlackBoxFunctionSolver, BlackBoxResolutionError,
};
use num_bigint::BigUint;

use crate::memory::MemoryValue;
use crate::Memory;

fn read_heap_vector<'a, F: AcirField>(
    memory: &'a Memory<F>,
    vector: &HeapVector,
) -> &'a [MemoryValue<F>] {
    let size = memory.read(vector.size);
    memory.read_slice(memory.read_ref(vector.pointer), size.to_usize())
}

fn read_heap_array<'a, F: AcirField>(
    memory: &'a Memory<F>,
    array: &HeapArray,
) -> &'a [MemoryValue<F>] {
    memory.read_slice(memory.read_ref(array.pointer), array.size)
}

/// Extracts the last byte of every value
fn to_u8_vec<F: AcirField>(inputs: &[MemoryValue<F>]) -> Vec<u8> {
    let mut result = Vec::with_capacity(inputs.len());
    for input in inputs {
        result.push(input.try_into().unwrap());
    }
    result
}

fn to_value_vec<F: AcirField>(input: &[u8]) -> Vec<MemoryValue<F>> {
    input.iter().map(|&x| x.into()).collect()
}

pub(crate) fn evaluate_black_box<F: AcirField, Solver: BlackBoxFunctionSolver<F>>(
    op: &BlackBoxOp,
    solver: &Solver,
    memory: &mut Memory<F>,
    bigint_solver: &mut BigIntSolver,
) -> Result<(), BlackBoxResolutionError> {
    match op {
        BlackBoxOp::AES128Encrypt { inputs, iv, key, outputs } => {
            let bb_func = black_box_function_from_op(op);

            let inputs = to_u8_vec(read_heap_vector(memory, inputs));

            let iv: [u8; 16] = to_u8_vec(read_heap_array(memory, iv)).try_into().map_err(|_| {
                BlackBoxResolutionError::Failed(bb_func, "Invalid iv length".to_string())
            })?;
            let key: [u8; 16] =
                to_u8_vec(read_heap_array(memory, key)).try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(bb_func, "Invalid ley length".to_string())
                })?;
            let ciphertext = aes128_encrypt(&inputs, iv, key)?;

            memory.write(outputs.size, ciphertext.len().into());
            memory.write_slice(memory.read_ref(outputs.pointer), &to_value_vec(&ciphertext));

            Ok(())
        }
        BlackBoxOp::Sha256 { message, output } => {
            let message = to_u8_vec(read_heap_vector(memory, message));
            let bytes = sha256(message.as_slice())?;
            memory.write_slice(memory.read_ref(output.pointer), &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Blake2s { message, output } => {
            let message = to_u8_vec(read_heap_vector(memory, message));
            let bytes = blake2s(message.as_slice())?;
            memory.write_slice(memory.read_ref(output.pointer), &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Blake3 { message, output } => {
            let message = to_u8_vec(read_heap_vector(memory, message));
            let bytes = blake3(message.as_slice())?;
            memory.write_slice(memory.read_ref(output.pointer), &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Keccak256 { message, output } => {
            let message = to_u8_vec(read_heap_vector(memory, message));
            let bytes = keccak256(message.as_slice())?;
            memory.write_slice(memory.read_ref(output.pointer), &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Keccakf1600 { message, output } => {
            let state_vec: Vec<u64> = read_heap_vector(memory, message)
                .iter()
                .map(|memory_value| memory_value.try_into().unwrap())
                .collect();
            let state: [u64; 25] = state_vec.try_into().unwrap();

            let new_state = keccakf1600(state)?;

            let new_state: Vec<MemoryValue<F>> = new_state.into_iter().map(|x| x.into()).collect();
            memory.write_slice(memory.read_ref(output.pointer), &new_state);
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

            let hashed_msg = to_u8_vec(read_heap_vector(memory, hashed_msg));

            let result = match op {
                BlackBoxOp::EcdsaSecp256k1 { .. } => {
                    ecdsa_secp256k1_verify(&hashed_msg, &public_key_x, &public_key_y, &signature)?
                }
                BlackBoxOp::EcdsaSecp256r1 { .. } => {
                    ecdsa_secp256r1_verify(&hashed_msg, &public_key_x, &public_key_y, &signature)?
                }
                _ => unreachable!("`BlackBoxOp` is guarded against being a non-ecdsa operation"),
            };

            memory.write(*result_address, result.into());
            Ok(())
        }
        BlackBoxOp::SchnorrVerify { public_key_x, public_key_y, message, signature, result } => {
            let public_key_x = *memory.read(*public_key_x).extract_field().unwrap();
            let public_key_y = *memory.read(*public_key_y).extract_field().unwrap();
            let message: Vec<u8> = to_u8_vec(read_heap_vector(memory, message));
            let signature: [u8; 64] =
                to_u8_vec(read_heap_vector(memory, signature)).try_into().unwrap();
            let verified =
                solver.schnorr_verify(&public_key_x, &public_key_y, &signature, &message)?;
            memory.write(*result, verified.into());
            Ok(())
        }
        BlackBoxOp::MultiScalarMul { points, scalars, outputs: result } => {
            let points: Vec<F> = read_heap_vector(memory, points)
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    if i % 3 == 2 {
                        let is_infinite: bool = x.try_into().unwrap();
                        F::from(is_infinite as u128)
                    } else {
                        *x.extract_field().unwrap()
                    }
                })
                .collect();
            let scalars: Vec<F> = read_heap_vector(memory, scalars)
                .iter()
                .map(|x| *x.extract_field().unwrap())
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
            let (x, y, is_infinite) = solver.multi_scalar_mul(&points, &scalars_lo, &scalars_hi)?;
            memory.write_slice(
                memory.read_ref(result.pointer),
                &[
                    MemoryValue::new_field(x),
                    MemoryValue::new_field(y),
                    MemoryValue::new_field(is_infinite),
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
            let input1_x = *memory.read(*input1_x).extract_field().unwrap();
            let input1_y = *memory.read(*input1_y).extract_field().unwrap();
            let input1_infinite: bool = memory.read(*input1_infinite).try_into().unwrap();
            let input2_x = *memory.read(*input2_x).extract_field().unwrap();
            let input2_y = *memory.read(*input2_y).extract_field().unwrap();
            let input2_infinite: bool = memory.read(*input2_infinite).try_into().unwrap();
            let (x, y, infinite) = solver.ec_add(
                &input1_x,
                &input1_y,
                &input1_infinite.into(),
                &input2_x,
                &input2_y,
                &input2_infinite.into(),
            )?;
            memory.write_slice(
                memory.read_ref(result.pointer),
                &[
                    MemoryValue::new_field(x),
                    MemoryValue::new_field(y),
                    MemoryValue::new_field(infinite),
                ],
            );
            Ok(())
        }
        BlackBoxOp::PedersenCommitment { inputs, domain_separator, output } => {
            let inputs: Vec<F> = read_heap_vector(memory, inputs)
                .iter()
                .map(|x| *x.extract_field().unwrap())
                .collect();
            let domain_separator: u32 =
                memory.read(*domain_separator).try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(
                        BlackBoxFunc::PedersenCommitment,
                        "Invalid signature length".to_string(),
                    )
                })?;
            let (x, y) = solver.pedersen_commitment(&inputs, domain_separator)?;
            memory.write_slice(
                memory.read_ref(output.pointer),
                &[MemoryValue::new_field(x), MemoryValue::new_field(y)],
            );
            Ok(())
        }
        BlackBoxOp::PedersenHash { inputs, domain_separator, output } => {
            let inputs: Vec<F> = read_heap_vector(memory, inputs)
                .iter()
                .map(|x| *x.extract_field().unwrap())
                .collect();
            let domain_separator: u32 =
                memory.read(*domain_separator).try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(
                        BlackBoxFunc::PedersenCommitment,
                        "Invalid signature length".to_string(),
                    )
                })?;
            let hash = solver.pedersen_hash(&inputs, domain_separator)?;
            memory.write(*output, MemoryValue::new_field(hash));
            Ok(())
        }
        BlackBoxOp::BigIntAdd { lhs, rhs, output } => {
            let lhs = memory.read(*lhs).try_into().unwrap();
            let rhs = memory.read(*rhs).try_into().unwrap();
            let output = memory.read(*output).try_into().unwrap();
            bigint_solver.bigint_op(lhs, rhs, output, BlackBoxFunc::BigIntAdd)?;
            Ok(())
        }
        BlackBoxOp::BigIntSub { lhs, rhs, output } => {
            let lhs = memory.read(*lhs).try_into().unwrap();
            let rhs = memory.read(*rhs).try_into().unwrap();
            let output = memory.read(*output).try_into().unwrap();
            bigint_solver.bigint_op(lhs, rhs, output, BlackBoxFunc::BigIntSub)?;
            Ok(())
        }
        BlackBoxOp::BigIntMul { lhs, rhs, output } => {
            let lhs = memory.read(*lhs).try_into().unwrap();
            let rhs = memory.read(*rhs).try_into().unwrap();
            let output = memory.read(*output).try_into().unwrap();
            bigint_solver.bigint_op(lhs, rhs, output, BlackBoxFunc::BigIntMul)?;
            Ok(())
        }
        BlackBoxOp::BigIntDiv { lhs, rhs, output } => {
            let lhs = memory.read(*lhs).try_into().unwrap();
            let rhs = memory.read(*rhs).try_into().unwrap();
            let output = memory.read(*output).try_into().unwrap();
            bigint_solver.bigint_op(lhs, rhs, output, BlackBoxFunc::BigIntDiv)?;
            Ok(())
        }
        BlackBoxOp::BigIntFromLeBytes { inputs, modulus, output } => {
            let input = read_heap_vector(memory, inputs);
            let input: Vec<u8> = input.iter().map(|x| x.try_into().unwrap()).collect();
            let modulus = read_heap_vector(memory, modulus);
            let modulus: Vec<u8> = modulus.iter().map(|x| x.try_into().unwrap()).collect();
            let output = memory.read(*output).try_into().unwrap();
            bigint_solver.bigint_from_bytes(&input, &modulus, output)?;
            Ok(())
        }
        BlackBoxOp::BigIntToLeBytes { input, output } => {
            let input: u32 = memory.read(*input).try_into().unwrap();
            let bytes = bigint_solver.bigint_to_bytes(input)?;
            let mut values = Vec::new();
            for i in 0..32 {
                if i < bytes.len() {
                    values.push(bytes[i].into());
                } else {
                    values.push(0_u8.into());
                }
            }
            memory.write_slice(memory.read_ref(output.pointer), &values);
            Ok(())
        }
        BlackBoxOp::Poseidon2Permutation { message, output, len } => {
            let input = read_heap_vector(memory, message);
            let input: Vec<F> = input.iter().map(|x| *x.extract_field().unwrap()).collect();
            let len = memory.read(*len).try_into().unwrap();
            let result = solver.poseidon2_permutation(&input, len)?;
            let mut values = Vec::new();
            for i in result {
                values.push(MemoryValue::new_field(i));
            }
            memory.write_slice(memory.read_ref(output.pointer), &values);
            Ok(())
        }
        BlackBoxOp::Sha256Compression { input, hash_values, output } => {
            let mut message = [0; 16];
            let inputs = read_heap_vector(memory, input);
            if inputs.len() != 16 {
                return Err(BlackBoxResolutionError::Failed(
                    BlackBoxFunc::Sha256Compression,
                    format!("Expected 16 inputs but encountered {}", &inputs.len()),
                ));
            }
            for (i, input) in inputs.iter().enumerate() {
                message[i] = input.try_into().unwrap();
            }
            let mut state = [0; 8];
            let values = read_heap_vector(memory, hash_values);
            if values.len() != 8 {
                return Err(BlackBoxResolutionError::Failed(
                    BlackBoxFunc::Sha256Compression,
                    format!("Expected 8 values but encountered {}", &values.len()),
                ));
            }
            for (i, value) in values.iter().enumerate() {
                state[i] = value.try_into().unwrap();
            }

            sha256compression(&mut state, &message);
            let state = state.map(|x| x.into());

            memory.write_slice(memory.read_ref(output.pointer), &state);
            Ok(())
        }
        BlackBoxOp::ToRadix { input, radix, output } => {
            let input: F = *memory.read(*input).extract_field().expect("ToRadix input not a field");

            let mut input = BigUint::from_bytes_be(&input.to_be_bytes());
            let radix = BigUint::from(*radix);

            let mut limbs: Vec<MemoryValue<F>> = Vec::with_capacity(output.size);

            for _ in 0..output.size {
                let limb = &input % &radix;
                limbs.push(MemoryValue::new_field(F::from_be_bytes_reduce(&limb.to_bytes_be())));
                input /= &radix;
            }

            memory.write_slice(memory.read_ref(output.pointer), &limbs);

            Ok(())
        }
    }
}

fn black_box_function_from_op(op: &BlackBoxOp) -> BlackBoxFunc {
    match op {
        BlackBoxOp::AES128Encrypt { .. } => BlackBoxFunc::AES128Encrypt,
        BlackBoxOp::Sha256 { .. } => BlackBoxFunc::SHA256,
        BlackBoxOp::Blake2s { .. } => BlackBoxFunc::Blake2s,
        BlackBoxOp::Blake3 { .. } => BlackBoxFunc::Blake3,
        BlackBoxOp::Keccak256 { .. } => BlackBoxFunc::Keccak256,
        BlackBoxOp::Keccakf1600 { .. } => BlackBoxFunc::Keccakf1600,
        BlackBoxOp::EcdsaSecp256k1 { .. } => BlackBoxFunc::EcdsaSecp256k1,
        BlackBoxOp::EcdsaSecp256r1 { .. } => BlackBoxFunc::EcdsaSecp256r1,
        BlackBoxOp::SchnorrVerify { .. } => BlackBoxFunc::SchnorrVerify,
        BlackBoxOp::PedersenCommitment { .. } => BlackBoxFunc::PedersenCommitment,
        BlackBoxOp::PedersenHash { .. } => BlackBoxFunc::PedersenHash,
        BlackBoxOp::MultiScalarMul { .. } => BlackBoxFunc::MultiScalarMul,
        BlackBoxOp::EmbeddedCurveAdd { .. } => BlackBoxFunc::EmbeddedCurveAdd,
        BlackBoxOp::BigIntAdd { .. } => BlackBoxFunc::BigIntAdd,
        BlackBoxOp::BigIntSub { .. } => BlackBoxFunc::BigIntSub,
        BlackBoxOp::BigIntMul { .. } => BlackBoxFunc::BigIntMul,
        BlackBoxOp::BigIntDiv { .. } => BlackBoxFunc::BigIntDiv,
        BlackBoxOp::BigIntFromLeBytes { .. } => BlackBoxFunc::BigIntFromLeBytes,
        BlackBoxOp::BigIntToLeBytes { .. } => BlackBoxFunc::BigIntToLeBytes,
        BlackBoxOp::Poseidon2Permutation { .. } => BlackBoxFunc::Poseidon2Permutation,
        BlackBoxOp::Sha256Compression { .. } => BlackBoxFunc::Sha256Compression,
        BlackBoxOp::ToRadix { .. } => unreachable!("ToRadix is not an ACIR BlackBoxFunc"),
    }
}

#[cfg(test)]
mod test {
    use acir::{
        brillig::{BlackBoxOp, MemoryAddress},
        FieldElement,
    };
    use acvm_blackbox_solver::{BigIntSolver, StubbedBlackBoxSolver};

    use crate::{
        black_box::{evaluate_black_box, to_u8_vec, to_value_vec},
        HeapArray, HeapVector, Memory,
    };

    #[test]
    fn sha256() {
        let message: Vec<u8> = b"hello world".to_vec();
        let message_length = message.len();

        let mut memory: Memory<FieldElement> = Memory::default();
        let message_pointer = 3;
        let result_pointer = message_pointer + message_length;
        memory.write(MemoryAddress(0), message_pointer.into());
        memory.write(MemoryAddress(1), message_length.into());
        memory.write(MemoryAddress(2), result_pointer.into());
        memory.write_slice(MemoryAddress(message_pointer), to_value_vec(&message).as_slice());

        let op = BlackBoxOp::Sha256 {
            message: HeapVector { pointer: 0.into(), size: 1.into() },
            output: HeapArray { pointer: 2.into(), size: 32 },
        };

        evaluate_black_box(&op, &StubbedBlackBoxSolver, &mut memory, &mut BigIntSolver::default())
            .unwrap();

        let result = memory.read_slice(MemoryAddress(result_pointer), 32);

        assert_eq!(
            to_u8_vec(result),
            vec![
                185, 77, 39, 185, 147, 77, 62, 8, 165, 46, 82, 215, 218, 125, 171, 250, 196, 132,
                239, 227, 122, 83, 128, 238, 144, 136, 247, 172, 226, 239, 205, 233
            ]
        );
    }
}
