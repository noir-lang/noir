use std::sync::Arc;

use acvm::{acir::AcirField, BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::{CallStack, DataFlowGraph},
    instruction::{Instruction, SimplifyResult},
    types::Type,
    value::ValueId,
};

use super::{array_is_constant, make_constant_array, to_u8_vec};

pub(super) fn simplify_ec_add(
    dfg: &mut DataFlowGraph,
    solver: impl BlackBoxFunctionSolver<FieldElement>,
    arguments: &[ValueId],
    block: BasicBlockId,
    call_stack: &CallStack,
) -> SimplifyResult {
    match (
        dfg.get_numeric_constant(arguments[0]),
        dfg.get_numeric_constant(arguments[1]),
        dfg.get_numeric_constant(arguments[2]),
        dfg.get_numeric_constant(arguments[3]),
        dfg.get_numeric_constant(arguments[4]),
        dfg.get_numeric_constant(arguments[5]),
    ) {
        (
            Some(point1_x),
            Some(point1_y),
            Some(point1_is_infinity),
            Some(point2_x),
            Some(point2_y),
            Some(point2_is_infinity),
        ) => {
            let Ok((result_x, result_y, result_is_infinity)) = solver.ec_add(
                &point1_x,
                &point1_y,
                &point1_is_infinity,
                &point2_x,
                &point2_y,
                &point2_is_infinity,
            ) else {
                return SimplifyResult::None;
            };

            let result_x = dfg.make_constant(result_x, Type::field());
            let result_y = dfg.make_constant(result_y, Type::field());
            let result_is_infinity = dfg.make_constant(result_is_infinity, Type::field());

            let typ = Type::Array(Arc::new(vec![Type::field()]), 3);

            let elements = im::vector![result_x, result_y, result_is_infinity];
            let instruction = Instruction::MakeArray { elements, typ };
            let result_array =
                dfg.insert_instruction_and_results(instruction, block, None, call_stack.clone());

            SimplifyResult::SimplifiedTo(result_array.first())
        }
        _ => SimplifyResult::None,
    }
}

pub(super) fn simplify_msm(
    dfg: &mut DataFlowGraph,
    solver: impl BlackBoxFunctionSolver<FieldElement>,
    arguments: &[ValueId],
    block: BasicBlockId,
    call_stack: &CallStack,
) -> SimplifyResult {
    // TODO: Handle MSMs where a subset of the terms are constant.
    match (dfg.get_array_constant(arguments[0]), dfg.get_array_constant(arguments[1])) {
        (Some((points, _)), Some((scalars, _))) => {
            let Some(points) = points
                .into_iter()
                .map(|id| dfg.get_numeric_constant(id))
                .collect::<Option<Vec<_>>>()
            else {
                return SimplifyResult::None;
            };

            let Some(scalars) = scalars
                .into_iter()
                .map(|id| dfg.get_numeric_constant(id))
                .collect::<Option<Vec<_>>>()
            else {
                return SimplifyResult::None;
            };

            let mut scalars_lo = Vec::new();
            let mut scalars_hi = Vec::new();
            for (i, scalar) in scalars.into_iter().enumerate() {
                if i % 2 == 0 {
                    scalars_lo.push(scalar);
                } else {
                    scalars_hi.push(scalar);
                }
            }

            let Ok((result_x, result_y, result_is_infinity)) =
                solver.multi_scalar_mul(&points, &scalars_lo, &scalars_hi)
            else {
                return SimplifyResult::None;
            };

            let result_x = dfg.make_constant(result_x, Type::field());
            let result_y = dfg.make_constant(result_y, Type::field());
            let result_is_infinity = dfg.make_constant(result_is_infinity, Type::field());

            let elements = im::vector![result_x, result_y, result_is_infinity];
            let typ = Type::Array(Arc::new(vec![Type::field()]), 3);
            let instruction = Instruction::MakeArray { elements, typ };
            let result_array =
                dfg.insert_instruction_and_results(instruction, block, None, call_stack.clone());

            SimplifyResult::SimplifiedTo(result_array.first())
        }
        _ => SimplifyResult::None,
    }
}

