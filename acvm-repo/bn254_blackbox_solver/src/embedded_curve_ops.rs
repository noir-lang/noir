use ark_ec::AffineRepr;
use ark_ec::VariableBaseMSM;
use ark_ff::{BigInt, MontConfig};

use crate::FieldElement;
use acir::AcirField;
use acir::BlackBoxFunc;

use crate::BlackBoxResolutionError;

/// Converts a field element to u128, returning an error if it doesn't fit.
fn field_to_u128_limb(
    limb: &FieldElement,
    func: BlackBoxFunc,
) -> Result<u128, BlackBoxResolutionError> {
    limb.try_into_u128().ok_or_else(|| {
        BlackBoxResolutionError::Failed(
            func,
            format!("Limb {} is not less than 2^128", limb.to_hex()),
        )
    })
}

/// Performs multi scalar multiplication of points with scalars.
pub fn multi_scalar_mul(
    points: &[FieldElement],
    scalars_lo: &[FieldElement],
    scalars_hi: &[FieldElement],
) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
    if points.len() != 3 * scalars_lo.len() || scalars_lo.len() != scalars_hi.len() {
        return Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::MultiScalarMul,
            "Points and scalars must have the same length".to_string(),
        ));
    }

    // Collect all bases (affine points) and scalars for batch MSM
    let mut bases = Vec::new();
    let mut big_ints = Vec::new();

    for i in (0..points.len()).step_by(3) {
        if points[i + 2] > FieldElement::one() {
            return Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::MultiScalarMul,
                "EmbeddedCurvePoint is malformed (non-boolean `is_infinite` flag)".to_string(),
            ));
        }
        let point = create_point(points[i], points[i + 1], points[i + 2])
            .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::MultiScalarMul, e))?;

        let scalar_low: u128 =
            field_to_u128_limb(&scalars_lo[i / 3], BlackBoxFunc::MultiScalarMul)?;

        let scalar_high: u128 =
            field_to_u128_limb(&scalars_hi[i / 3], BlackBoxFunc::MultiScalarMul)?;

        // Convert to BigInt<4>, using u64 limbs.
        let limbs_array = [
            scalar_low as u64,
            (scalar_low >> 64) as u64,
            scalar_high as u64,
            (scalar_high >> 64) as u64,
        ];
        let scalar_bigint = BigInt::new(limbs_array);

        // Check if this is smaller than the grumpkin modulus
        if scalar_bigint >= ark_grumpkin::FrConfig::MODULUS {
            // Format as hex string (big-endian, most significant limb first)
            let hex_str = format!(
                "{:016x}{:016x}{:016x}{:016x}",
                limbs_array[3], limbs_array[2], limbs_array[1], limbs_array[0]
            );
            return Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::MultiScalarMul,
                format!("{hex_str} is not a valid grumpkin scalar"),
            ));
        }

        bases.push(point);
        big_ints.push(scalar_bigint);
    }

    // Perform batch multi-scalar multiplication
    let output_point = ark_grumpkin::Projective::msm_bigint(&bases, &big_ints);
    let output_point = ark_grumpkin::Affine::from(output_point);

    if let Some((out_x, out_y)) = output_point.xy() {
        Ok((
            FieldElement::from_repr(out_x),
            FieldElement::from_repr(out_y),
            FieldElement::from(u128::from(output_point.is_zero())),
        ))
    } else {
        Ok((FieldElement::from(0_u128), FieldElement::from(0_u128), FieldElement::from(1_u128)))
    }
}

pub fn embedded_curve_add(
    input1: [FieldElement; 3],
    input2: [FieldElement; 3],
) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
    if input1[2] > FieldElement::one() || input2[2] > FieldElement::one() {
        return Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EmbeddedCurveAdd,
            "EmbeddedCurvePoint is malformed (non-boolean `is_infinite` flag)".to_string(),
        ));
    }

    let point1 = create_point(input1[0], input1[1], input1[2])
        .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::EmbeddedCurveAdd, e))?;
    let point2 = create_point(input2[0], input2[1], input2[2])
        .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::EmbeddedCurveAdd, e))?;

    for point in [point1, point2] {
        if point == ark_grumpkin::Affine::zero() {
            return Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::EmbeddedCurveAdd,
                format!("Infinite input: embedded_curve_add({point1}, {point2})"),
            ));
        }
    }

    let res = ark_grumpkin::Affine::from(point1 + point2);
    if let Some((res_x, res_y)) = res.xy() {
        Ok((FieldElement::from_repr(res_x), FieldElement::from_repr(res_y), FieldElement::zero()))
    } else {
        assert!(res.is_zero());
        Ok((FieldElement::from(0_u128), FieldElement::from(0_u128), FieldElement::from(1_u128)))
    }
}

