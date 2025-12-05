use std::sync::Arc;

use acvm::acir::BlackBoxFunc;
use acvm::blackbox_solver::sha256_compression;
use acvm::{BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement, acir::AcirField};
use im::Vector;
use noirc_errors::call_stack::CallStackId;

use crate::ssa::ir::types::NumericType;
use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    instruction::{Instruction, Intrinsic},
    types::Type,
    value::ValueId,
};

use super::{SimplifyResult, array_is_constant, make_constant_array, to_u8_vec};

pub(super) fn simplify_ec_add(
    dfg: &mut DataFlowGraph,
    solver: impl BlackBoxFunctionSolver<FieldElement>,
    arguments: &[ValueId],
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    let points = Vector::from(vec![
        arguments[0],
        arguments[1],
        arguments[2],
        arguments[3],
        arguments[4],
        arguments[5],
    ]);
    let zero = dfg.make_constant(FieldElement::zero(), NumericType::NativeField);
    let one = dfg.make_constant(FieldElement::one(), NumericType::NativeField);
    let scalars = Vector::from(vec![one, zero, one, zero]);
    simplify_msm_helper(dfg, solver, &points, &scalars, &arguments[6], block, call_stack)
}

fn simplify_msm_helper(
    dfg: &mut DataFlowGraph,
    solver: impl BlackBoxFunctionSolver<FieldElement>,
    points: &Vector<ValueId>,
    scalars: &Vector<ValueId>,
    predicate: &ValueId,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
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
                constant_scalars_lo.push(lo);
                constant_scalars_hi.push(hi);
                constant_points.push(FieldElement::zero());
                constant_points.push(FieldElement::zero());
                constant_points.push(FieldElement::one());
            }
            (_, _, _, _, Some(infinity)) if infinity.is_one() => {
                constant_scalars_lo.push(FieldElement::zero());
                constant_scalars_hi.push(FieldElement::zero());
                constant_points.push(FieldElement::zero());
                constant_points.push(FieldElement::zero());
                constant_points.push(FieldElement::one());
            }
            (Some(lo), Some(hi), Some(x), Some(y), Some(infinity)) => {
                constant_scalars_lo.push(lo);
                constant_scalars_hi.push(hi);
                constant_points.push(x);
                constant_points.push(y);
                constant_points.push(infinity);
            }
            _ => {
                var_points.push(points[3 * i]);
                var_points.push(points[3 * i + 1]);
                var_points.push(points[3 * i + 2]);
                var_scalars.push(scalars[2 * i]);
                var_scalars.push(scalars[2 * i + 1]);
            }
        }
    }

    // If there are no constant terms, we can't simplify
    if constant_scalars_lo.is_empty() {
        return SimplifyResult::None;
    }
    let Ok((result_x, result_y, result_is_infinity)) =
        solver.multi_scalar_mul(&constant_points, &constant_scalars_lo, &constant_scalars_hi, true)
    else {
        return SimplifyResult::None;
    };

    // If there are no variable term, we can directly return the constant result
    if var_scalars.is_empty() {
        let result_x = dfg.make_constant(result_x, NumericType::NativeField);
        let result_y = dfg.make_constant(result_y, NumericType::NativeField);
        let result_is_infinity = dfg.make_constant(result_is_infinity, NumericType::bool());

        let elements = im::vector![result_x, result_y, result_is_infinity];
        let typ = Type::Array(Arc::new(vec![Type::field(), Type::field(), Type::bool()]), 1);
        let instruction = Instruction::MakeArray { elements, typ };
        let result_array = dfg.insert_instruction_and_results(instruction, block, None, call_stack);

        return SimplifyResult::SimplifiedTo(result_array.first());
    }
    // If there is only one non-null constant term, we cannot simplify
    if constant_scalars_lo.len() == 1 && result_is_infinity != FieldElement::one() {
        return SimplifyResult::None;
    }

    // Add the constant part back to the non-constant part, if it is not null
    let one = dfg.make_constant(FieldElement::one(), NumericType::NativeField);
    let zero = dfg.make_constant(FieldElement::zero(), NumericType::NativeField);
    if result_is_infinity.is_zero() {
        var_scalars.push(one);
        var_scalars.push(zero);
        let result_x = dfg.make_constant(result_x, NumericType::NativeField);
        let result_y = dfg.make_constant(result_y, NumericType::NativeField);

        // Pushing a bool here is intentional, multi_scalar_mul takes two arguments:
        // `points: [(Field, Field, bool); N]` and `scalars: [(Field, Field); N]`.
        let result_is_infinity = dfg.make_constant(result_is_infinity, NumericType::bool());

        var_points.push(result_x);
        var_points.push(result_y);
        var_points.push(result_is_infinity);
    }
    // Construct the simplified MSM expression
    let typ =
        Type::Array(Arc::new(vec![Type::field(), Type::field()]), var_scalars.len() as u32 / 2);
    let scalars = Instruction::MakeArray { elements: var_scalars.into(), typ };
    let scalars = dfg.insert_instruction_and_results(scalars, block, None, call_stack).first();
    let typ = Type::Array(
        Arc::new(vec![Type::field(), Type::field(), Type::bool()]),
        var_points.len() as u32 / 3,
    );
    let points = Instruction::MakeArray { elements: var_points.into(), typ };
    let points = dfg.insert_instruction_and_results(points, block, None, call_stack).first();
    let msm = dfg.import_intrinsic(Intrinsic::BlackBox(BlackBoxFunc::MultiScalarMul));
    SimplifyResult::SimplifiedToInstruction(Instruction::Call {
        func: msm,
        arguments: vec![points, scalars, *predicate],
    })
}
pub(super) fn simplify_msm(
    dfg: &mut DataFlowGraph,
    solver: impl BlackBoxFunctionSolver<FieldElement>,
    arguments: &[ValueId],
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    let predicate = arguments[2];
    match (dfg.get_array_constant(arguments[0]), dfg.get_array_constant(arguments[1])) {
        (Some((points, _)), Some((scalars, _))) => {
            simplify_msm_helper(dfg, solver, &points, &scalars, &predicate, block, call_stack)
        }
        _ => SimplifyResult::None,
    }
}

