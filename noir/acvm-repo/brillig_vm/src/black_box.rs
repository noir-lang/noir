use acir::brillig::{BlackBoxOp, HeapArray, HeapVector, Value};
use acir::{BlackBoxFunc, FieldElement};
use acvm_blackbox_solver::{
    blake2s, blake3, ecdsa_secp256k1_verify, ecdsa_secp256r1_verify, keccak256, keccakf1600,
    sha256, BlackBoxFunctionSolver, BlackBoxResolutionError,
};

use crate::{Memory, Registers};

fn read_heap_vector<'a>(
    memory: &'a Memory,
    registers: &Registers,
    vector: &HeapVector,
) -> &'a [Value] {
    memory
        .read_slice(registers.get(vector.pointer).to_usize(), registers.get(vector.size).to_usize())
}

fn read_heap_array<'a>(
    memory: &'a Memory,
    registers: &Registers,
    array: &HeapArray,
) -> &'a [Value] {
    memory.read_slice(registers.get(array.pointer).to_usize(), array.size)
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
    registers: &mut Registers,
    memory: &mut Memory,
) -> Result<(), BlackBoxResolutionError> {
    match op {
        BlackBoxOp::Sha256 { message, output } => {
            let message = to_u8_vec(read_heap_vector(memory, registers, message));
            let bytes = sha256(message.as_slice())?;
            memory.write_slice(registers.get(output.pointer).to_usize(), &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Blake2s { message, output } => {
            let message = to_u8_vec(read_heap_vector(memory, registers, message));
            let bytes = blake2s(message.as_slice())?;
            memory.write_slice(registers.get(output.pointer).to_usize(), &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Blake3 { message, output } => {
            let message = to_u8_vec(read_heap_vector(memory, registers, message));
            let bytes = blake3(message.as_slice())?;
            memory.write_slice(registers.get(output.pointer).to_usize(), &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Keccak256 { message, output } => {
            let message = to_u8_vec(read_heap_vector(memory, registers, message));
            let bytes = keccak256(message.as_slice())?;
            memory.write_slice(registers.get(output.pointer).to_usize(), &to_value_vec(&bytes));
            Ok(())
        }
        BlackBoxOp::Keccakf1600 { message, output } => {
            let state_vec: Vec<u64> = read_heap_vector(memory, registers, message)
                .iter()
                .map(|value| value.to_field().try_to_u64().unwrap())
                .collect();
            let state: [u64; 25] = state_vec.try_into().unwrap();

            let new_state = keccakf1600(state)?;

            let new_state: Vec<Value> =
                new_state.into_iter().map(|x| Value::from(x as usize)).collect();
            memory.write_slice(registers.get(output.pointer).to_usize(), &new_state);
            Ok(())
        }
        BlackBoxOp::EcdsaSecp256k1 {
            hashed_msg,
            public_key_x,
            public_key_y,
            signature,
            result: result_register,
        }
        | BlackBoxOp::EcdsaSecp256r1 {
            hashed_msg,
            public_key_x,
            public_key_y,
            signature,
            result: result_register,
        } => {
            let bb_func = black_box_function_from_op(op);

            let public_key_x: [u8; 32] = to_u8_vec(read_heap_array(
                memory,
                registers,
                public_key_x,
            ))
            .try_into()
            .map_err(|_| {
                BlackBoxResolutionError::Failed(bb_func, "Invalid public key x length".to_string())
            })?;
            let public_key_y: [u8; 32] = to_u8_vec(read_heap_array(
                memory,
                registers,
                public_key_y,
            ))
            .try_into()
            .map_err(|_| {
                BlackBoxResolutionError::Failed(bb_func, "Invalid public key y length".to_string())
            })?;
            let signature: [u8; 64] = to_u8_vec(read_heap_array(memory, registers, signature))
                .try_into()
                .map_err(|_| {
                    BlackBoxResolutionError::Failed(bb_func, "Invalid signature length".to_string())
                })?;

            let hashed_msg = to_u8_vec(read_heap_vector(memory, registers, hashed_msg));

            let result = match op {
                BlackBoxOp::EcdsaSecp256k1 { .. } => {
                    ecdsa_secp256k1_verify(&hashed_msg, &public_key_x, &public_key_y, &signature)?
                }
                BlackBoxOp::EcdsaSecp256r1 { .. } => {
                    ecdsa_secp256r1_verify(&hashed_msg, &public_key_x, &public_key_y, &signature)?
                }
                _ => unreachable!("`BlackBoxOp` is guarded against being a non-ecdsa operation"),
            };

            registers.set(*result_register, result.into());
            Ok(())
        }
        BlackBoxOp::SchnorrVerify { public_key_x, public_key_y, message, signature, result } => {
            let public_key_x = registers.get(*public_key_x).to_field();
            let public_key_y = registers.get(*public_key_y).to_field();
            let message: Vec<u8> = to_u8_vec(read_heap_vector(memory, registers, message));
            let signature: Vec<u8> = to_u8_vec(read_heap_vector(memory, registers, signature));
            let verified =
                solver.schnorr_verify(&public_key_x, &public_key_y, &signature, &message)?;
            registers.set(*result, verified.into());
            Ok(())
        }
        BlackBoxOp::FixedBaseScalarMul { low, high, result } => {
            let low = registers.get(*low).to_field();
            let high = registers.get(*high).to_field();
            let (x, y) = solver.fixed_base_scalar_mul(&low, &high)?;
            memory.write_slice(registers.get(result.pointer).to_usize(), &[x.into(), y.into()]);
            Ok(())
        }
        BlackBoxOp::EmbeddedCurveAdd { input1_x, input1_y, input2_x, input2_y, result } => {
            let input1_x = registers.get(*input1_x).to_field();
            let input1_y = registers.get(*input1_y).to_field();
            let input2_x = registers.get(*input2_x).to_field();
            let input2_y = registers.get(*input2_y).to_field();
            let (x, y) = solver.ec_add(&input1_x, &input1_y, &input2_x, &input2_y)?;
            memory.write_slice(registers.get(result.pointer).to_usize(), &[x.into(), y.into()]);
            Ok(())
        }
        BlackBoxOp::EmbeddedCurveDouble { input1_x, input1_y, result } => {
            let input1_x = registers.get(*input1_x).to_field();
            let input1_y = registers.get(*input1_y).to_field();
            let (x, y) = solver.ec_double(&input1_x, &input1_y)?;
            memory.write_slice(registers.get(result.pointer).to_usize(), &[x.into(), y.into()]);
            Ok(())
        }
        BlackBoxOp::PedersenCommitment { inputs, domain_separator, output } => {
            let inputs: Vec<FieldElement> =
                read_heap_vector(memory, registers, inputs).iter().map(|x| x.to_field()).collect();
            let domain_separator: u32 =
                registers.get(*domain_separator).to_u128().try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(
                        BlackBoxFunc::PedersenCommitment,
                        "Invalid signature length".to_string(),
                    )
                })?;
            let (x, y) = solver.pedersen_commitment(&inputs, domain_separator)?;
            memory.write_slice(registers.get(output.pointer).to_usize(), &[x.into(), y.into()]);
            Ok(())
        }
        BlackBoxOp::PedersenHash { inputs, domain_separator, output } => {
            let inputs: Vec<FieldElement> =
                read_heap_vector(memory, registers, inputs).iter().map(|x| x.to_field()).collect();
            let domain_separator: u32 =
                registers.get(*domain_separator).to_u128().try_into().map_err(|_| {
                    BlackBoxResolutionError::Failed(
                        BlackBoxFunc::PedersenCommitment,
                        "Invalid signature length".to_string(),
                    )
                })?;
            let hash = solver.pedersen_hash(&inputs, domain_separator)?;
            registers.set(*output, hash.into());
            Ok(())
        }
        BlackBoxOp::BigIntAdd { .. } => todo!(),
        BlackBoxOp::BigIntNeg { .. } => todo!(),
        BlackBoxOp::BigIntMul { .. } => todo!(),
        BlackBoxOp::BigIntDiv { .. } => todo!(),
        BlackBoxOp::BigIntFromLeBytes { .. } => todo!(),
        BlackBoxOp::BigIntToLeBytes { .. } => todo!(),
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
        BlackBoxOp::EmbeddedCurveDouble { .. } => BlackBoxFunc::EmbeddedCurveDouble,
        BlackBoxOp::BigIntAdd { .. } => BlackBoxFunc::BigIntAdd,
        BlackBoxOp::BigIntNeg { .. } => BlackBoxFunc::BigIntNeg,
        BlackBoxOp::BigIntMul { .. } => BlackBoxFunc::BigIntMul,
        BlackBoxOp::BigIntDiv { .. } => BlackBoxFunc::BigIntDiv,
        BlackBoxOp::BigIntFromLeBytes { .. } => BlackBoxFunc::BigIntFromLeBytes,
        BlackBoxOp::BigIntToLeBytes { .. } => BlackBoxFunc::BigIntToLeBytes,
    }
}

#[cfg(test)]
mod test {
    use acir::brillig::BlackBoxOp;

    use crate::{
        black_box::{evaluate_black_box, to_u8_vec, to_value_vec},
        DummyBlackBoxSolver, HeapArray, HeapVector, Memory, Registers, Value,
    };

    #[test]
    fn sha256() {
        let message: Vec<u8> = b"hello world".to_vec();
        let message_length = message.len();

        let mut memory = Memory::from(vec![]);
        let message_pointer = 0;
        let result_pointer = message_pointer + message_length;
        memory.write_slice(message_pointer, to_value_vec(&message).as_slice());

        let mut registers = Registers {
            inner: vec![
                Value::from(message_pointer),
                Value::from(message_length),
                Value::from(result_pointer),
            ],
        };

        let op = BlackBoxOp::Sha256 {
            message: HeapVector { pointer: 0.into(), size: 1.into() },
            output: HeapArray { pointer: 2.into(), size: 32 },
        };

        evaluate_black_box(&op, &DummyBlackBoxSolver, &mut registers, &mut memory).unwrap();

        let result = memory.read_slice(result_pointer, 32);

        assert_eq!(
            to_u8_vec(result),
            vec![
                185, 77, 39, 185, 147, 77, 62, 8, 165, 46, 82, 215, 218, 125, 171, 250, 196, 132,
                239, 227, 122, 83, 128, 238, 144, 136, 247, 172, 226, 239, 205, 233
            ]
        );
    }
}
