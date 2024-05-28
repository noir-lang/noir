// Taken from: https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/crypto/pedersen/pederson_hash.rs

use std::sync::OnceLock;

use ark_ec::{short_weierstrass::Affine, CurveConfig, CurveGroup};
use grumpkin::GrumpkinParameters;

use crate::generator::generators::derive_generators;

use super::commitment::commit_native_with_index;

/// Given a vector of fields, generate a pedersen hash using the indexed generators.
pub(crate) fn hash_with_index(
    inputs: &[grumpkin::Fq],
    starting_index: u32,
) -> <GrumpkinParameters as CurveConfig>::BaseField {
    let length_as_scalar: <GrumpkinParameters as CurveConfig>::ScalarField =
        (inputs.len() as u64).into();
    let length_prefix = *length_generator() * length_as_scalar;
    let result = length_prefix + commit_native_with_index(inputs, starting_index);
    result.into_affine().x
}

fn length_generator() -> &'static Affine<GrumpkinParameters> {
    static INSTANCE: OnceLock<Affine<GrumpkinParameters>> = OnceLock::new();
    INSTANCE.get_or_init(|| derive_generators("pedersen_hash_length".as_bytes(), 1, 0)[0])
}

#[cfg(test)]
pub(crate) mod test {

    use super::*;

    use acir::{AcirField, FieldElement};
    use ark_std::One;
    use grumpkin::Fq;

    //reference: https://github.com/AztecProtocol/barretenberg/blob/master/cpp/src/barretenberg/crypto/pedersen_hash/pedersen.test.cpp
    #[test]
    fn hash_one() {
        // https://github.com/AztecProtocol/aztec-packages/blob/72931bdb8202c34042cdfb8cee2ef44b75939879/barretenberg/cpp/src/barretenberg/crypto/pedersen_hash/pedersen.test.cpp#L21-L26
        let res = hash_with_index(&[Fq::one(), Fq::one()], 0);

        assert_eq!(
            res,
            FieldElement::from_hex(
                "0x07ebfbf4df29888c6cd6dca13d4bb9d1a923013ddbbcbdc3378ab8845463297b",
            )
            .unwrap()
            .into_repr(),
        );
    }

    #[test]
    fn test_hash_with_index() {
        // https://github.com/AztecProtocol/aztec-packages/blob/72931bdb8202c34042cdfb8cee2ef44b75939879/barretenberg/cpp/src/barretenberg/crypto/pedersen_hash/pedersen.test.cpp#L28-L33
        let res = hash_with_index(&[Fq::one(), Fq::one()], 5);

        assert_eq!(
            res,
            FieldElement::from_hex(
                "0x1c446df60816b897cda124524e6b03f36df0cec333fad87617aab70d7861daa6",
            )
            .unwrap()
            .into_repr(),
        );
    }
}
