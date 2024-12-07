// TODO(https://github.com/noir-lang/noir/issues/4932): rename this file to something more generic
use ark_ec::AffineRepr;
use num_bigint::BigUint;

use crate::FieldElement;
use acir::AcirField;
use acir::BlackBoxFunc;

use crate::BlackBoxResolutionError;

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

    let mut output_point = grumpkin::SWAffine::zero();

    for i in (0..points.len()).step_by(3) {
        let point =
            create_point(points[i], points[i + 1], points[i + 2] == FieldElement::from(1_u128))
                .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::MultiScalarMul, e))?;

        let scalar_low: u128 = scalars_lo[i / 3].try_into_u128().ok_or_else(|| {
            BlackBoxResolutionError::Failed(
                BlackBoxFunc::MultiScalarMul,
                format!("Limb {} is not less than 2^128", scalars_lo[i].to_hex()),
            )
        })?;

        let scalar_high: u128 = scalars_hi[i / 3].try_into_u128().ok_or_else(|| {
            BlackBoxResolutionError::Failed(
                BlackBoxFunc::MultiScalarMul,
                format!("Limb {} is not less than 2^128", scalars_hi[i].to_hex()),
            )
        })?;

        let mut bytes = scalar_high.to_be_bytes().to_vec();
        bytes.extend_from_slice(&scalar_low.to_be_bytes());

        let grumpkin_integer = BigUint::from_bytes_be(&bytes);

        // Check if this is smaller than the grumpkin modulus
        // if grumpkin_integer >= grumpkin::FrConfig::MODULUS.into() {
        //     return Err(BlackBoxResolutionError::Failed(
        //         BlackBoxFunc::MultiScalarMul,
        //         format!("{} is not a valid grumpkin scalar", grumpkin_integer.to_str_radix(16)),
        //     ));
        // }

        let iteration_output_point =
            grumpkin::SWAffine::from(point.mul_bigint(grumpkin_integer.to_u64_digits()));

        output_point = grumpkin::SWAffine::from(output_point + iteration_output_point);
    }

    if let Some((out_x, out_y)) = output_point.xy() {
        Ok((
            FieldElement::from_repr(*out_x),
            FieldElement::from_repr(*out_y),
            FieldElement::from(output_point.is_zero() as u128),
        ))
    } else {
        Ok((FieldElement::from(0_u128), FieldElement::from(0_u128), FieldElement::from(1_u128)))
    }
}

pub fn embedded_curve_add(
    input1: [FieldElement; 3],
    input2: [FieldElement; 3],
) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
    let point1 = create_point(input1[0], input1[1], input1[2] == FieldElement::one())
        .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::EmbeddedCurveAdd, e))?;
    let point2 = create_point(input2[0], input2[1], input2[2] == FieldElement::one())
        .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::EmbeddedCurveAdd, e))?;
    let res = grumpkin::SWAffine::from(point1 + point2);
    if let Some((res_x, res_y)) = res.xy() {
        Ok((
            FieldElement::from_repr(*res_x),
            FieldElement::from_repr(*res_y),
            FieldElement::from(res.is_zero() as u128),
        ))
    } else if res.is_zero() {
        Ok((FieldElement::from(0_u128), FieldElement::from(0_u128), FieldElement::from(1_u128)))
    } else {
        Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EmbeddedCurveAdd,
            "Point is not on curve".to_string(),
        ))
    }
}

fn create_point(
    x: FieldElement,
    y: FieldElement,
    is_infinite: bool,
) -> Result<grumpkin::SWAffine, String> {
    if is_infinite {
        return Ok(grumpkin::SWAffine::zero());
    }
    let point = grumpkin::SWAffine::new_unchecked(x.into_repr(), y.into_repr());
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

    fn get_generator() -> [FieldElement; 3] {
        let generator = grumpkin::SWAffine::generator();
        let generator_x = FieldElement::from_repr(*generator.x().unwrap());
        let generator_y = FieldElement::from_repr(*generator.y().unwrap());
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

    // #[test]
    // fn rejects_grumpkin_modulus() {
    //     let x = grumpkin::FrConfig::MODULUS.to_bytes_be();

    //     let low = FieldElement::from_be_bytes_reduce(&x[16..32]);
    //     let high = FieldElement::from_be_bytes_reduce(&x[0..16]);

    //     let res = multi_scalar_mul(&get_generator(), &[low], &[high]);

    //     assert_eq!(
    //         res,
    //         Err(BlackBoxResolutionError::Failed(
    //             BlackBoxFunc::MultiScalarMul,
    //             "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47 is not a valid grumpkin scalar".into(),
    //         ))
    //     );
    // }

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
}
