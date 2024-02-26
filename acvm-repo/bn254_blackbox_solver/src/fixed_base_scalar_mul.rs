use ark_ec::AffineRepr;
use ark_ff::MontConfig;
use num_bigint::BigUint;

use acir::{BlackBoxFunc, FieldElement};

use crate::BlackBoxResolutionError;

pub fn fixed_base_scalar_mul(
    low: &FieldElement,
    high: &FieldElement,
) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
    let low: u128 = low.try_into_u128().ok_or_else(|| {
        BlackBoxResolutionError::Failed(
            BlackBoxFunc::FixedBaseScalarMul,
            format!("Limb {} is not less than 2^128", low.to_hex()),
        )
    })?;

    let high: u128 = high.try_into_u128().ok_or_else(|| {
        BlackBoxResolutionError::Failed(
            BlackBoxFunc::FixedBaseScalarMul,
            format!("Limb {} is not less than 2^128", high.to_hex()),
        )
    })?;

    let mut bytes = high.to_be_bytes().to_vec();
    bytes.extend_from_slice(&low.to_be_bytes());

    // Check if this is smaller than the grumpkin modulus
    let grumpkin_integer = BigUint::from_bytes_be(&bytes);

    if grumpkin_integer >= grumpkin::FrConfig::MODULUS.into() {
        return Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::FixedBaseScalarMul,
            format!("{} is not a valid grumpkin scalar", grumpkin_integer.to_str_radix(16)),
        ));
    }

    let result = grumpkin::SWAffine::from(
        grumpkin::SWAffine::generator().mul_bigint(grumpkin_integer.to_u64_digits()),
    );
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
    let mut point1 = grumpkin::SWAffine::new(input1_x.into_repr(), input1_y.into_repr());
    let point2 = grumpkin::SWAffine::new(input2_x.into_repr(), input2_y.into_repr());
    let res = point1 + point2;
    point1 = res.into();
    if let Some((res_x, res_y)) = point1.xy() {
        Ok((FieldElement::from_repr(*res_x), FieldElement::from_repr(*res_y)))
    } else {
        Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EmbeddedCurveAdd,
            "Point is not on curve".to_string(),
        ))
    }
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

        let expected_error =  Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::FixedBaseScalarMul,
            "Limb 0000000000000000000000000000000100000000000000000000000000000000 is not less than 2^128".into()
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
                "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47 is not a valid grumpkin scalar".into()
            ))
        );
    }
}