fn create_point(
    x: FieldElement,
    y: FieldElement,
    is_infinite: FieldElement,
) -> Result<ark_grumpkin::Affine, String> {
    if is_infinite.is_one() {
        return Ok(ark_grumpkin::Affine::zero());
    } else if !is_infinite.is_zero() {
        return Err("`is_infinite` flag is non-boolean".to_string());
    }

    let point = ark_grumpkin::Affine::new_unchecked(x.into_repr(), y.into_repr());
    if !point.is_on_curve() {
        return Err(format!("Point ({}, {}) is not on curve", x.to_hex(), y.to_hex()));
    };
    if !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err(format!("Point ({}, {}) is not in correct subgroup", x.to_hex(), y.to_hex()));
    };
    Ok(point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::BigInteger;
    use num_bigint::BigUint;

    fn get_generator() -> [FieldElement; 3] {
        let generator = ark_grumpkin::Affine::generator();
        let generator_x = FieldElement::from_repr(generator.x().unwrap());
        let generator_y = FieldElement::from_repr(generator.y().unwrap());
        [generator_x, generator_y, FieldElement::zero()]
    }

    #[test]
    fn smoke_test() -> Result<(), BlackBoxResolutionError> {
        // We check that multiplying 1 by generator results in the generator
        let generator = get_generator();

        let res = multi_scalar_mul(&generator, &[FieldElement::one()], &[FieldElement::zero()])?;

        assert_eq!(generator[0], res.0);
        assert_eq!(generator[1], res.1);
        Ok(())
    }

    #[test]
    fn low_high_smoke_test() -> Result<(), BlackBoxResolutionError> {
        let points = get_generator();
        let scalars_lo = [FieldElement::one()];
        let scalars_hi = [FieldElement::from(2u128)];

        let res = multi_scalar_mul(&points, &scalars_lo, &scalars_hi)?;
        let x = "0702ab9c7038eeecc179b4f209991bcb68c7cb05bf4c532d804ccac36199c9a9";
        let y = "23f10e9e43a3ae8d75d24154e796aae12ae7af546716e8f81a2564f1b5814130";

        assert_eq!(x, res.0.to_hex());
        assert_eq!(y, res.1.to_hex());
        Ok(())
    }

    #[test]
    fn rejects_invalid_scalar_limbs() {
        let points = get_generator();

        let max_limb = FieldElement::from(u128::MAX);
        let invalid_limb = max_limb + FieldElement::one();

        let expected_error = Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::MultiScalarMul,
            "Limb 0000000000000000000000000000000100000000000000000000000000000000 is not less than 2^128".into(),
        ));

        let res = multi_scalar_mul(&points, &[FieldElement::one()], &[invalid_limb]);
        assert_eq!(res, expected_error);

        let res = multi_scalar_mul(&points, &[invalid_limb], &[FieldElement::one()]);
        assert_eq!(res, expected_error);
    }

    #[test]
    fn rejects_grumpkin_modulus_when_pedantic() {
        let x = ark_grumpkin::FrConfig::MODULUS.to_bytes_be();
        let low = FieldElement::from_be_bytes_reduce(&x[16..32]);
        let high = FieldElement::from_be_bytes_reduce(&x[0..16]);

        let res = multi_scalar_mul(&get_generator(), &[low], &[high]);

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::MultiScalarMul,
                "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47 is not a valid grumpkin scalar".into(),
            ))
        );
    }

    #[test]
    fn rejects_invalid_point() {
        let invalid_point_x = FieldElement::one();
        let invalid_point_y = FieldElement::one();
        let valid_scalar_low = FieldElement::zero();
        let valid_scalar_high = FieldElement::zero();

        let res = multi_scalar_mul(
            &[invalid_point_x, invalid_point_y, FieldElement::zero()],
            &[valid_scalar_low],
            &[valid_scalar_high],
        );

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::MultiScalarMul,
                "Point (0000000000000000000000000000000000000000000000000000000000000001, 0000000000000000000000000000000000000000000000000000000000000001) is not on curve".into(),
            ))
        );
    }

    #[test]
    fn throws_on_args_length_mismatch() {
        let points = get_generator();
        let scalars_lo = [FieldElement::from(2u128)];
        let scalars_hi = [];

        let res = multi_scalar_mul(&points, &scalars_lo, &scalars_hi);

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::MultiScalarMul,
                "Points and scalars must have the same length".into(),
            ))
        );
    }

    #[test]
    fn rejects_addition_of_points_not_in_curve() {
        let x = FieldElement::from(1u128);
        let y = FieldElement::from(2u128);

        let res = embedded_curve_add(
            [x, y, FieldElement::from(0u128)],
            [x, y, FieldElement::from(0u128)],
        );

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::EmbeddedCurveAdd,
                "Point (0000000000000000000000000000000000000000000000000000000000000001, 0000000000000000000000000000000000000000000000000000000000000002) is not on curve".into(),
            ))
        );
    }

    #[test]
    fn rejects_addition_of_infinite_points_when_pedantic() {
        let x = FieldElement::from(1u128);
        let y = FieldElement::from(1u128);

        let res = embedded_curve_add(
            [x, y, FieldElement::from(1u128)],
            [x, y, FieldElement::from(1u128)],
        );

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::EmbeddedCurveAdd,
                "Infinite input: embedded_curve_add(infinity, infinity)".into(),
            ))
        );
    }

    #[test]
    fn output_of_msm_matches_add() -> Result<(), BlackBoxResolutionError> {
        let points = get_generator();
        let scalars_lo = [FieldElement::from(2u128)];
        let scalars_hi = [FieldElement::zero()];

        let msm_res = multi_scalar_mul(&points, &scalars_lo, &scalars_hi)?;
        let add_res = embedded_curve_add(
            [points[0], points[1], FieldElement::from(0u128)],
            [points[0], points[1], FieldElement::from(0u128)],
        )?;

        assert_eq!(msm_res.0, add_res.0);
        assert_eq!(msm_res.1, add_res.1);
        Ok(())
    }

    #[test]
    fn rejects_non_boolean_is_infinite_flag() {
        let a = get_generator();

        let mut b = get_generator();
        // Manipulate `is_infinite` to be non-boolean.
        b[2] = FieldElement::from(2u32);

        let res = embedded_curve_add(a, b);

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::EmbeddedCurveAdd,
                "EmbeddedCurvePoint is malformed (non-boolean `is_infinite` flag)".into(),
            ))
        );
    }

    fn msm_against_add_and_mul(
        points: &[FieldElement],
        scalars_lo: &[FieldElement],
        scalars_hi: &[FieldElement],
    ) {
        // Manual MSM via add and mul
        let mut output_point = ark_grumpkin::Affine::zero();
        for i in (0..points.len()).step_by(3) {
            let point = create_point(points[i], points[i + 1], points[i + 2]).unwrap();

            let scalar_low: u128 =
                field_to_u128_limb(&scalars_lo[i / 3], BlackBoxFunc::MultiScalarMul).unwrap();

            let scalar_high: u128 =
                field_to_u128_limb(&scalars_hi[i / 3], BlackBoxFunc::MultiScalarMul).unwrap();

            let mut bytes = scalar_high.to_be_bytes().to_vec();
            bytes.extend_from_slice(&scalar_low.to_be_bytes());

            let grumpkin_integer = BigUint::from_bytes_be(&bytes);

            // Check if this is smaller than the grumpkin modulus
            assert!(
                grumpkin_integer < ark_grumpkin::FrConfig::MODULUS.into(),
                "invalid grumpkin scalar",
            );

            let iteration_output_point =
                ark_grumpkin::Affine::from(point.mul_bigint(grumpkin_integer.to_u64_digits()));

            output_point = ark_grumpkin::Affine::from(output_point + iteration_output_point);
        }

        // Batch MSM
        let output_point2 = multi_scalar_mul(points, scalars_lo, scalars_hi).unwrap();

        // Checks both implementations have the same result
        if let Some((out_x, out_y)) = output_point.xy() {
            assert_eq!(FieldElement::from_repr(out_x), output_point2.0);
            assert_eq!(FieldElement::from_repr(out_y), output_point2.1);
        } else {
            // Point at infinity
            assert_eq!(output_point2.2, FieldElement::from(1u128));
        }
    }

    #[test]
    // Checks that multi_scalar_mul() produce the same result as adding and multiplying manually the points.
    fn batch_msm() {
        let generator = get_generator();

        // Helper to generate nth multiple of generator
        let gen_multiple = |n: u64| -> (FieldElement, FieldElement, FieldElement) {
            let mut point = ark_grumpkin::Affine::zero();
            for _ in 0..n {
                point = ark_grumpkin::Affine::from(point + ark_grumpkin::Affine::generator());
            }
            if let Some((x, y)) = point.xy() {
                (FieldElement::from_repr(x), FieldElement::from_repr(y), FieldElement::zero())
            } else {
                (FieldElement::zero(), FieldElement::zero(), FieldElement::one())
            }
        };

        // Test case 1: Single point with small scalar
        {
            let points = vec![generator[0], generator[1], generator[2]];
            let scalars_lo = vec![FieldElement::from(7u128)];
            let scalars_hi = vec![FieldElement::from(0u128)];
            msm_against_add_and_mul(&points, &scalars_lo, &scalars_hi);
        }

        // Test case 2: Two points with varied scalars
        {
            let point2 = gen_multiple(2);
            let points =
                vec![generator[0], generator[1], generator[2], point2.0, point2.1, point2.2];
            let scalars_lo = vec![FieldElement::from(3u128), FieldElement::from(11u128)];
            let scalars_hi = vec![FieldElement::from(0u128), FieldElement::from(0u128)];
            msm_against_add_and_mul(&points, &scalars_lo, &scalars_hi);
        }

        // Test case 3: Three points with high/low scalar combinations
        {
            let point2 = gen_multiple(2);
            let point3 = gen_multiple(3);
            let points = vec![
                generator[0],
                generator[1],
                generator[2],
                point2.0,
                point2.1,
                point2.2,
                point3.0,
                point3.1,
                point3.2,
            ];
            let scalars_lo = vec![
                FieldElement::from(5u128),
                FieldElement::from(17u128),
                FieldElement::from(42u128),
            ];
            let scalars_hi = vec![
                FieldElement::from(0u128),
                FieldElement::from(1u128),
                FieldElement::from(2u128),
            ];
            msm_against_add_and_mul(&points, &scalars_lo, &scalars_hi);
        }

        // Test case 4: Five points with larger scalars
        {
            let point2 = gen_multiple(2);
            let point3 = gen_multiple(3);
            let point4 = gen_multiple(4);
            let point5 = gen_multiple(5);
            let points = vec![
                generator[0],
                generator[1],
                generator[2],
                point2.0,
                point2.1,
                point2.2,
                point3.0,
                point3.1,
                point3.2,
                point4.0,
                point4.1,
                point4.2,
                point5.0,
                point5.1,
                point5.2,
            ];
            let scalars_lo = vec![
                FieldElement::from(100u128),
                FieldElement::from(200u128),
                FieldElement::from(300u128),
                FieldElement::from(u128::MAX),
                FieldElement::from(12345678901234567890u128),
            ];
            let scalars_hi = vec![
                FieldElement::from(0u128),
                FieldElement::from(5u128),
                FieldElement::from(10u128),
                FieldElement::from(0u128),
                FieldElement::from(100u128),
            ];
            msm_against_add_and_mul(&points, &scalars_lo, &scalars_hi);
        }

        // Test case 5: Single point with zero scalar
        {
            let points = vec![generator[0], generator[1], generator[2]];
            let scalars_lo = vec![FieldElement::from(0u128)];
            let scalars_hi = vec![FieldElement::from(0u128)];
            msm_against_add_and_mul(&points, &scalars_lo, &scalars_hi);
        }

        // Test case 6: Multiple points with mixed zero and non-zero scalars
        {
            let point2 = gen_multiple(2);
            let point3 = gen_multiple(3);
            let points = vec![
                generator[0],
                generator[1],
                generator[2],
                point2.0,
                point2.1,
                point2.2,
                point3.0,
                point3.1,
                point3.2,
            ];
            let scalars_lo = vec![
                FieldElement::from(0u128),
                FieldElement::from(42u128),
                FieldElement::from(0u128),
            ];
            let scalars_hi = vec![
                FieldElement::from(0u128),
                FieldElement::from(0u128),
                FieldElement::from(0u128),
            ];
            msm_against_add_and_mul(&points, &scalars_lo, &scalars_hi);
        }
    }
}
