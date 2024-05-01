// TODO(https://github.com/noir-lang/noir/issues/4932): rename this file to something more generic
use ark_ec::AffineRepr;
use ark_ff::MontConfig;
use num_bigint::BigUint;

use acir::{BlackBoxFunc, FieldElement};

use crate::BlackBoxResolutionError;

/// Performs fixed-base scalar multiplication using the curve's generator point.
pub fn fixed_base_scalar_mul(
    low: &FieldElement,
    high: &FieldElement,
) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
    let generator = grumpkin::SWAffine::generator();
    let generator_x = FieldElement::from_repr(*generator.x().unwrap());
    let generator_y = FieldElement::from_repr(*generator.y().unwrap());

    variable_base_scalar_mul(&generator_x, &generator_y, low, high).map_err(|err| match err {
        BlackBoxResolutionError::Failed(_, message) => {
            BlackBoxResolutionError::Failed(BlackBoxFunc::FixedBaseScalarMul, message)
        }
    })
}

pub fn variable_base_scalar_mul(
    point_x: &FieldElement,
    point_y: &FieldElement,
    scalar_low: &FieldElement,
    scalar_high: &FieldElement,
) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
    let point1 = create_point(*point_x, *point_y)
        .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::VariableBaseScalarMul, e))?;

    let scalar_low: u128 = scalar_low.try_into_u128().ok_or_else(|| {
        BlackBoxResolutionError::Failed(
            BlackBoxFunc::VariableBaseScalarMul,
            format!("Limb {} is not less than 2^128", scalar_low.to_hex()),
        )
    })?;

    let scalar_high: u128 = scalar_high.try_into_u128().ok_or_else(|| {
        BlackBoxResolutionError::Failed(
            BlackBoxFunc::VariableBaseScalarMul,
            format!("Limb {} is not less than 2^128", scalar_high.to_hex()),
        )
    })?;

    let mut bytes = scalar_high.to_be_bytes().to_vec();
    bytes.extend_from_slice(&scalar_low.to_be_bytes());

    // Check if this is smaller than the grumpkin modulus
    let grumpkin_integer = BigUint::from_bytes_be(&bytes);

    if grumpkin_integer >= grumpkin::FrConfig::MODULUS.into() {
        return Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::VariableBaseScalarMul,
            format!("{} is not a valid grumpkin scalar", grumpkin_integer.to_str_radix(16)),
        ));
    }

    let result = grumpkin::SWAffine::from(point1.mul_bigint(grumpkin_integer.to_u64_digits()));
    if let Some((res_x, res_y)) = result.xy() {
        Ok((FieldElement::from_repr(*res_x), FieldElement::from_repr(*res_y)))
    } else {
        Ok((FieldElement::zero(), FieldElement::zero()))
    }
}

pub fn embedded_curve_add(
    input1_x: FieldElement,
    input1_y: FieldElement,
    input2_x: FieldElement,
    input2_y: FieldElement,
) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
    let point1 = create_point(input1_x, input1_y)
        .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::EmbeddedCurveAdd, e))?;
    let point2 = create_point(input2_x, input2_y)
        .map_err(|e| BlackBoxResolutionError::Failed(BlackBoxFunc::EmbeddedCurveAdd, e))?;
    let res = grumpkin::SWAffine::from(point1 + point2);
    if let Some((res_x, res_y)) = res.xy() {
        Ok((FieldElement::from_repr(*res_x), FieldElement::from_repr(*res_y)))
    } else {
        Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EmbeddedCurveAdd,
            "Point is not on curve".to_string(),
        ))
    }
}

