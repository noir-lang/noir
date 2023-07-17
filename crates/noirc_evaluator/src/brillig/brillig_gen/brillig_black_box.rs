use acvm::acir::{
    brillig::{BlackBoxOp, HeapVector, RegisterOrMemory},
    BlackBoxFunc,
};

use crate::brillig::brillig_ir::BrilligContext;

/// Transforms SSA's black box function calls into the corresponding brillig instructions
/// Extracting arguments and results from the SSA function call
/// And making any necessary type conversions to adapt noir's blackbox calls to brillig's
pub(crate) fn convert_black_box_call(
    brillig_context: &mut BrilligContext,
    bb_func: &BlackBoxFunc,
    function_arguments: &[RegisterOrMemory],
    function_results: &[RegisterOrMemory],
) {
    match bb_func {
        BlackBoxFunc::SHA256 => {
            if let (
                [RegisterOrMemory::HeapArray(message_array)],
                [RegisterOrMemory::HeapArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_vector = brillig_context.array_to_vector(message_array);
                brillig_context.black_box_op_instruction(BlackBoxOp::Sha256 {
                    message: message_vector,
                    output: *result_array,
                });
            } else {
                unreachable!("ICE: SHA256 expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Blake2s => {
            if let (
                [RegisterOrMemory::HeapArray(message_array)],
                [RegisterOrMemory::HeapArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_vector = brillig_context.array_to_vector(message_array);
                brillig_context.black_box_op_instruction(BlackBoxOp::Blake2s {
                    message: message_vector,
                    output: *result_array,
                });
            } else {
                unreachable!("ICE: Blake2s expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Keccak256 => {
            if let (
                [RegisterOrMemory::HeapArray(message_array), RegisterOrMemory::RegisterIndex(array_size)],
                [RegisterOrMemory::HeapArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_vector =
                    HeapVector { size: *array_size, pointer: message_array.pointer };
                brillig_context.black_box_op_instruction(BlackBoxOp::Keccak256 {
                    message: message_vector,
                    output: *result_array,
                });
            } else {
                unreachable!("ICE: Keccak256 expects message, message size and result array")
            }
        }
        BlackBoxFunc::HashToField128Security => {
            if let (
                [RegisterOrMemory::HeapArray(message_array)],
                [RegisterOrMemory::RegisterIndex(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_vector = brillig_context.array_to_vector(message_array);
                brillig_context.black_box_op_instruction(BlackBoxOp::HashToField128Security {
                    message: message_vector,
                    output: *result_register,
                });
            } else {
                unreachable!("ICE: HashToField128Security expects one array argument and one register result")
            }
        }
        BlackBoxFunc::EcdsaSecp256k1 => {
            if let (
                [RegisterOrMemory::HeapArray(public_key_x), RegisterOrMemory::HeapArray(public_key_y), RegisterOrMemory::HeapArray(signature), RegisterOrMemory::HeapArray(message_hash)],
                [RegisterOrMemory::RegisterIndex(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_hash_vector = brillig_context.array_to_vector(message_hash);
                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256k1 {
                    hashed_msg: message_hash_vector,
                    public_key_x: *public_key_x,
                    public_key_y: *public_key_y,
                    signature: *signature,
                    result: *result_register,
                });
            } else {
                unreachable!(
                    "ICE: EcdsaSecp256k1 expects four array arguments and one register result"
                )
            }
        }
        BlackBoxFunc::Pedersen => {
            if let (
                [RegisterOrMemory::HeapArray(message_array), RegisterOrMemory::RegisterIndex(domain_separator)],
                [RegisterOrMemory::HeapArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_vector = brillig_context.array_to_vector(message_array);
                brillig_context.black_box_op_instruction(BlackBoxOp::Pedersen {
                    inputs: message_vector,
                    domain_separator: *domain_separator,
                    output: *result_array,
                });
            } else {
                unreachable!("ICE: Pedersen expects one array argument, a register for the domain separator, and one array result")
            }
        }
        BlackBoxFunc::SchnorrVerify => {
            if let (
                [RegisterOrMemory::RegisterIndex(public_key_x), RegisterOrMemory::RegisterIndex(public_key_y), RegisterOrMemory::HeapArray(signature), RegisterOrMemory::HeapArray(message_hash)],
                [RegisterOrMemory::RegisterIndex(result_register)],
            ) = (function_arguments, function_results)
            {
                let message_hash = brillig_context.array_to_vector(message_hash);
                let signature = brillig_context.array_to_vector(signature);
                brillig_context.black_box_op_instruction(BlackBoxOp::SchnorrVerify {
                    public_key_x: *public_key_x,
                    public_key_y: *public_key_y,
                    message: message_hash,
                    signature,
                    result: *result_register,
                });
            } else {
                unreachable!("ICE: Schnorr verify expects two registers for the public key, an array for signature, an array for the message hash and one result register")
            }
        }
        BlackBoxFunc::FixedBaseScalarMul => {
            if let (
                [RegisterOrMemory::RegisterIndex(scalar)],
                [RegisterOrMemory::HeapArray(result_array)],
            ) = (function_arguments, function_results)
            {
                brillig_context.black_box_op_instruction(BlackBoxOp::FixedBaseScalarMul {
                    input: *scalar,
                    result: *result_array,
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
