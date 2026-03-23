//! Codegen for native (black box) function calls.
use acvm::{
    AcirField,
    acir::{BlackBoxFunc, brillig::BlackBoxOp},
};

use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext,
    brillig_variable::{BrilligVariable, SingleAddrVariable},
    debug_show::DebugToString,
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
                // SSA points array has 2N elements (x, y per point).
                // BlackBoxOp expects 3N elements (x, y, is_infinite per point).
                let points_heap = brillig_context.codegen_brillig_array_to_heap_array(*points);
                let scalars = brillig_context.codegen_brillig_array_to_heap_array(*scalars);

                // Compute the number of points: points array size is 2N (SemiFlattenedLength)
                let two_n = points.size;
                // Allocate expanded 3N-element array
                let expanded_size =
                    acvm::acir::brillig::lengths::SemiFlattenedLength(two_n.0 / 2 * 3);
                let expanded_points = brillig_context.allocate_heap_array(expanded_size);
                // Allocate memory for expanded points
                let expanded_size_reg = brillig_context
                    .make_usize_constant_instruction(F::from(u128::from(expanded_size.0)));
                brillig_context
                    .codegen_allocate_mem(expanded_points.pointer, expanded_size_reg.address);

                // Zero constant for is_infinite computation
                let zero_const =
                    brillig_context.make_constant_instruction(F::zero(), F::max_num_bits());
                // Keep Allocated guards alive so registers don't get reused
                let _temp_x_alloc = brillig_context.allocate_register();
                let temp_x = *_temp_x_alloc;
                let _temp_y_alloc = brillig_context.allocate_register();
                let temp_y = *_temp_y_alloc;
                let _x_is_zero_alloc = brillig_context.allocate_register();
                let x_is_zero = *_x_is_zero_alloc;
                let _y_is_zero_alloc = brillig_context.allocate_register();
                let y_is_zero = *_y_is_zero_alloc;
                let _is_infinite_alloc = brillig_context.allocate_register();
                let is_infinite_reg = *_is_infinite_alloc;
                // Iterate through points: for each (x, y) pair, write (x, y, is_infinite)
                let n_points = two_n.0 / 2;
                for i in 0..n_points {
                    let src_offset =
                        brillig_context.make_usize_constant_instruction(F::from(u128::from(i * 2)));
                    let dst_offset =
                        brillig_context.make_usize_constant_instruction(F::from(u128::from(i * 3)));

                    // Load x
                    brillig_context.codegen_load_with_offset(
                        points_heap.pointer,
                        *src_offset,
                        temp_x,
                    );
                    // Store x
                    brillig_context.codegen_store_with_offset(
                        expanded_points.pointer,
                        *dst_offset,
                        temp_x,
                    );

                    // Load y
                    let src_y_offset = brillig_context
                        .make_usize_constant_instruction(F::from(u128::from(i * 2 + 1)));
                    brillig_context.codegen_load_with_offset(
                        points_heap.pointer,
                        *src_y_offset,
                        temp_y,
                    );
                    // Store y
                    let dst_y_offset = brillig_context
                        .make_usize_constant_instruction(F::from(u128::from(i * 3 + 1)));
                    brillig_context.codegen_store_with_offset(
                        expanded_points.pointer,
                        *dst_y_offset,
                        temp_y,
                    );

                    // Compute is_infinite = (x == 0) & (y == 0)
                    let x_var = SingleAddrVariable::new(temp_x, F::max_num_bits());
                    let y_var = SingleAddrVariable::new(temp_y, F::max_num_bits());
                    let zero_var = SingleAddrVariable::new(zero_const.address, F::max_num_bits());
                    let x_eq_zero = SingleAddrVariable::new(x_is_zero, 1);
                    let y_eq_zero = SingleAddrVariable::new(y_is_zero, 1);
                    let is_inf_var = SingleAddrVariable::new(is_infinite_reg, 1);

                    brillig_context.binary_instruction(
                        x_var,
                        zero_var,
                        x_eq_zero,
                        BrilligBinaryOp::Equals,
                    );
                    brillig_context.binary_instruction(
                        y_var,
                        zero_var,
                        y_eq_zero,
                        BrilligBinaryOp::Equals,
                    );
                    brillig_context.binary_instruction(
                        x_eq_zero,
                        y_eq_zero,
                        is_inf_var,
                        BrilligBinaryOp::And,
                    );

                    // Store is_infinite
                    let dst_inf_offset = brillig_context
                        .make_usize_constant_instruction(F::from(u128::from(i * 3 + 2)));
                    brillig_context.codegen_store_with_offset(
                        expanded_points.pointer,
                        *dst_inf_offset,
                        is_infinite_reg,
                    );
                }

                // Allocate temp 3-element output array for blackbox result
                let temp_output_size = acvm::acir::brillig::lengths::SemiFlattenedLength(3);
                let temp_output = brillig_context.allocate_heap_array(temp_output_size);
                let temp_output_size_reg =
                    brillig_context.make_usize_constant_instruction(F::from(3_u128));
                brillig_context
                    .codegen_allocate_mem(temp_output.pointer, temp_output_size_reg.address);

                brillig_context.black_box_op_instruction(BlackBoxOp::MultiScalarMul {
                    points: *expanded_points,
                    scalars: *scalars,
                    outputs: *temp_output,
                });

                // Copy first 2 elements of temp output to actual output
                let outputs_heap = brillig_context.codegen_brillig_array_to_heap_array(*outputs);
                let zero_idx = brillig_context.make_usize_constant_instruction(F::zero());
                let one_idx = brillig_context.make_usize_constant_instruction(F::one());
                let _temp_val_alloc = brillig_context.allocate_register();
                let temp_val = *_temp_val_alloc;
                brillig_context.codegen_load_with_offset(temp_output.pointer, *zero_idx, temp_val);
                brillig_context.codegen_store_with_offset(
                    outputs_heap.pointer,
                    *zero_idx,
                    temp_val,
                );
                brillig_context.codegen_load_with_offset(temp_output.pointer, *one_idx, temp_val);
                brillig_context.codegen_store_with_offset(outputs_heap.pointer, *one_idx, temp_val);
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
                    BrilligVariable::SingleAddr(input2_x),
                    BrilligVariable::SingleAddr(input2_y),
                ],
                [BrilligVariable::BrilligArray(result_array)],
            ) = (function_arguments, function_results)
            {
                // Compute is_infinite for each point: (x == 0) & (y == 0)
                let zero_const =
                    brillig_context.make_constant_instruction(F::zero(), F::max_num_bits());
                let zero_var = SingleAddrVariable::new(zero_const.address, F::max_num_bits());

                // Keep Allocated guards alive so registers don't get reused
                let _x1_eq_zero_alloc = brillig_context.allocate_single_addr(1);
                let x1_eq_zero = *_x1_eq_zero_alloc;
                let _y1_eq_zero_alloc = brillig_context.allocate_single_addr(1);
                let y1_eq_zero = *_y1_eq_zero_alloc;
                let _is_inf1_alloc = brillig_context.allocate_single_addr(1);
                let is_inf1 = *_is_inf1_alloc;
                brillig_context.binary_instruction(
                    *input1_x,
                    zero_var,
                    x1_eq_zero,
                    BrilligBinaryOp::Equals,
                );
                brillig_context.binary_instruction(
                    *input1_y,
                    zero_var,
                    y1_eq_zero,
                    BrilligBinaryOp::Equals,
                );
                brillig_context.binary_instruction(
                    x1_eq_zero,
                    y1_eq_zero,
                    is_inf1,
                    BrilligBinaryOp::And,
                );

                let _x2_eq_zero_alloc = brillig_context.allocate_single_addr(1);
                let x2_eq_zero = *_x2_eq_zero_alloc;
                let _y2_eq_zero_alloc = brillig_context.allocate_single_addr(1);
                let y2_eq_zero = *_y2_eq_zero_alloc;
                let _is_inf2_alloc = brillig_context.allocate_single_addr(1);
                let is_inf2 = *_is_inf2_alloc;
                brillig_context.binary_instruction(
                    *input2_x,
                    zero_var,
                    x2_eq_zero,
                    BrilligBinaryOp::Equals,
                );
                brillig_context.binary_instruction(
                    *input2_y,
                    zero_var,
                    y2_eq_zero,
                    BrilligBinaryOp::Equals,
                );
                brillig_context.binary_instruction(
                    x2_eq_zero,
                    y2_eq_zero,
                    is_inf2,
                    BrilligBinaryOp::And,
                );

                // Allocate temp 3-element output array for blackbox result
                let temp_output_size = acvm::acir::brillig::lengths::SemiFlattenedLength(3);
                let temp_output = brillig_context.allocate_heap_array(temp_output_size);
                let temp_output_size_reg =
                    brillig_context.make_usize_constant_instruction(F::from(3_u128));
                brillig_context
                    .codegen_allocate_mem(temp_output.pointer, temp_output_size_reg.address);

                brillig_context.black_box_op_instruction(BlackBoxOp::EmbeddedCurveAdd {
                    input1_x: input1_x.address,
                    input1_y: input1_y.address,
                    input1_infinite: is_inf1.address,
                    input2_x: input2_x.address,
                    input2_y: input2_y.address,
                    input2_infinite: is_inf2.address,
                    result: *temp_output,
                });

                // Copy first 2 elements of temp output to actual output
                let result_heap =
                    brillig_context.codegen_brillig_array_to_heap_array(*result_array);
                let zero_idx = brillig_context.make_usize_constant_instruction(F::zero());
                let one_idx = brillig_context.make_usize_constant_instruction(F::one());
                let _temp_val_alloc = brillig_context.allocate_register();
                let temp_val = *_temp_val_alloc;
                brillig_context.codegen_load_with_offset(temp_output.pointer, *zero_idx, temp_val);
                brillig_context.codegen_store_with_offset(result_heap.pointer, *zero_idx, temp_val);
                brillig_context.codegen_load_with_offset(temp_output.pointer, *one_idx, temp_val);
                brillig_context.codegen_store_with_offset(result_heap.pointer, *one_idx, temp_val);
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
        BlackBoxFunc::RecursiveAggregation => unreachable!(
            "ICE: `BlackBoxFunc::RecursiveAggregation` calls are disallowed in Brillig"
        ),
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
