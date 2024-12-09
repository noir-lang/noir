use acvm::{
    acir::{
        brillig::{BlackBoxOp, HeapVector, ValueOrArray},
        BlackBoxFunc,
    },
    AcirField,
};

use crate::brillig::brillig_ir::{
    brillig_variable::BrilligVariable, debug_show::DebugToString, registers::RegisterAllocator,
    BrilligContext,
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
                let message_vector = convert_array_or_vector(brillig_context, *message, bb_func);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Blake2s {
                    message: message_vector,
                    output: output_heap_array,
                });

                brillig_context.deallocate_heap_vector(message_vector);
                brillig_context.deallocate_heap_array(output_heap_array);
            } else {
                unreachable!("ICE: Blake2s expects one array argument and one array result")
            }
        }
        BlackBoxFunc::Blake3 => {
            if let ([message], [BrilligVariable::BrilligArray(result_array)]) =
                (function_arguments, function_results)
            {
                let message_vector = convert_array_or_vector(brillig_context, *message, bb_func);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Blake3 {
                    message: message_vector,
                    output: output_heap_array,
                });

                brillig_context.deallocate_heap_vector(message_vector);
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
                [BrilligVariable::BrilligArray(public_key_x), BrilligVariable::BrilligArray(public_key_y), BrilligVariable::BrilligArray(signature), message],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let hashed_msg = convert_array_or_vector(brillig_context, *message, bb_func);
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

                brillig_context.deallocate_heap_vector(hashed_msg);
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
                [BrilligVariable::BrilligArray(public_key_x), BrilligVariable::BrilligArray(public_key_y), BrilligVariable::BrilligArray(signature), message],
                [BrilligVariable::SingleAddr(result_register)],
            ) = (function_arguments, function_results)
            {
                let hashed_msg = convert_array_or_vector(brillig_context, *message, bb_func);
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

                brillig_context.deallocate_heap_vector(hashed_msg);
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
                let points = convert_array_or_vector(brillig_context, *points, bb_func);
                let scalars = convert_array_or_vector(brillig_context, *scalars, bb_func);
                let outputs = brillig_context.codegen_brillig_array_to_heap_array(*outputs);

                brillig_context.black_box_op_instruction(BlackBoxOp::MultiScalarMul {
                    points,
                    scalars,
                    outputs,
                });
                brillig_context.deallocate_heap_vector(points);
                brillig_context.deallocate_heap_vector(scalars);
                brillig_context.deallocate_heap_array(outputs);
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
        BlackBoxFunc::BigIntAdd => {
            if let (
                [BrilligVariable::SingleAddr(lhs), BrilligVariable::SingleAddr(_lhs_modulus), BrilligVariable::SingleAddr(rhs), BrilligVariable::SingleAddr(_rhs_modulus)],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(_modulus_id)],
            ) = (function_arguments, function_results)
            {
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
                [BrilligVariable::SingleAddr(lhs), BrilligVariable::SingleAddr(_lhs_modulus), BrilligVariable::SingleAddr(rhs), BrilligVariable::SingleAddr(_rhs_modulus)],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(_modulus_id)],
            ) = (function_arguments, function_results)
            {
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
                [BrilligVariable::SingleAddr(lhs), BrilligVariable::SingleAddr(_lhs_modulus), BrilligVariable::SingleAddr(rhs), BrilligVariable::SingleAddr(_rhs_modulus)],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(_modulus_id)],
            ) = (function_arguments, function_results)
            {
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
                [BrilligVariable::SingleAddr(lhs), BrilligVariable::SingleAddr(_lhs_modulus), BrilligVariable::SingleAddr(rhs), BrilligVariable::SingleAddr(_rhs_modulus)],
                [BrilligVariable::SingleAddr(output), BrilligVariable::SingleAddr(_modulus_id)],
            ) = (function_arguments, function_results)
            {
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
                let inputs = convert_array_or_vector(brillig_context, *inputs, bb_func);
                let modulus = convert_array_or_vector(brillig_context, *modulus, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntFromLeBytes {
                    inputs,
                    modulus,
                    output: output.address,
                });
                brillig_context.deallocate_heap_vector(inputs);
                brillig_context.deallocate_heap_vector(modulus);
            } else {
                unreachable!(
                    "ICE: BigIntFromLeBytes expects a register and an array  as arguments and two result registers"
                )
            }
        }
        BlackBoxFunc::BigIntToLeBytes => {
            if let (
                [BrilligVariable::SingleAddr(input), BrilligVariable::SingleAddr(_modulus)],
                [output],
            ) = (function_arguments, function_results)
            {
                let output = convert_array_or_vector(brillig_context, *output, bb_func);
                brillig_context.black_box_op_instruction(BlackBoxOp::BigIntToLeBytes {
                    input: input.address,
                    output,
                });
                brillig_context.deallocate_heap_vector(output);
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
                let message_vector = convert_array_or_vector(brillig_context, *message, bb_func);
                let output_heap_array =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);

                brillig_context.black_box_op_instruction(BlackBoxOp::Poseidon2Permutation {
                    message: message_vector,
                    output: output_heap_array,
                    len: state_len.address,
                });

                brillig_context.deallocate_heap_vector(message_vector);
                brillig_context.deallocate_heap_array(output_heap_array);
            } else {
                unreachable!("ICE: Poseidon2Permutation expects one array argument, a length and one array result")
            }
        }
        BlackBoxFunc::Sha256Compression => {
            if let (
                [BrilligVariable::BrilligArray(input_array), BrilligVariable::BrilligArray(hash_values)],
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
                [BrilligVariable::SingleAddr(out_len), BrilligVariable::BrilligVector(outputs)],
            ) = (function_arguments, function_results)
            {
                let inputs = convert_array_or_vector(brillig_context, *inputs, bb_func);
                let iv = brillig_context.codegen_brillig_array_to_heap_array(*iv);
                let key = brillig_context.codegen_brillig_array_to_heap_array(*key);

                let outputs_vector =
                    brillig_context.codegen_brillig_vector_to_heap_vector(*outputs);

                brillig_context.black_box_op_instruction(BlackBoxOp::AES128Encrypt {
                    inputs,
                    iv,
                    key,
                    outputs: outputs_vector,
                });

                brillig_context.mov_instruction(out_len.address, outputs_vector.size);
                // Returns slice, so we need to allocate memory for it after the fact

                brillig_context.initialize_externally_returned_vector(*outputs, outputs_vector);

                brillig_context.deallocate_heap_vector(inputs);
                brillig_context.deallocate_heap_vector(outputs_vector);
                brillig_context.deallocate_heap_array(iv);
                brillig_context.deallocate_heap_array(key);
            } else {
                unreachable!("ICE: AES128Encrypt expects three array arguments, one array result")
            }
        }
    }
}

fn convert_array_or_vector<F: AcirField + DebugToString, Registers: RegisterAllocator>(
    brillig_context: &mut BrilligContext<F, Registers>,
    array_or_vector: BrilligVariable,
    bb_func: &BlackBoxFunc,
) -> HeapVector {
    let array_or_vector = brillig_context.variable_to_value_or_array(array_or_vector);
    match array_or_vector {
        ValueOrArray::HeapArray(array) => {
            let vector =
                HeapVector { pointer: array.pointer, size: brillig_context.allocate_register() };
            brillig_context.usize_const_instruction(vector.size, array.size.into());
            vector
        }
        ValueOrArray::HeapVector(vector) => vector,
        _ => unreachable!(
            "ICE: {} expected an array or a vector, but got {:?}",
            bb_func.name(),
            array_or_vector
        ),
    }
}
