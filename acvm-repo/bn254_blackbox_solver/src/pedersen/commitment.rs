// Taken from: https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/crypto/pedersen/pederson.rs

use ark_ec::{short_weierstrass::Affine, AffineRepr, CurveGroup};
use ark_ff::{MontConfig, PrimeField};
use grumpkin::{Fq, FqConfig, Fr, FrConfig, GrumpkinParameters};

use crate::generator::generators::{derive_generators, DEFAULT_DOMAIN_SEPARATOR};

/// Given a vector of fields, generate a pedersen commitment using the indexed generators.
pub(crate) fn commit_native_with_index(
    inputs: &[Fq],
    starting_index: u32,
) -> Affine<GrumpkinParameters> {
    let generators =
        derive_generators(DEFAULT_DOMAIN_SEPARATOR, inputs.len() as u32, starting_index);

    // As |F_r| > |F_q|, we can safely convert any `F_q` into an `F_r` uniquely.
    assert!(FrConfig::MODULUS > FqConfig::MODULUS);

    inputs.iter().enumerate().fold(Affine::zero(), |mut acc, (i, input)| {
        acc = (acc + (generators[i] * Fr::from_bigint(input.into_bigint()).unwrap()).into_affine())
            .into_affine();
        acc
    })
}

#[cfg(test)]
mod test {

    use acir::{AcirField, FieldElement};
    use ark_ec::short_weierstrass::Affine;
    use ark_std::{One, Zero};
    use grumpkin::Fq;

    use crate::pedersen::commitment::commit_native_with_index;

    #[test]
    fn commitment() {
        // https://github.com/AztecProtocol/aztec-packages/blob/72931bdb8202c34042cdfb8cee2ef44b75939879/barretenberg/cpp/src/barretenberg/crypto/pedersen_commitment/pedersen.test.cpp#L10-L18
        let res = commit_native_with_index(&[Fq::one(), Fq::one()], 0);
        let expected = Affine::new(
            FieldElement::from_hex(
                "0x2f7a8f9a6c96926682205fb73ee43215bf13523c19d7afe36f12760266cdfe15",
            )
            .unwrap()
            .into_repr(),
            FieldElement::from_hex(
                "0x01916b316adbbf0e10e39b18c1d24b33ec84b46daddf72f43878bcc92b6057e6",
            )
            .unwrap()
            .into_repr(),
        );

        assert_eq!(res, expected);
    }

    #[test]
    fn commitment_with_zero() {
        // https://github.com/AztecProtocol/aztec-packages/blob/72931bdb8202c34042cdfb8cee2ef44b75939879/barretenberg/cpp/src/barretenberg/crypto/pedersen_commitment/pedersen.test.cpp#L20-L29
        let res = commit_native_with_index(&[Fq::zero(), Fq::one()], 0);
        let expected = Affine::new(
            FieldElement::from_hex(
                "0x054aa86a73cb8a34525e5bbed6e43ba1198e860f5f3950268f71df4591bde402",
            )
            .unwrap()
            .into_repr(),
            FieldElement::from_hex(
                "0x209dcfbf2cfb57f9f6046f44d71ac6faf87254afc7407c04eb621a6287cac126",
            )
            .unwrap()
            .into_repr(),
        );

        assert_eq!(res, expected);
    }
}
