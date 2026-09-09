use acir::{
    AcirField, BlackBoxFunc,
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;

use crate::pwg::{OpcodeResolutionError, input_to_value, insert_value};

pub(super) fn multi_scalar_mul<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    points: &[FunctionInput<F>],
    scalars: &[FunctionInput<F>],
    predicate: FunctionInput<F>,
    outputs: (Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let (res_x, res_y) =
        execute_multi_scalar_mul(backend, initial_witness, points, scalars, predicate)?;

    // Insert the resulting point into the witness map
    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    Ok(())
}

pub(crate) fn execute_multi_scalar_mul<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &WitnessMap<F>,
    points: &[FunctionInput<F>],
    scalars: &[FunctionInput<F>],
    predicate: FunctionInput<F>,
) -> Result<(F, F), OpcodeResolutionError<F>> {
    assert!(scalars.len().is_multiple_of(2), "Number of scalars must be even");
    assert!(points.len().is_multiple_of(2), "Number of points must be a multiple of 2");
    assert_eq!(
        scalars.len() / 2,
        points.len() / 2,
        "Number of scalars must be the same as the number of points"
    );

    for point in points.chunks(2) {
        if let [x, y] = *point {
            check_all_or_nothing_pair(BlackBoxFunc::MultiScalarMul, "Coordinates", x, y)?;
        }
    }

    for scalar in scalars.chunks(2) {
        if let [lo, hi] = *scalar {
            check_all_or_nothing_pair(BlackBoxFunc::MultiScalarMul, "Scalar limbs", lo, hi)?;
        }
    }

    let points: Result<Vec<_>, _> =
        points.iter().map(|input| input_to_value(initial_witness, *input)).collect();
    let points: Vec<_> = points?.into_iter().collect();

    let scalars: Result<Vec<_>, _> =
        scalars.iter().map(|input| input_to_value(initial_witness, *input)).collect();

    let predicate = input_to_value(initial_witness, predicate)?.is_one();

    let mut scalars_lo = Vec::new();
    let mut scalars_hi = Vec::new();
    for (i, scalar) in scalars?.into_iter().enumerate() {
        if i % 2 == 0 {
            scalars_lo.push(scalar);
        } else {
            scalars_hi.push(scalar);
        }
    }
    // Call the backend's multi-scalar multiplication function
    let (res_x, res_y) = backend.multi_scalar_mul(&points, &scalars_lo, &scalars_hi, predicate)?;
    Ok((res_x, res_y))
}

pub(super) fn embedded_curve_add<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    input1: [FunctionInput<F>; 2],
    input2: [FunctionInput<F>; 2],
    predicate: FunctionInput<F>,
    outputs: (Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let (res_x, res_y) =
        execute_embedded_curve_add(backend, initial_witness, input1, input2, predicate)?;

    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    Ok(())
}

pub(crate) fn execute_embedded_curve_add<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &WitnessMap<F>,
    input1: [FunctionInput<F>; 2],
    input2: [FunctionInput<F>; 2],
    predicate: FunctionInput<F>,
) -> Result<(F, F), OpcodeResolutionError<F>> {
    check_all_or_nothing_pair(BlackBoxFunc::EmbeddedCurveAdd, "Coordinates", input1[0], input1[1])?;
    check_all_or_nothing_pair(BlackBoxFunc::EmbeddedCurveAdd, "Coordinates", input2[0], input2[1])?;

    let input1_x = input_to_value(initial_witness, input1[0])?;
    let input1_y = input_to_value(initial_witness, input1[1])?;
    let input2_x = input_to_value(initial_witness, input2[0])?;
    let input2_y = input_to_value(initial_witness, input2[1])?;
    let predicate = input_to_value(initial_witness, predicate)?.is_one();
    let (res_x, res_y) = backend.ec_add(&input1_x, &input1_y, &input2_x, &input2_y, predicate)?;

    Ok((res_x, res_y))
}

