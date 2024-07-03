use acvm::{
    acir::{brillig::BlackBoxOp, BlackBoxFunc},
    AcirField,
};

use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligVariable, BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    BrilligBinaryOp, BrilligContext,
};

/// Transforms SSA's black box function calls into the corresponding brillig instructions
/// Extracting arguments and results from the SSA function call
/// And making any necessary type conversions to adapt noir's blackbox calls to brillig's
pub(crate) fn convert_black_box_call<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F>,
    bb_func: &BlackBoxFunc,
    function_arguments: &[BrilligVariable],
    function_results: &[BrilligVariable],
) {
    match bb_func {
        BlackBoxFunc::SHA256 => {
            if let ([message], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::Sha256 {
                    message: message_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                });
            } else {
                unreachable!("ICE: SHA256 expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Blake2s => {
            if let ([message], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::Blake2s {
                    message: message_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                });
            } else {
                unreachable!("ICE: Blake2s expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Blake3 => {
            if let ([message], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::Blake3 {
                    message: message_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                });
            } else {
                unreachable!("ICE: Blake3 expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Keccak256 => {
            if let (
                [message, BrilligVariable::SingleAddr(message_size)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let mut message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                let message_size_as_usize =
                    SingleAddrVariable::new_usize(brillig_context.allocate_register());
                // Message_size is not usize
                brillig_context.cast_instruction(message_size_as_usize, *message_size);

                message_vector.size = message_size_as_usize.address;

                brillig_context.black_box_op_instruction(BlackBoxOp::Keccak256 {
                    message: message_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                });
                brillig_context.deallocate_single_addr(message_size_as_usize);
            } else {
                unreachable!("ICE: Keccak256 expects message, message size and result array")
            }
        }
        BlackBoxFunc::Keccakf1600 => {
            if let ([message], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let state_vector = convert_array_or_vector(brillig_context, message, bb_func);

                brillig_context.black_box_op_instruction(BlackBoxOp::Keccakf1600 {
                    message: state_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                });
            } else {
                unreachable!("ICE: Keccakf1600 expects one array argument and one array result")
            }
        }
        BlackBoxFunc::EcdsaSecp256k1 => {
            if let (
                [BrilligVariable::BrilligArray(public_key_x), BrilligVariable::BrilligArray(public_key_y), BrilligVariable::BrilligArray(signature), message],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_hash_vector =
                    convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256k1 {
                    hashed_msg: message_hash_vector.to_heap_vector(),
                    public_key_x: public_key_x.to_heap_array(),
                    public_key_y: public_key_y.to_heap_array(),
                    signature: signature.to_heap_array(),
                    result: result_register.address,
                });
            } else {
                unreachable!(
                    "ICE: EcdsaSecp256k1 expects four array arguments and one register result"
                )
            }
        }
        BlackBoxFunc::EcdsaSecp256r1 => {
            if let (
                [BrilligVariable::BrilligArray(public_key_x), BrilligVariable::BrilligArray(public_key_y), BrilligVariable::BrilligArray(signature), message],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_hash_vector =
                    convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256r1 {
                    hashed_msg: message_hash_vector.to_heap_vector(),
                    public_key_x: public_key_x.to_heap_array(),
                    public_key_y: public_key_y.to_heap_array(),
                    signature: signature.to_heap_array(),
                    result: result_register.address,
                });
            } else {
                unreachable!(
                    "ICE: EcdsaSecp256r1 expects four array arguments and one register result"
                )
            }
        }

        BlackBoxFunc::PedersenCommitment => {
            if let (
                [message, BrilligVariable::SingleAddr(domain_separator)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::PedersenCommitment {
                    inputs: message_vector.to_heap_vector(),
                    domain_separator: domain_separator.address,
                    output: result_array.to_heap_array(),
                });
            } else {
                unreachable!("ICE: Pedersen expects one array argument, a register for the domain separator, and one array result")
            }
        }
        BlackBoxFunc::PedersenHash => {
            if let (
                [message, BrilligVariable::SingleAddr(domain_separator)],
                [BrilligVariable::SingleAddr(result)],
            ) = (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::PedersenHash {
                    inputs: message_vector.to_heap_vector(),
                    domain_separator: domain_separator.address,
                    output: result.address,
                });
            } else {
                unreachable!("ICE: Pedersen hash expects one array argument, a register for the domain separator, and one register result")
            }
        }
        BlackBoxFunc::SchnorrVerify => {
            if let (
                [BrilligVariable::SingleAddr(public_key_x), BrilligVariable::SingleAddr(public_key_y), BrilligVariable::BrilligArray(signature), message],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_hash = convert_array_or_vector(brillig_context, message, bb_func);
                let signature = brillig_context.array_to_vector_instruction(signature);
                brillig_context.black_box_op_instruction(BlackBoxOp::SchnorrVerify {
                    public_key_x: public_key_x.address,
                    public_key_y: public_key_y.address,
                    message: message_hash.to_heap_vector(),
                    signature: signature.to_heap_vector(),
                    result: result_register.address,
                });
            } else {
                unreachable!("ICE: Schnorr verify expects two registers for the public key, an array for signature, an array for the message hash and one result register")
            }
        }
        BlackBoxFunc::MultiScalarMul => {
            if let ([points, scalars], [BrilligVariable::BrilligArray(outputs)]) =
                (function_arguments, function_results)
            {
                let points = convert_array_or_vector(brillig_context, points, bb_func);
                let scalars = convert_array_or_vector(brillig_context, scalars, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::MultiScalarMul {
                    points: points.to_heap_vector(),
                    scalars: scalars.to_heap_vector(),
                    outputs: outputs.to_heap_array(),
                });
            } else {
                unreachable!(
                    "ICE: MultiScalarMul expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::EmbeddedCurveAdd => {
            if let (
                [BrilligVariable::SingleAddr(input1_x), BrilligVariable::SingleAddr(input1_y), BrilligVariable::SingleAddr(input1_infinite), BrilligVariable::SingleAddr(input2_x), BrilligVariable::SingleAddr(input2_y), BrilligVariable::SingleAddr(input2_infinite)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::EmbeddedCurveAdd {
                    input1_x: input1_x.address,
                    input1_y: input1_y.address,
                    input1_infinite: input1_infinite.address,
                    input2_x: input2_x.address,
                    input2_y: input2_y.address,
                    input2_infinite: input2_infinite.address,
                    result: result_array.to_heap_array(),
                });
            } else {
                unreachable!(
                    "ICE: EmbeddedCurveAdd expects four register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::AND => {
            unreachable!("ICE: `BlackBoxFunc::AND` calls should be transformed into a `BinaryOp`")
        }
        BlackBoxFunc::XOR => {
            unreachable!("ICE: `BlackBoxFunc::XOR` calls should be transformed into a `BinaryOp`")
        }
        BlackBoxFunc::RANGE => unreachable!(
            "ICE: `BlackBoxFunc::RANGE` calls should be transformed into a `Instruction::Cast`"
        ),
        BlackBoxFunc::RecursiveAggregation => {}
        BlackBoxFunc::BigIntAdd => {
            if let (
                [BrilligVariable::SingleAddr(lhs), BrilligVariable::SingleAddr(lhs_modulus), BrilligVariable::SingleAddr(rhs), BrilligVariable::SingleAddr(rhs_modulus)],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(modulus_id)],
            ) = (function_arguments, function_results)
            {
                prepare_bigint_output(brillig_context, lhs_modulus, rhs_modulus, modulus_id);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntAdd {
                    lhs: lhs.address,
                    rhs: rhs.address,
                    output: output.address,
                });
            } else {
                unreachable!(
                    "ICE: BigIntAdd expects four register arguments and two result registers"
                )
            }
        }
        BlackBoxFunc::BigIntSub => {
            if let (
                [BrilligVariable::SingleAddr(lhs), BrilligVariable::SingleAddr(lhs_modulus), BrilligVariable::SingleAddr(rhs), BrilligVariable::SingleAddr(rhs_modulus)],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(modulus_id)],
            ) = (function_arguments, function_results)
            {
                prepare_bigint_output(brillig_context, lhs_modulus, rhs_modulus, modulus_id);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntSub {
                    lhs: lhs.address,
                    rhs: rhs.address,
                    output: output.address,
                });
            } else {
                unreachable!(
                    "ICE: BigIntSub expects four register arguments and two result registers"
                )
            }
        }
        BlackBoxFunc::BigIntMul => {
            if let (
                [BrilligVariable::SingleAddr(lhs), BrilligVariable::SingleAddr(lhs_modulus), BrilligVariable::SingleAddr(rhs), BrilligVariable::SingleAddr(rhs_modulus)],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(modulus_id)],
            ) = (function_arguments, function_results)
            {
                prepare_bigint_output(brillig_context, lhs_modulus, rhs_modulus, modulus_id);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntMul {
                    lhs: lhs.address,
                    rhs: rhs.address,
                    output: output.address,
                });
            } else {
                unreachable!(
                    "ICE: BigIntMul expects four register arguments and two result registers"
                )
            }
        }
        BlackBoxFunc::BigIntDiv => {
            if let (
                [BrilligVariable::SingleAddr(lhs), BrilligVariable::SingleAddr(lhs_modulus), BrilligVariable::SingleAddr(rhs), BrilligVariable::SingleAddr(rhs_modulus)],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(modulus_id)],
            ) = (function_arguments, function_results)
            {
                prepare_bigint_output(brillig_context, lhs_modulus, rhs_modulus, modulus_id);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntDiv {
                    lhs: lhs.address,
                    rhs: rhs.address,
                    output: output.address,
                });
            } else {
                unreachable!(
                    "ICE: BigIntDiv expects four register arguments and two result registers"
                )
            }
        }
        BlackBoxFunc::BigIntFromLeBytes => {
            if let (
                [inputs, modulus],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(_modulus_id)],
            ) = (function_arguments, function_results)
            {
                let inputs_vector = convert_array_or_vector(brillig_context, inputs, bb_func);
                let modulus_vector = convert_array_or_vector(brillig_context, modulus, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntFromLeBytes {
                    inputs: inputs_vector.to_heap_vector(),
                    modulus: modulus_vector.to_heap_vector(),
                    output: output.address,
                });
            } else {
                unreachable!(
                    "ICE: BigIntFromLeBytes expects a register and an array  as arguments and two result registers"
                )
            }
        }
        BlackBoxFunc::BigIntToLeBytes => {
            if let (
                [BrilligVariable::SingleAddr(input), BrilligVariable::SingleAddr(_modulus)],
                [result_array],
            ) = (function_arguments, function_results)
            {
                let output = convert_array_or_vector(brillig_context, result_array, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntToLeBytes {
                    input: input.address,
                    output: output.to_heap_vector(),
                });
            } else {
                unreachable!(
                    "ICE: BigIntToLeBytes expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::Poseidon2Permutation => {
            if let (
                [message, BrilligVariable::SingleAddr(state_len)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::Poseidon2Permutation {
                    message: message_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                    len: state_len.address,
                });
            } else {
                unreachable!("ICE: Poseidon2Permutation expects one array argument, a length and one array result")
            }
        }
        BlackBoxFunc::Sha256Compression => {
            if let ([message, hash_values], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                let hash_vector = convert_array_or_vector(brillig_context, hash_values, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::Sha256Compression {
                    input: message_vector.to_heap_vector(),
                    hash_values: hash_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                });
            } else {
                unreachable!("ICE: Sha256Compression expects two array argument, one array result")
            }
        }
        BlackBoxFunc::AES128Encrypt => {
            if let (
                [inputs, BrilligVariable::BrilligArray(iv), BrilligVariable::BrilligArray(key)],
                [BrilligVariable::SingleAddr(out_len), outputs],
            ) = (function_arguments, function_results)
            {
                let inputs = convert_array_or_vector(brillig_context, inputs, bb_func);
                let outputs = convert_array_or_vector(brillig_context, outputs, bb_func);
                let output_vec = outputs.to_heap_vector();
                brillig_context.black_box_op_instruction(BlackBoxOp::AES128Encrypt {
                    inputs: inputs.to_heap_vector(),
                    iv: iv.to_heap_array(),
                    key: key.to_heap_array(),
                    outputs: output_vec,
                });
                brillig_context.mov_instruction(out_len.address, output_vec.size);
                // Returns slice, so we need to allocate memory for it after the fact
                brillig_context.increase_free_memory_pointer_instruction(output_vec.size);
            } else {
                unreachable!("ICE: AES128Encrypt expects three array arguments, one array result")
            }
        }
    }
}

fn convert_array_or_vector<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F>,
    array_or_vector: &BrilligVariable,
    bb_func: &BlackBoxFunc,
) -> BrilligVector {
    match array_or_vector {
        BrilligVariable::BrilligArray(array) => brillig_context.array_to_vector_instruction(array),
        BrilligVariable::BrilligVector(vector) => *vector,
        _ => unreachable!(
            "ICE: {} expected an array or a vector, but got {:?}",
            bb_func.name(),
            array_or_vector
        ),
    }
}

fn prepare_bigint_output<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F>,
    lhs_modulus: &SingleAddrVariable,
    rhs_modulus: &SingleAddrVariable,
    modulus_id: &SingleAddrVariable,
) {
    // Check moduli
    let condition = brillig_context.allocate_register();
    let condition_adr = SingleAddrVariable { address: condition, bit_size: 1 };
    brillig_context.binary_instruction(
        *lhs_modulus,
        *rhs_modulus,
        condition_adr,
        BrilligBinaryOp::Equals,
    );
    brillig_context.codegen_constrain(
        condition_adr,
        Some("moduli should be identical in BigInt operation".to_string()),
    );
    brillig_context.deallocate_register(condition);

    brillig_context.mov_instruction(modulus_id.address, lhs_modulus.address);
}
