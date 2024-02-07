use acir::brillig::{BlackBoxOp, HeapArray, HeapVector, Value};
use acir::{BlackBoxFunc, FieldElement};
use acvm_blackbox_solver::{
    blake2s, blake3, ecdsa_secp256k1_verify, ecdsa_secp256r1_verify, keccak256, keccakf1600,
    sha256, BlackBoxFunctionSolver, BlackBoxResolutionError,
};

use crate::Memory;

fn read_heap_vector<'a>(memory: &'a Memory, vector: &HeapVector) -> &'a [Value] {
    memory.read_slice(memory.read_ref(vector.pointer), memory.read(vector.size).to_usize())
}

fn read_heap_array<'a>(memory: &'a Memory, array: &HeapArray) -> &'a [Value] {
    memory.read_slice(memory.read_ref(array.pointer), array.size)
}

/// Extracts the last byte of every value
fn to_u8_vec(inputs: &[Value]) -> Vec<u8> {
    let mut result = Vec::with_capacity(inputs.len());
    for input in inputs {
        let field_bytes = input.to_field().to_be_bytes();
        let byte = field_bytes.last().unwrap();
        result.push(*byte);
    }
    result
}

fn to_value_vec(input: &[u8]) -> Vec<Value> {
    input.iter().map(|x| Value::from(*x as usize)).collect()
}

pub(crate) fn evaluate_black_box<Solver: BlackBoxFunctionSolver>(
    op: &BlackBoxOp,
    solver: &Solver,
    memory: &mut Memory,
) -> Result<(), BlackBoxResolutionError> {
    match op {
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
                .map(|value| value.to_field().try_to_u64().unwrap())
                .collect();
            let state: [u64; 25] = state_vec.try_into().unwrap();

            let new_state = keccakf1600(state)?;

            let new_state: Vec<Value> =
                new_state.into_iter().map(|x| Value::from(x as usize)).collect();
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
            let public_key_x = memory.read(*public_key_x).to_field();
            let public_key_y = memory.read(*public_key_y).to_field();
            let message: Vec<u8> = to_u8_vec(read_heap_vector(memory, message));
            let signature: Vec<u8> = to_u8_vec(read_heap_vector(memory, signature));
            let verified =
                solver.schnorr_verify(&public_key_x, &public_key_y, &signature, &message)?;
            memory.write(*result, verified.into());
            Ok(())
        }
        BlackBoxOp::FixedBaseScalarMul { low, high, result } => {
            let low = memory.read(*low).to_field();
            let high = memory.read(*high).to_field();
            let (x, y) = solver.fixed_base_scalar_mul(&low, &high)?;
            memory.write_slice(memory.read_ref(result.pointer), &[x.into(), y.into()]);
            Ok(())
        }
        BlackBoxOp::EmbeddedCurveAdd { input1_x, input1_y, input2_x, input2_y, result } => {
            let input1_x = memory.read(*input1_x).to_field();
            let input1_y = memory.read(*input1_y).to_field();
            let input2_x = memory.read(*input2_x).to_field();
            let input2_y = memory.read(*input2_y).to_field();
            let (x, y) = solver.ec_add(&input1_x, &input1_y, &input2_x, &input2_y)?;
            memory.write_slice(memory.read_ref(result.pointer), &[x.into(), y.into()]);
            Ok(())
        }
        BlackBoxOp::PedersenCommitment { inputs, domain_separator, output } => {
            let inputs: Vec<FieldElement> =
                read_heap_vector(memory, inputs).iter().map(|x| x.to_field()).collect();
            let domain_separator: u32 =
                memory.read(*domain_separator).to_u128().try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(
                        BlackBoxFunc::PedersenCommitment,
                        "Invalid signature length".to_string(),
                    )
                })?;
            let (x, y) = solver.pedersen_commitment(&inputs, domain_separator)?;
            memory.write_slice(memory.read_ref(output.pointer), &[x.into(), y.into()]);
            Ok(())
        }
        BlackBoxOp::PedersenHash { inputs, domain_separator, output } => {
            let inputs: Vec<FieldElement> =
                read_heap_vector(memory, inputs).iter().map(|x| x.to_field()).collect();
            let domain_separator: u32 =
                memory.read(*domain_separator).to_u128().try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(
                        BlackBoxFunc::PedersenCommitment,
                        "Invalid signature length".to_string(),
                    )
                })?;
            let hash = solver.pedersen_hash(&inputs, domain_separator)?;
            memory.write(*output, hash.into());
            Ok(())
        }
        BlackBoxOp::BigIntAdd { .. } => todo!(),
        BlackBoxOp::BigIntSub { .. } => todo!(),
        BlackBoxOp::BigIntMul { .. } => todo!(),
        BlackBoxOp::BigIntDiv { .. } => todo!(),
        BlackBoxOp::BigIntFromLeBytes { .. } => todo!(),
        BlackBoxOp::BigIntToLeBytes { .. } => todo!(),
        BlackBoxOp::Poseidon2Permutation { .. } => todo!(),
        BlackBoxOp::Sha256Compression { .. } => todo!(),
    }
}

fn black_box_function_from_op(op: &BlackBoxOp) -> BlackBoxFunc {
    match op {
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
        BlackBoxOp::FixedBaseScalarMul { .. } => BlackBoxFunc::FixedBaseScalarMul,
        BlackBoxOp::EmbeddedCurveAdd { .. } => BlackBoxFunc::EmbeddedCurveAdd,
        BlackBoxOp::BigIntAdd { .. } => BlackBoxFunc::BigIntAdd,
        BlackBoxOp::BigIntSub { .. } => BlackBoxFunc::BigIntSub,
        BlackBoxOp::BigIntMul { .. } => BlackBoxFunc::BigIntMul,
        BlackBoxOp::BigIntDiv { .. } => BlackBoxFunc::BigIntDiv,
        BlackBoxOp::BigIntFromLeBytes { .. } => BlackBoxFunc::BigIntFromLeBytes,
        BlackBoxOp::BigIntToLeBytes { .. } => BlackBoxFunc::BigIntToLeBytes,
        BlackBoxOp::Poseidon2Permutation { .. } => BlackBoxFunc::Poseidon2Permutation,
        BlackBoxOp::Sha256Compression { .. } => BlackBoxFunc::Sha256Compression,
    }
}

#[cfg(test)]
mod test {
    use acir::brillig::{BlackBoxOp, MemoryAddress};

    use crate::{
        black_box::{evaluate_black_box, to_u8_vec, to_value_vec},
        DummyBlackBoxSolver, HeapArray, HeapVector, Memory,
    };

    #[test]
    fn sha256() {
        let message: Vec<u8> = b"hello world".to_vec();
        let message_length = message.len();

        let mut memory = Memory::default();
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

        evaluate_black_box(&op, &DummyBlackBoxSolver, &mut memory).unwrap();

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