pub(super) fn simplify_poseidon2_permutation(
    dfg: &mut DataFlowGraph,
    solver: impl BlackBoxFunctionSolver<FieldElement>,
    arguments: &[ValueId],
    block: BasicBlockId,
    call_stack: &CallStack,
) -> SimplifyResult {
    match (dfg.get_array_constant(arguments[0]), dfg.get_numeric_constant(arguments[1])) {
        (Some((state, _)), Some(state_length)) if array_is_constant(dfg, &state) => {
            let state: Vec<FieldElement> = state
                .iter()
                .map(|id| {
                    dfg.get_numeric_constant(*id)
                        .expect("value id from array should point at constant")
                })
                .collect();

            let Some(state_length) = state_length.try_to_u32() else {
                return SimplifyResult::None;
            };

            let Ok(new_state) = solver.poseidon2_permutation(&state, state_length) else {
                return SimplifyResult::None;
            };

            let new_state = new_state.into_iter();
            let typ = Type::field();
            let result_array = make_constant_array(dfg, new_state, typ, block, call_stack);

            SimplifyResult::SimplifiedTo(result_array)
        }
        _ => SimplifyResult::None,
    }
}

pub(super) fn simplify_schnorr_verify(
    dfg: &mut DataFlowGraph,
    solver: impl BlackBoxFunctionSolver<FieldElement>,
    arguments: &[ValueId],
) -> SimplifyResult {
    match (
        dfg.get_numeric_constant(arguments[0]),
        dfg.get_numeric_constant(arguments[1]),
        dfg.get_array_constant(arguments[2]),
        dfg.get_array_constant(arguments[3]),
    ) {
        (Some(public_key_x), Some(public_key_y), Some((signature, _)), Some((message, _)))
            if array_is_constant(dfg, &signature) && array_is_constant(dfg, &message) =>
        {
            let signature = to_u8_vec(dfg, signature);
            let signature: [u8; 64] =
                signature.try_into().expect("Compiler should produce correctly sized signature");

            let message = to_u8_vec(dfg, message);

            let Ok(valid_signature) =
                solver.schnorr_verify(&public_key_x, &public_key_y, &signature, &message)
            else {
                return SimplifyResult::None;
            };

            let valid_signature = dfg.make_constant(valid_signature.into(), Type::bool());
            SimplifyResult::SimplifiedTo(valid_signature)
        }
        _ => SimplifyResult::None,
    }
}

pub(super) fn simplify_hash(
    dfg: &mut DataFlowGraph,
    arguments: &[ValueId],
    hash_function: fn(&[u8]) -> Result<[u8; 32], BlackBoxResolutionError>,
    block: BasicBlockId,
    call_stack: &CallStack,
) -> SimplifyResult {
    match dfg.get_array_constant(arguments[0]) {
        Some((input, _)) if array_is_constant(dfg, &input) => {
            let input_bytes: Vec<u8> = to_u8_vec(dfg, input);

            let hash = hash_function(&input_bytes)
                .expect("Rust solvable black box function should not fail");

            let hash_values = hash.iter().map(|byte| FieldElement::from_be_bytes_reduce(&[*byte]));

            let u8_type = Type::unsigned(8);
            let result_array = make_constant_array(dfg, hash_values, u8_type, block, call_stack);
            SimplifyResult::SimplifiedTo(result_array)
        }
        _ => SimplifyResult::None,
    }
}

type ECDSASignatureVerifier = fn(
    hashed_msg: &[u8],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError>;

pub(super) fn simplify_signature(
    dfg: &mut DataFlowGraph,
    arguments: &[ValueId],
    signature_verifier: ECDSASignatureVerifier,
) -> SimplifyResult {
    match (
        dfg.get_array_constant(arguments[0]),
        dfg.get_array_constant(arguments[1]),
        dfg.get_array_constant(arguments[2]),
        dfg.get_array_constant(arguments[3]),
    ) {
        (
            Some((public_key_x, _)),
            Some((public_key_y, _)),
            Some((signature, _)),
            Some((hashed_message, _)),
        ) if array_is_constant(dfg, &public_key_x)
            && array_is_constant(dfg, &public_key_y)
            && array_is_constant(dfg, &signature)
            && array_is_constant(dfg, &hashed_message) =>
        {
            let public_key_x: [u8; 32] = to_u8_vec(dfg, public_key_x)
                .try_into()
                .expect("ECDSA public key fields are 32 bytes");
            let public_key_y: [u8; 32] = to_u8_vec(dfg, public_key_y)
                .try_into()
                .expect("ECDSA public key fields are 32 bytes");
            let signature: [u8; 64] =
                to_u8_vec(dfg, signature).try_into().expect("ECDSA signatures are 64 bytes");
            let hashed_message: Vec<u8> = to_u8_vec(dfg, hashed_message);

            let valid_signature =
                signature_verifier(&hashed_message, &public_key_x, &public_key_y, &signature)
                    .expect("Rust solvable black box function should not fail");

            let valid_signature = dfg.make_constant(valid_signature.into(), Type::bool());
            SimplifyResult::SimplifiedTo(valid_signature)
        }
        _ => SimplifyResult::None,
    }
}
