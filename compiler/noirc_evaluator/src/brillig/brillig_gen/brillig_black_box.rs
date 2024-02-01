use acvm::acir::{brillig::BlackBoxOp, BlackBoxFunc};

use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligVariable, BrilligVector},
    BrilligContext,
};

/// Transforms SSA's black box function calls into the corresponding brillig instructions
/// Extracting arguments and results from the SSA function call
/// And making any necessary type conversions to adapt noir's blackbox calls to brillig's
pub(crate) fn convert_black_box_call(
    brillig_context: &mut BrilligContext,
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
                [message, BrilligVariable::Simple(array_size)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let mut message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                message_vector.size = *array_size;

                brillig_context.black_box_op_instruction(BlackBoxOp::Keccak256 {
                    message: message_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                });
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
                [BrilligVariable::Simple(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_hash_vector =
                    convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256k1 {
                    hashed_msg: message_hash_vector.to_heap_vector(),
                    public_key_x: public_key_x.to_heap_array(),
                    public_key_y: public_key_y.to_heap_array(),
                    signature: signature.to_heap_array(),
                    result: *result_register,
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
                [BrilligVariable::Simple(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_hash_vector =
                    convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256r1 {
                    hashed_msg: message_hash_vector.to_heap_vector(),
                    public_key_x: public_key_x.to_heap_array(),
                    public_key_y: public_key_y.to_heap_array(),
                    signature: signature.to_heap_array(),
                    result: *result_register,
                });
            } else {
                unreachable!(
                    "ICE: EcdsaSecp256r1 expects four array arguments and one register result"
                )
            }
        }

        BlackBoxFunc::PedersenCommitment => {
            if let (
                [message, BrilligVariable::Simple(domain_separator)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::PedersenCommitment {
                    inputs: message_vector.to_heap_vector(),
                    domain_separator: *domain_separator,
                    output: result_array.to_heap_array(),
                });
            } else {
                unreachable!("ICE: Pedersen expects one array argument, a register for the domain separator, and one array result")
            }
        }
        BlackBoxFunc::PedersenHash => {
            if let (
                [message, BrilligVariable::Simple(domain_separator)],
                [BrilligVariable::Simple(result)],
            ) = (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::PedersenHash {
                    inputs: message_vector.to_heap_vector(),
                    domain_separator: *domain_separator,
                    output: *result,
                });
            } else {
                unreachable!("ICE: Pedersen hash expects one array argument, a register for the domain separator, and one register result")
            }
        }
        BlackBoxFunc::SchnorrVerify => {
            if let (
                [BrilligVariable::Simple(public_key_x), BrilligVariable::Simple(public_key_y), BrilligVariable::BrilligArray(signature), message],
                [BrilligVariable::Simple(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_hash = convert_array_or_vector(brillig_context, message, bb_func);
                let signature = brillig_context.array_to_vector(signature);
                brillig_context.black_box_op_instruction(BlackBoxOp::SchnorrVerify {
                    public_key_x: *public_key_x,
                    public_key_y: *public_key_y,
                    message: message_hash.to_heap_vector(),
                    signature: signature.to_heap_vector(),
                    result: *result_register,
                });
            } else {
                unreachable!("ICE: Schnorr verify expects two registers for the public key, an array for signature, an array for the message hash and one result register")
            }
        }
        BlackBoxFunc::FixedBaseScalarMul => {
            if let (
                [BrilligVariable::Simple(low), BrilligVariable::Simple(high)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::FixedBaseScalarMul {
                    low: *low,
                    high: *high,
                    result: result_array.to_heap_array(),
                });
            } else {
                unreachable!(
                    "ICE: FixedBaseScalarMul expects one register argument and one array result"
                )
            }
        }
        BlackBoxFunc::EmbeddedCurveAdd => {
            if let (
                [BrilligVariable::Simple(input1_x), BrilligVariable::Simple(input1_y), BrilligVariable::Simple(input2_x), BrilligVariable::Simple(input2_y)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::EmbeddedCurveAdd {
                    input1_x: *input1_x,
                    input1_y: *input1_y,
                    input2_x: *input2_x,
                    input2_y: *input2_y,
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
        BlackBoxFunc::RecursiveAggregation => unimplemented!(
            "ICE: `BlackBoxFunc::RecursiveAggregation` is not implemented by the Brillig VM"
        ),
        BlackBoxFunc::BigIntAdd => {
            if let (
                [BrilligVariable::Simple(lhs), BrilligVariable::Simple(rhs)],
                [BrilligVariable::Simple(output)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntAdd {
                    lhs: *lhs,
                    rhs: *rhs,
                    output: *output,
                });
            } else {
                unreachable!(
                    "ICE: EmbeddedCurveAdd expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::BigIntNeg => {
            if let (
                [BrilligVariable::Simple(lhs), BrilligVariable::Simple(rhs)],
                [BrilligVariable::Simple(output)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntNeg {
                    lhs: *lhs,
                    rhs: *rhs,
                    output: *output,
                });
            } else {
                unreachable!(
                    "ICE: EmbeddedCurveAdd expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::BigIntMul => {
            if let (
                [BrilligVariable::Simple(lhs), BrilligVariable::Simple(rhs)],
                [BrilligVariable::Simple(output)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntMul {
                    lhs: *lhs,
                    rhs: *rhs,
                    output: *output,
                });
            } else {
                unreachable!(
                    "ICE: EmbeddedCurveAdd expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::BigIntDiv => {
            if let (
                [BrilligVariable::Simple(lhs), BrilligVariable::Simple(rhs)],
                [BrilligVariable::Simple(output)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntDiv {
                    lhs: *lhs,
                    rhs: *rhs,
                    output: *output,
                });
            } else {
                unreachable!(
                    "ICE: EmbeddedCurveAdd expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::BigIntFromLeBytes => {
            if let ([inputs, modulus], [BrilligVariable::Simple(output)]) =
                (function_arguments, function_results)
            {
                let inputs_vector = convert_array_or_vector(brillig_context, inputs, bb_func);
                let modulus_vector = convert_array_or_vector(brillig_context, modulus, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntFromLeBytes {
                    inputs: inputs_vector.to_heap_vector(),
                    modulus: modulus_vector.to_heap_vector(),
                    output: *output,
                });
            } else {
                unreachable!(
                    "ICE: EmbeddedCurveAdd expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::BigIntToLeBytes => {
            if let (
                [BrilligVariable::Simple(input)],
                [BrilligVariable::BrilligVector(result_vector)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntToLeBytes {
                    input: *input,
                    output: result_vector.to_heap_vector(),
                });
            } else {
                unreachable!(
                    "ICE: EmbeddedCurveAdd expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::Poseidon2Permutation => {
            if let (
                [message, BrilligVariable::Simple(state_len)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::Poseidon2Permutation {
                    message: message_vector.to_heap_vector(),
                    output: result_array.to_heap_array(),
                    len: *state_len,
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
    }
}

fn convert_array_or_vector(
    brillig_context: &mut BrilligContext,
    array_or_vector: &BrilligVariable,
    bb_func: &BlackBoxFunc,
) -> BrilligVector {
    match array_or_vector {
        BrilligVariable::BrilligArray(array) => brillig_context.array_to_vector(array),
        BrilligVariable::BrilligVector(vector) => *vector,
        _ => unreachable!(
            "ICE: {} expected an array or a vector, but got {:?}",
            bb_func.name(),
            array_or_vector
        ),
    }
}
