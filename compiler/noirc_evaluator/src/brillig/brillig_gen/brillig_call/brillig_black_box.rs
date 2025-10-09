//! Codegen for native (black box) function calls.
use acvm::{
    AcirField,
    acir::{
        BlackBoxFunc,
        brillig::{BlackBoxOp, HeapArray, ValueOrArray},
    },
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
            if let ([message], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let message_array = get_heap_array(brillig_context, *message, bb_func);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Blake2s {
                    message: message_array,
                    output: output_heap_array,
                });

                brillig_context.deallocate_heap_array(message_array);
                brillig_context.deallocate_heap_array(output_heap_array);
            } else {
                unreachable!("ICE: Blake2s expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Blake3 => {
            if let ([message], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let message_array = get_heap_array(brillig_context, *message, bb_func);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Blake3 {
                    message: message_array,
                    output: output_heap_array,
                });

                brillig_context.deallocate_heap_array(message_array);
                brillig_context.deallocate_heap_array(output_heap_array);
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
                    input: input_heap_array,
                    output: output_heap_array,
                });

                brillig_context.deallocate_heap_array(input_heap_array);
                brillig_context.deallocate_heap_array(output_heap_array);
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
                    message,
                ],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let hashed_msg = get_heap_array(brillig_context, *message, bb_func);
                let public_key_x =
                    brillig_context.codegen_brillig_array_to_heap_array(*public_key_x);
                let public_key_y =
                    brillig_context.codegen_brillig_array_to_heap_array(*public_key_y);
                let signature = brillig_context.codegen_brillig_array_to_heap_array(*signature);

                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256k1 {
                    hashed_msg,
                    public_key_x,
                    public_key_y,
                    signature,
                    result: result_register.address,
                });

                brillig_context.deallocate_heap_array(hashed_msg);
                brillig_context.deallocate_heap_array(public_key_x);
                brillig_context.deallocate_heap_array(public_key_y);
                brillig_context.deallocate_heap_array(signature);
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
                    message,
                ],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let hashed_msg = get_heap_array(brillig_context, *message, bb_func);
                let public_key_x =
                    brillig_context.codegen_brillig_array_to_heap_array(*public_key_x);
                let public_key_y =
                    brillig_context.codegen_brillig_array_to_heap_array(*public_key_y);
                let signature = brillig_context.codegen_brillig_array_to_heap_array(*signature);

                brillig_context.black_box_op_instruction(BlackBoxOp::EcdsaSecp256r1 {
                    hashed_msg,
                    public_key_x,
                    public_key_y,
                    signature,
                    result: result_register.address,
                });

                brillig_context.deallocate_heap_array(hashed_msg);
                brillig_context.deallocate_heap_array(public_key_x);
                brillig_context.deallocate_heap_array(public_key_y);
                brillig_context.deallocate_heap_array(signature);
            } else {
                unreachable!(
                    "ICE: EcdsaSecp256r1 expects four array arguments and one register result"
                )
            }
        }

        BlackBoxFunc::MultiScalarMul => {
            if let ([points, scalars], [BrilligVariable::BrilligArray(outputs)]) =
                (function_arguments, function_results)
            {
                let points = get_heap_array(brillig_context, *points, bb_func);
                let scalars = get_heap_array(brillig_context, *scalars, bb_func);
                let outputs = brillig_context.codegen_brillig_array_to_heap_array(*outputs);

                brillig_context.black_box_op_instruction(BlackBoxOp::MultiScalarMul {
                    points,
                    scalars,
                    outputs,
                });
                brillig_context.deallocate_heap_array(points);
                brillig_context.deallocate_heap_array(scalars);
                brillig_context.deallocate_heap_array(outputs);
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
                    result,
                });
                brillig_context.deallocate_heap_array(result);
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
            if let ([message], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let message_array = get_heap_array(brillig_context, *message, bb_func);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Poseidon2Permutation {
                    message: message_array,
                    output: output_heap_array,
                });

                brillig_context.deallocate_heap_array(message_array);
                brillig_context.deallocate_heap_array(output_heap_array);
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
                    input,
                    hash_values,
                    output,
                });

                brillig_context.deallocate_heap_array(input);
                brillig_context.deallocate_heap_array(hash_values);
                brillig_context.deallocate_heap_array(output);
            } else {
                unreachable!("ICE: Sha256Compression expects two array argument, one array result")
            }
        }
        BlackBoxFunc::AES128Encrypt => {
            if let (
                [inputs, BrilligVariable::BrilligArray(iv), BrilligVariable::BrilligArray(key)],
                [outputs],
            ) = (function_arguments, function_results)
            {
                let inputs = get_heap_array(brillig_context, *inputs, bb_func);
                let iv = brillig_context.codegen_brillig_array_to_heap_array(*iv);
                let key = brillig_context.codegen_brillig_array_to_heap_array(*key);

                let outputs_array = get_heap_array(brillig_context, *outputs, bb_func);

                brillig_context.black_box_op_instruction(BlackBoxOp::AES128Encrypt {
                    inputs,
                    iv,
                    key,
                    outputs: outputs_array,
                });

                brillig_context.deallocate_heap_array(inputs);
                brillig_context.deallocate_heap_array(outputs_array);
                brillig_context.deallocate_heap_array(iv);
                brillig_context.deallocate_heap_array(key);
            } else {
                unreachable!("ICE: AES128Encrypt expects three array arguments, one array result")
            }
        }
    }
}

/// Converts a [BrilligVariable] into a heap-allocated [HeapArray]
/// suitable for use as an input to a Brillig [BlackBoxOp].
/// 
/// # Panics
/// If the input is not a [BrilligVariable::BrilligArray]
fn get_heap_array<F: AcirField + DebugToString, Registers: RegisterAllocator>(
    brillig_context: &mut BrilligContext<F, Registers>,
    array: BrilligVariable,
    bb_func: &BlackBoxFunc,
) -> HeapArray {
    let array_or_array = brillig_context.variable_to_value_or_array(array);
    match array_or_array {
        ValueOrArray::HeapArray(array) => array,
        _ => {
            unreachable!("ICE: {} expected an array, but got {:?}", bb_func.name(), array_or_array)
        }
    }
}
