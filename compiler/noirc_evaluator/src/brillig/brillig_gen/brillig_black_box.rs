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
        BlackBoxFunc::HashToField128Security => {
            if let ([message], [BrilligVariable::Simple(result_register)]) =
                (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, message, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::HashToField128Security {
                    message: message_vector.to_heap_vector(),
                    output: *result_register,
                });
            } else {
                unreachable!("ICE: HashToField128Security expects one array argument and one register result")
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
        _ => unimplemented!("ICE: Black box function {:?} is not implemented", bb_func),
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