pub(super) fn simplify_poseidon2_permutation(
    dfg: &mut DataFlowGraph,
    solver: impl BlackBoxFunctionSolver<FieldElement>,
    arguments: &[ValueId],
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    match dfg.get_array_constant(arguments[0]) {
        Some((state, _)) if array_is_constant(dfg, &state) => {
            let state: Vec<FieldElement> = state
                .iter()
                .map(|id| {
                    dfg.get_numeric_constant(*id)
                        .expect("value id from array should point at constant")
                })
                .collect();

            let Ok(new_state) = solver.poseidon2_permutation(&state) else {
                return SimplifyResult::None;
            };

            let new_state = new_state.into_iter();
            let typ = NumericType::NativeField;
            let result_array = make_constant_array(dfg, new_state, typ, block, call_stack);

            SimplifyResult::SimplifiedTo(result_array)
        }
        _ => SimplifyResult::None,
    }
}

pub(super) fn simplify_sha256_compression(
    dfg: &mut DataFlowGraph,
    arguments: &[ValueId],
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    match (dfg.get_array_constant(arguments[0]), dfg.get_array_constant(arguments[1])) {
        (Some((state, _)), Some((msg_blocks, _)))
            if array_is_constant(dfg, &state) && array_is_constant(dfg, &msg_blocks) =>
        {
            let state: Option<Vec<u32>> = state
                .iter()
                .map(|id| {
                    dfg.get_numeric_constant(*id)
                        .expect("value id from array should point at constant")
                        .try_to_u32()
                })
                .collect();

            let Some(mut state) = state.and_then(|vec| <[u32; 8]>::try_from(vec).ok()) else {
                return SimplifyResult::None;
            };

            let msg_blocks: Option<Vec<u32>> = msg_blocks
                .iter()
                .map(|id| {
                    dfg.get_numeric_constant(*id)
                        .expect("value id from array should point at constant")
                        .try_to_u32()
                })
                .collect();

            let Some(msg_blocks) = msg_blocks.and_then(|vec| <[u32; 16]>::try_from(vec).ok())
            else {
                return SimplifyResult::None;
            };

            sha256_compression(&mut state, &msg_blocks);

            let new_state = state.into_iter().map(FieldElement::from);
            let typ = NumericType::Unsigned { bit_size: 32 };
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
    call_stack: CallStackId,
) -> SimplifyResult {
    match dfg.get_array_constant(arguments[0]) {
        Some((input, _)) if array_is_constant(dfg, &input) => {
            let input_bytes: Vec<u8> = to_u8_vec(dfg, input);

            let hash = hash_function(&input_bytes)
                .expect("Rust solvable black box function should not fail");

            let hash_values = hash.iter().map(|byte| FieldElement::from_be_bytes_reduce(&[*byte]));

            let u8_type = NumericType::Unsigned { bit_size: 8 };
            let result_array = make_constant_array(dfg, hash_values, u8_type, block, call_stack);
            SimplifyResult::SimplifiedTo(result_array)
        }
        _ => SimplifyResult::None,
    }
}

type ECDSASignatureVerifier = fn(
    hashed_msg: &[u8; 32],
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
        dfg.get_array_constant(arguments[4]),
    ) {
        (
            Some((public_key_x, _)),
            Some((public_key_y, _)),
            Some((signature, _)),
            Some((hashed_message, _)),
            Some((predicate, _)),
        ) if array_is_constant(dfg, &public_key_x)
            && array_is_constant(dfg, &public_key_y)
            && array_is_constant(dfg, &signature)
            && array_is_constant(dfg, &hashed_message) =>
        {
            if dfg.get_numeric_constant(predicate[0]) == Some(FieldElement::zero()) {
                let valid_signature = dfg.make_constant(1_u128.into(), NumericType::bool());
                return SimplifyResult::SimplifiedTo(valid_signature);
            }
            let public_key_x: [u8; 32] = to_u8_vec(dfg, public_key_x)
                .try_into()
                .expect("ECDSA public key fields are 32 bytes");
            let public_key_y: [u8; 32] = to_u8_vec(dfg, public_key_y)
                .try_into()
                .expect("ECDSA public key fields are 32 bytes");
            let signature: [u8; 64] =
                to_u8_vec(dfg, signature).try_into().expect("ECDSA signatures are 64 bytes");
            let hashed_message: [u8; 32] = to_u8_vec(dfg, hashed_message)
                .try_into()
                .expect("ECDSA message hashes are 32 bytes");

            let valid_signature =
                signature_verifier(&hashed_message, &public_key_x, &public_key_y, &signature)
                    .expect("Rust solvable black box function should not fail");

            let valid_signature = dfg.make_constant(valid_signature.into(), NumericType::bool());
            SimplifyResult::SimplifiedTo(valid_signature)
        }
        _ => SimplifyResult::None,
    }
}

#[cfg(test)]
mod embedded_curve_add {
    use crate::assert_ssa_snapshot;
    use crate::ssa::Ssa;

    #[test]
    fn one_constant_argument_is_not_simplified() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: Field, v1: Field, v2: u1):
                v3 = call embedded_curve_add (v0, v1, v2, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 1, u1 1) -> [(Field, Field, u1); 1]
                return v3
            }"#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            v7 = call embedded_curve_add(v0, v1, v2, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 1, u1 1) -> [(Field, Field, u1); 1]
            return v7
        }
        ");
    }
}

