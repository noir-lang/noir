//! Codegen for native (black box) function calls.
use acvm::{
    AcirField,
    acir::{BlackBoxFunc, brillig::BlackBoxOp},
};

use crate::brillig::brillig_ir::{
    BrilligContext, brillig_variable::BrilligVariable, debug_show::DebugToString,
    registers::RegisterAllocator,
};

/// Transforms SSA's black box function calls into the corresponding brillig instructions
/// Extracting arguments and results from the SSA function call
/// And making any necessary type conversions to adapt noir's blackbox calls to brillig's
pub(crate) fn convert_black_box_call<F: AcirField + DebugToString, Registers: RegisterAllocator>(
    brillig_context: &mut BrilligContext<F, Registers>,
    bb_func: &BlackBoxFunc,
    function_arguments: &[BrilligVariable],
    function_results: &[BrilligVariable],
) {
    match bb_func {
        BlackBoxFunc::Blake2s => {
            if let (
                [BrilligVariable::BrilligArray(message)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_array = brillig_context.codegen_brillig_array_to_heap_array(*message);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Blake2s {
                    message: *message_array,
                    output: *output_heap_array,
                });
            } else {
                unreachable!("ICE: Blake2s expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Blake3 => {
            if let (
                [BrilligVariable::BrilligArray(message)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_array = brillig_context.codegen_brillig_array_to_heap_array(*message);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Blake3 {
                    message: *message_array,
                    output: *output_heap_array,
                });
            } else {
                unreachable!("ICE: Blake3 expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Keccakf1600 => {
            if let (
                [BrilligVariable::BrilligArray(input_array)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let input_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*input_array);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Keccakf1600 {
                    input: *input_heap_array,
                    output: *output_heap_array,
                });
            } else {
                unreachable!("ICE: Keccakf1600 expects one array argument and one array result")
            }
        }
        BlackBoxFunc::EcdsaSecp256k1 => {
            if let (
                [
                    BrilligVariable::BrilligArray(public_key_x),
                    BrilligVariable::BrilligArray(public_key_y),
                    BrilligVariable::BrilligArray(signature),
                    BrilligVariable::BrilligArray(message),
                ],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let hashed_msg = brillig_context.codegen_brillig_array_to_heap_array(*message);
                let public_key_x =
                    brillig_context.codegen_brillig_array_to_heap_array(*public_key_x);
                let public_key_y =
                    brillig_context.codegen_brillig_array_to_heap_array(*public_key_y);
                let signature = brillig_context.codegen_brillig_array_to_heap_array(*signature);

                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256k1 {
                    hashed_msg: *hashed_msg,
                    public_key_x: *public_key_x,
                    public_key_y: *public_key_y,
                    signature: *signature,
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
                [
                    BrilligVariable::BrilligArray(public_key_x),
                    BrilligVariable::BrilligArray(public_key_y),
                    BrilligVariable::BrilligArray(signature),
                    BrilligVariable::BrilligArray(message),
                ],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let hashed_msg = brillig_context.codegen_brillig_array_to_heap_array(*message);
                let public_key_x =
                    brillig_context.codegen_brillig_array_to_heap_array(*public_key_x);
                let public_key_y =
                    brillig_context.codegen_brillig_array_to_heap_array(*public_key_y);
                let signature = brillig_context.codegen_brillig_array_to_heap_array(*signature);

                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256r1 {
                    hashed_msg: *hashed_msg,
                    public_key_x: *public_key_x,
                    public_key_y: *public_key_y,
                    signature: *signature,
                    result: result_register.address,
                });
            } else {
                unreachable!(
                    "ICE: EcdsaSecp256r1 expects four array arguments and one register result"
                )
            }
        }

        BlackBoxFunc::MultiScalarMul => {
            if let (
                [BrilligVariable::BrilligArray(points), BrilligVariable::BrilligArray(scalars)],
                [BrilligVariable::BrilligArray(outputs)],
            ) = (function_arguments, function_results)
            {
                let points = brillig_context.codegen_brillig_array_to_heap_array(*points);
                let scalars = brillig_context.codegen_brillig_array_to_heap_array(*scalars);
                let outputs = brillig_context.codegen_brillig_array_to_heap_array(*outputs);

                brillig_context.black_box_op_instruction(BlackBoxOp::MultiScalarMul {
                    points: *points,
                    scalars: *scalars,
                    outputs: *outputs,
                });
            } else {
                unreachable!(
                    "ICE: MultiScalarMul expects two register arguments and one array result"
                )
            }
        }
        BlackBoxFunc::EmbeddedCurveAdd => {
            if let (
                [
                    BrilligVariable::SingleAddr(input1_x),
                    BrilligVariable::SingleAddr(input1_y),
                    BrilligVariable::SingleAddr(input1_infinite),
                    BrilligVariable::SingleAddr(input2_x),
                    BrilligVariable::SingleAddr(input2_y),
                    BrilligVariable::SingleAddr(input2_infinite),
                ],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let result = brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::EmbeddedCurveAdd {
                    input1_x: input1_x.address,
                    input1_y: input1_y.address,
                    input1_infinite: input1_infinite.address,
                    input2_x: input2_x.address,
                    input2_y: input2_y.address,
                    input2_infinite: input2_infinite.address,
                    result: *result,
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
        BlackBoxFunc::Poseidon2Permutation => {
            if let (
                [BrilligVariable::BrilligArray(message)],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let message_array = brillig_context.codegen_brillig_array_to_heap_array(*message);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Poseidon2Permutation {
                    message: *message_array,
                    output: *output_heap_array,
                });
            } else {
                unreachable!(
                    "ICE: Poseidon2Permutation expects one array argument, a length and one array result"
                )
            }
        }
        BlackBoxFunc::Sha256Compression => {
            if let (
                [
                    BrilligVariable::BrilligArray(input_array),
                    BrilligVariable::BrilligArray(hash_values),
                ],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                let input = brillig_context.codegen_brillig_array_to_heap_array(*input_array);
                let hash_values = brillig_context.codegen_brillig_array_to_heap_array(*hash_values);
                let output = brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Sha256Compression {
                    input: *input,
                    hash_values: *hash_values,
                    output: *output,
                });
            } else {
                unreachable!("ICE: Sha256Compression expects two array argument, one array result")
            }
        }
        BlackBoxFunc::AES128Encrypt => {
            if let (
                [
                    BrilligVariable::BrilligArray(inputs),
                    BrilligVariable::BrilligArray(iv),
                    BrilligVariable::BrilligArray(key),
                ],
                [BrilligVariable::BrilligArray(outputs)],
            ) = (function_arguments, function_results)
            {
                let inputs = brillig_context.codegen_brillig_array_to_heap_array(*inputs);
                let iv = brillig_context.codegen_brillig_array_to_heap_array(*iv);
                let key = brillig_context.codegen_brillig_array_to_heap_array(*key);
                let outputs_array = brillig_context.codegen_brillig_array_to_heap_array(*outputs);

                brillig_context.black_box_op_instruction(BlackBoxOp::AES128Encrypt {
                    inputs: *inputs,
                    iv: *iv,
                    key: *key,
                    outputs: *outputs_array,
                });
            } else {
                unreachable!("ICE: AES128Encrypt expects three array arguments, one array result")
            }
        }
    }
}
