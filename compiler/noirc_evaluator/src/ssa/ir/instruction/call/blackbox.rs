use std::sync::Arc;

use acvm::{acir::AcirField, BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};

use crate::ssa::ir::instruction::BlackBoxFunc;
use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::{CallStack, DataFlowGraph},
    instruction::{Instruction, Intrinsic, SimplifyResult},
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
    let mut is_constant;

    match (dfg.get_array_constant(arguments[0]), dfg.get_array_constant(arguments[1])) {
        (Some((points, _)), Some((scalars, _))) => {
            // We decompose points and scalars into constant and non-constant parts in order to simplify MSMs where a subset of the terms are constant.
            let mut constant_points = vec![];
            let mut constant_scalars_lo = vec![];
            let mut constant_scalars_hi = vec![];
            let mut var_points = vec![];
            let mut var_scalars = vec![];
            let len = scalars.len() / 2;
            for i in 0..len {
                match (
                    dfg.get_numeric_constant(scalars[2 * i]),
                    dfg.get_numeric_constant(scalars[2 * i + 1]),
                    dfg.get_numeric_constant(points[3 * i]),
                    dfg.get_numeric_constant(points[3 * i + 1]),
                    dfg.get_numeric_constant(points[3 * i + 2]),
                ) {
                    (Some(lo), Some(hi), _, _, _) if lo.is_zero() && hi.is_zero() => {
                        is_constant = true;
                        constant_scalars_lo.push(lo);
                        constant_scalars_hi.push(hi);
                        constant_points.push(FieldElement::zero());
                        constant_points.push(FieldElement::zero());
                        constant_points.push(FieldElement::one());
                    }
                    (_, _, _, _, Some(infinity)) if infinity.is_one() => {
                        is_constant = true;
                        constant_scalars_lo.push(FieldElement::zero());
                        constant_scalars_hi.push(FieldElement::zero());
                        constant_points.push(FieldElement::zero());
                        constant_points.push(FieldElement::zero());
                        constant_points.push(FieldElement::one());
                    }
                    (Some(lo), Some(hi), Some(x), Some(y), Some(infinity)) => {
                        is_constant = true;
                        constant_scalars_lo.push(lo);
                        constant_scalars_hi.push(hi);
                        constant_points.push(x);
                        constant_points.push(y);
                        constant_points.push(infinity);
                    }
                    _ => {
                        is_constant = false;
                    }
                }

                if !is_constant {
                    var_points.push(points[3 * i]);
                    var_points.push(points[3 * i + 1]);
                    var_points.push(points[3 * i + 2]);
                    var_scalars.push(scalars[2 * i]);
                    var_scalars.push(scalars[2 * i + 1]);
                }
            }

            // If there are no constant terms, we can't simplify
            if constant_scalars_lo.is_empty() {
                return SimplifyResult::None;
            }
            let Ok((result_x, result_y, result_is_infinity)) = solver.multi_scalar_mul(
                &constant_points,
                &constant_scalars_lo,
                &constant_scalars_hi,
            ) else {
                return SimplifyResult::None;
            };

            // If there are no variable term, we can directly return the constant result
            if var_scalars.is_empty() {
                let result_x = dfg.make_constant(result_x, Type::field());
                let result_y = dfg.make_constant(result_y, Type::field());
                let result_is_infinity = dfg.make_constant(result_is_infinity, Type::field());

                let elements = im::vector![result_x, result_y, result_is_infinity];
                let typ = Type::Array(Arc::new(vec![Type::field()]), 3);
                let instruction = Instruction::MakeArray { elements, typ };
                let result_array = dfg.insert_instruction_and_results(
                    instruction,
                    block,
                    None,
                    call_stack.clone(),
                );

                return SimplifyResult::SimplifiedTo(result_array.first());
            }
            // If there is only one non-null constant term, we cannot simplify
            if constant_scalars_lo.len() == 1 && result_is_infinity != FieldElement::one() {
                return SimplifyResult::None;
            }
            // Add the constant part back to the non-constant part, if it is not null
            let one = dfg.make_constant(FieldElement::one(), Type::field());
            let zero = dfg.make_constant(FieldElement::zero(), Type::field());
            if result_is_infinity.is_zero() {
                var_scalars.push(one);
                var_scalars.push(zero);
                let result_x = dfg.make_constant(result_x, Type::field());
                let result_y = dfg.make_constant(result_y, Type::field());
                let result_is_infinity = dfg.make_constant(result_is_infinity, Type::bool());
                var_points.push(result_x);
                var_points.push(result_y);
                var_points.push(result_is_infinity);
            }
            // Construct the simplified MSM expression
            let typ = Type::Array(Arc::new(vec![Type::field()]), var_scalars.len() as u32);
            let scalars = Instruction::MakeArray { elements: var_scalars.into(), typ };
            let scalars = dfg
                .insert_instruction_and_results(scalars, block, None, call_stack.clone())
                .first();
            let typ = Type::Array(Arc::new(vec![Type::field()]), var_points.len() as u32);
            let points = Instruction::MakeArray { elements: var_points.into(), typ };
            let points =
                dfg.insert_instruction_and_results(points, block, None, call_stack.clone()).first();
            let msm = dfg.import_intrinsic(Intrinsic::BlackBox(BlackBoxFunc::MultiScalarMul));
            SimplifyResult::SimplifiedToInstruction(Instruction::Call {
                func: msm,
                arguments: vec![points, scalars],
            })
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

#[cfg(feature = "bn254")]
#[cfg(test)]
mod test {
    use crate::ssa::opt::assert_normalized_ssa_equals;
    use crate::ssa::Ssa;

    #[cfg(feature = "bn254")]
    #[test]
    fn full_constant_folding() {
        let src = r#"
            acir(inline) fn main f0 {
            b0():
                v0 = make_array [Field 2, Field 3, Field 5, Field 5] : [Field; 4]
                v1 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860, Field 0, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, Field 0] : [Field; 6]
                v2 = call multi_scalar_mul (v1, v0) -> [Field; 3]
                return v2
            }"#;
        let ssa = Ssa::from_str(src).unwrap();

        let expected_src = r#"
            acir(inline) fn main f0 {
            b0():
                v3 = make_array [Field 2, Field 3, Field 5, Field 5] : [Field; 4]
                v7 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860, Field 0, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, Field 0] : [Field; 6]
                v10 = make_array [Field 1478523918288173385110236399861791147958001875200066088686689589556927843200, Field 700144278551281040379388961242974992655630750193306467120985766322057145630, Field 0] : [Field; 3]
                return v10
            }
            "#;
        assert_normalized_ssa_equals(ssa, expected_src);
    }

    #[cfg(feature = "bn254")]
    #[test]
    fn simplify_zero() {
        let src = r#"
            acir(inline) fn main f0 {
            b0(v0: Field, v1: Field):
                v2 = make_array [v0, Field 0, Field 0, Field 0, v0, Field 0] : [Field; 6]
                v3 = make_array [
                Field 0, Field 0, Field 1, v0, v1, Field 0, Field 1, v0, Field 0] : [Field; 9]
                v4 = call multi_scalar_mul (v3, v2) -> [Field; 3]

                return v4
            
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        //First point is zero, second scalar is zero, so we should be left with the scalar mul of the last point.
        let expected_src = r#"
            acir(inline) fn main f0 {
            b0(v0: Field, v1: Field):
                v3 = make_array [v0, Field 0, Field 0, Field 0, v0, Field 0] : [Field; 6]
                v5 = make_array [Field 0, Field 0, Field 1, v0, v1, Field 0, Field 1, v0, Field 0] : [Field; 9]
                v6 = make_array [v0, Field 0] : [Field; 2]
                v7 = make_array [Field 1, v0, Field 0] : [Field; 3]
                v9 = call multi_scalar_mul(v7, v6) -> [Field; 3]
                return v9
            }
            "#;
        assert_normalized_ssa_equals(ssa, expected_src);
    }

    #[cfg(feature = "bn254")]
    #[test]
    fn partial_constant_folding() {
        let src = r#"
            acir(inline) fn main f0 {
            b0(v0: Field, v1: Field):
                v2 = make_array [Field 1, Field 0, v0, Field 0, Field 2, Field 0] : [Field; 6]
                v3 = make_array [
                Field 1, Field 17631683881184975370165255887551781615748388533673675138860, Field 0, v0, v1, Field 0, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, Field 0] : [Field; 9]
                v4 = call multi_scalar_mul (v3, v2) -> [Field; 3]
                return v4
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        //First and last scalar/point are constant, so we should be left with the msm of the middle point and the folded constant point
        let expected_src = r#"
            acir(inline) fn main f0 {
            b0(v0: Field, v1: Field):
                v5 = make_array [Field 1, Field 0, v0, Field 0, Field 2, Field 0] : [Field; 6]
                v7 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860, Field 0, v0, v1, Field 0, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, Field 0] : [Field; 9]
                v8 = make_array [v0, Field 0, Field 1, Field 0] : [Field; 4]
                v12 = make_array [v0, v1, Field 0, Field -3227352362257037263902424173275354266044964400219754872043023745437788450996, Field 8902249110305491597038405103722863701255802573786510474664632793109847672620, u1 0] : [Field; 6]
                v14 = call multi_scalar_mul(v12, v8) -> [Field; 3]
                return v14
            }
            "#;
        assert_normalized_ssa_equals(ssa, expected_src);
    }
}