#[cfg(test)]
mod multi_scalar_mul {
    use crate::assert_ssa_snapshot;
    use crate::ssa::Ssa;

    #[test]
    fn full_constant_folding() {
        let src = r#"
            acir(inline) fn main f0 {
              b0():
                v0 = make_array [Field 2, Field 3, Field 5, Field 5] : [(Field, Field); 2]
                v1 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0] : [(Field, Field, u1); 2]
                v2 = call multi_scalar_mul (v1, v0, u1 1) -> [(Field, Field, u1); 1]
                return v2
            }"#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 2, Field 3, Field 5, Field 5] : [(Field, Field); 2]
            v7 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0] : [(Field, Field, u1); 2]
            v10 = make_array [Field 1478523918288173385110236399861791147958001875200066088686689589556927843200, Field 700144278551281040379388961242974992655630750193306467120985766322057145630, u1 0] : [(Field, Field, u1); 1]
            return v10
        }
        ");
    }

    #[test]
    fn simplify_zero() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: Field, v1: Field):
                v2 = make_array [v0, Field 0, Field 0, Field 0, v0, Field 0] : [(Field, Field); 3]
                v3 = make_array [
                Field 0, Field 0, u1 1, v0, v1, u1 0, Field 1, v0, u1 0] : [(Field, Field, u1); 3]
                v4 = call multi_scalar_mul (v3, v2, u1 1) -> [(Field, Field, u1); 1]

                return v4
            
            }"#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        //First point is zero, second scalar is zero, so we should be left with the scalar mul of the last point.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = make_array [v0, Field 0, Field 0, Field 0, v0, Field 0] : [(Field, Field); 3]
            v7 = make_array [Field 0, Field 0, u1 1, v0, v1, u1 0, Field 1, v0, u1 0] : [(Field, Field, u1); 3]
            v8 = make_array [v0, Field 0] : [(Field, Field); 1]
            v9 = make_array [Field 1, v0, u1 0] : [(Field, Field, u1); 1]
            v11 = call multi_scalar_mul(v9, v8, u1 1) -> [(Field, Field, u1); 1]
            return v11
        }
        ");
    }

    #[test]
    fn partial_constant_folding() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: Field, v1: Field):
                v2 = make_array [Field 1, Field 0, v0, Field 0, Field 2, Field 0] : [(Field, Field); 3]
                v3 = make_array [
                Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0, v0, v1, u1 0, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0] : [(Field, Field, u1); 3]
                v4 = call multi_scalar_mul (v3, v2, u1 1) -> [(Field, Field, u1); 1]
                return v4
            }"#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        //First and last scalar/point are constant, so we should be left with the msm of the middle point and the folded constant point
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v5 = make_array [Field 1, Field 0, v0, Field 0, Field 2, Field 0] : [(Field, Field); 3]
            v8 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0, v0, v1, u1 0, Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0] : [(Field, Field, u1); 3]
            v9 = make_array [v0, Field 0, Field 1, Field 0] : [(Field, Field); 2]
            v12 = make_array [v0, v1, u1 0, Field -3227352362257037263902424173275354266044964400219754872043023745437788450996, Field 8902249110305491597038405103722863701255802573786510474664632793109847672620, u1 0] : [(Field, Field, u1); 2]
            v15 = call multi_scalar_mul(v12, v9, u1 1) -> [(Field, Field, u1); 1]
            return v15
        }
        ");
    }
}

#[cfg(test)]
mod sha256_compression {
    use crate::assert_ssa_snapshot;
    use crate::ssa::Ssa;

    #[test]
    fn is_optimized_out_with_constant_arguments() {
        let src = r#"
            acir(inline) fn main f0 {
              b0():
                v0 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 8]
                v1 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 16]
                v2 = call sha256_compression(v0, v1) -> [u32; 8]
                return v2
            }"#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 8]
            v2 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 16]
            v11 = make_array [u32 2091193876, u32 1113340840, u32 3461668143, u32 3254913767, u32 3068490961, u32 2551409935, u32 2927503052, u32 3205228454] : [u32; 8]
            return v11
        }
        ");
    }
}