/// Checks that the two halves of an input pair are either both witnesses or both constants,
/// erroring otherwise. `kind` names the pair in the error message, e.g. "Coordinates" for a
/// point's `(x, y)` or "Scalar limbs" for a scalar's `(lo, hi)`.
fn check_all_or_nothing_pair<F: AcirField>(
    func: BlackBoxFunc,
    kind: &str,
    first: FunctionInput<F>,
    second: FunctionInput<F>,
) -> Result<(), OpcodeResolutionError<F>> {
    match (first, second) {
        (FunctionInput::Witness(_), FunctionInput::Witness(_))
        | (FunctionInput::Constant(_), FunctionInput::Constant(_)) => Ok(()),
        _ => Err(OpcodeResolutionError::BlackBoxFunctionFailed(
            func,
            format!(
                "{kind} must be either both witnesses or both constants. Found: {first:?}, {second:?}"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acir::{
        AcirField, BlackBoxFunc, FieldElement,
        circuit::opcodes::FunctionInput,
        native_types::{Witness, WitnessMap},
    };
    use bn254_blackbox_solver::Bn254BlackBoxSolver;

    use super::{execute_embedded_curve_add, execute_multi_scalar_mul};
    use crate::pwg::OpcodeResolutionError;

    /// `y` coordinate of the Grumpkin generator, whose `x` coordinate is 1.
    fn generator_y() -> FieldElement {
        FieldElement::try_from_str("17631683881184975370165255887551781615748388533673675138860")
            .unwrap()
    }

    /// Witness map holding the generator's `y` coordinate in `Witness(2)` and the scalar
    /// `1` split into limbs `lo = 1` (`Witness(3)`) and `hi = 0` (`Witness(4)`).
    /// `Witness(1)` holds the generator's `x` coordinate, which is `1`.
    fn witness_map() -> WitnessMap<FieldElement> {
        WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::one()),
            (Witness(2), generator_y()),
            (Witness(3), FieldElement::one()),
            (Witness(4), FieldElement::zero()),
        ]))
    }

    fn msm(
        points: [FunctionInput<FieldElement>; 2],
        scalars: [FunctionInput<FieldElement>; 2],
    ) -> Result<(FieldElement, FieldElement), OpcodeResolutionError<FieldElement>> {
        execute_multi_scalar_mul(
            &Bn254BlackBoxSolver,
            &witness_map(),
            &points,
            &scalars,
            FunctionInput::Constant(FieldElement::one()),
        )
    }

    fn assert_mixed_pair_rejected<T: std::fmt::Debug>(
        result: Result<T, OpcodeResolutionError<FieldElement>>,
        expected_func: BlackBoxFunc,
    ) {
        match result {
            Err(OpcodeResolutionError::BlackBoxFunctionFailed(func, message)) => {
                assert_eq!(func, expected_func);
                assert!(
                    message.contains("both witnesses or both constants"),
                    "unexpected failure message: {message}"
                );
            }
            other => panic!("expected a mixed constant/witness pair to be rejected, got {other:?}"),
        }
    }

    #[test]
    fn multi_scalar_mul_accepts_uniform_scalar_limbs() {
        let points = [FunctionInput::Witness(Witness(1)), FunctionInput::Witness(Witness(2))];

        let all_witnesses =
            msm(points, [FunctionInput::Witness(Witness(3)), FunctionInput::Witness(Witness(4))])
                .expect("all-witness scalar limbs are valid");
        let all_constants = msm(
            points,
            [
                FunctionInput::Constant(FieldElement::one()),
                FunctionInput::Constant(FieldElement::zero()),
            ],
        )
        .expect("all-constant scalar limbs are valid");

        // `1 * G == G`, whichever way the scalar limbs are declared.
        assert_eq!(all_witnesses, all_constants);
        assert_eq!(all_witnesses, (FieldElement::one(), generator_y()));
    }

    #[test]
    fn multi_scalar_mul_rejects_mixed_scalar_limbs() {
        let points = [FunctionInput::Witness(Witness(1)), FunctionInput::Witness(Witness(2))];

        assert_mixed_pair_rejected(
            msm(
                points,
                [FunctionInput::Constant(FieldElement::one()), FunctionInput::Witness(Witness(4))],
            ),
            BlackBoxFunc::MultiScalarMul,
        );
        assert_mixed_pair_rejected(
            msm(
                points,
                [FunctionInput::Witness(Witness(3)), FunctionInput::Constant(FieldElement::zero())],
            ),
            BlackBoxFunc::MultiScalarMul,
        );
    }

    #[test]
    fn multi_scalar_mul_rejects_mixed_point_coordinates() {
        let scalars = [FunctionInput::Witness(Witness(3)), FunctionInput::Witness(Witness(4))];

        assert_mixed_pair_rejected(
            msm(
                [FunctionInput::Constant(FieldElement::one()), FunctionInput::Witness(Witness(2))],
                scalars,
            ),
            BlackBoxFunc::MultiScalarMul,
        );
    }

    #[test]
    fn embedded_curve_add_rejects_mixed_point_coordinates() {
        let uniform = [FunctionInput::Witness(Witness(1)), FunctionInput::Witness(Witness(2))];
        let mixed =
            [FunctionInput::Constant(FieldElement::one()), FunctionInput::Witness(Witness(2))];

        for (input1, input2) in [(mixed, uniform), (uniform, mixed)] {
            assert_mixed_pair_rejected(
                execute_embedded_curve_add(
                    &Bn254BlackBoxSolver,
                    &witness_map(),
                    input1,
                    input2,
                    FunctionInput::Constant(FieldElement::one()),
                ),
                BlackBoxFunc::EmbeddedCurveAdd,
            );
        }
    }
}