fn create_point(x: FieldElement, y: FieldElement) -> Result<grumpkin::SWAffine, String> {
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
mod grumpkin_fixed_base_scalar_mul {
    use ark_ff::BigInteger;

    use super::*;

    #[test]
    fn smoke_test() -> Result<(), BlackBoxResolutionError> {
        let input = FieldElement::one();

        let res = fixed_base_scalar_mul(&input, &FieldElement::zero())?;
        let x = "0000000000000000000000000000000000000000000000000000000000000001";
        let y = "0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c";

        assert_eq!(x, res.0.to_hex());
        assert_eq!(y, res.1.to_hex());
        Ok(())
    }

    #[test]
    fn low_high_smoke_test() -> Result<(), BlackBoxResolutionError> {
        let low = FieldElement::one();
        let high = FieldElement::from(2u128);

        let res = fixed_base_scalar_mul(&low, &high)?;
        let x = "0702ab9c7038eeecc179b4f209991bcb68c7cb05bf4c532d804ccac36199c9a9";
        let y = "23f10e9e43a3ae8d75d24154e796aae12ae7af546716e8f81a2564f1b5814130";

        assert_eq!(x, res.0.to_hex());
        assert_eq!(y, res.1.to_hex());
        Ok(())
    }

    #[test]
    fn rejects_invalid_limbs() {
        let max_limb = FieldElement::from(u128::MAX);
        let invalid_limb = max_limb + FieldElement::one();

        let expected_error = Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::FixedBaseScalarMul,
            "Limb 0000000000000000000000000000000100000000000000000000000000000000 is not less than 2^128".into(),
        ));

        let res = fixed_base_scalar_mul(&invalid_limb, &FieldElement::zero());
        assert_eq!(res, expected_error);

        let res = fixed_base_scalar_mul(&FieldElement::zero(), &invalid_limb);
        assert_eq!(res, expected_error);
    }

    #[test]
    fn rejects_grumpkin_modulus() {
        let x = grumpkin::FrConfig::MODULUS.to_bytes_be();

        let high = FieldElement::from_be_bytes_reduce(&x[0..16]);
        let low = FieldElement::from_be_bytes_reduce(&x[16..32]);

        let res = fixed_base_scalar_mul(&low, &high);

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::FixedBaseScalarMul,
                "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47 is not a valid grumpkin scalar".into(),
            ))
        );
    }

    #[test]
    fn variable_base_matches_fixed_base_for_generator_on_input(
    ) -> Result<(), BlackBoxResolutionError> {
        let low = FieldElement::one();
        let high = FieldElement::from(2u128);

        let generator = grumpkin::SWAffine::generator();
        let generator_x = FieldElement::from_repr(*generator.x().unwrap());
        let generator_y = FieldElement::from_repr(*generator.y().unwrap());

        let fixed_res = fixed_base_scalar_mul(&low, &high)?;
        let variable_res = variable_base_scalar_mul(&generator_x, &generator_y, &low, &high)?;

        assert_eq!(fixed_res, variable_res);
        Ok(())
    }

    #[test]
    fn variable_base_scalar_mul_rejects_invalid_point() {
        let invalid_point_x = FieldElement::one();
        let invalid_point_y = FieldElement::one();
        let valid_scalar_low = FieldElement::zero();
        let valid_scalar_high = FieldElement::zero();

        let res = variable_base_scalar_mul(
            &invalid_point_x,
            &invalid_point_y,
            &valid_scalar_low,
            &valid_scalar_high,
        );

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::VariableBaseScalarMul,
                "Point (0000000000000000000000000000000000000000000000000000000000000001, 0000000000000000000000000000000000000000000000000000000000000001) is not on curve".into(),
            ))
        );
    }

    #[test]
    fn rejects_addition_of_points_not_in_curve() {
        let x = FieldElement::from(1u128);
        let y = FieldElement::from(2u128);

        let res = embedded_curve_add(x, y, x, y);

        assert_eq!(
            res,
            Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::EmbeddedCurveAdd,
                "Point (0000000000000000000000000000000000000000000000000000000000000001, 0000000000000000000000000000000000000000000000000000000000000002) is not on curve".into(),
            ))
        );
    }
}
