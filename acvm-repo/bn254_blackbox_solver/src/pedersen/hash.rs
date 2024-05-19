// Taken from: https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/crypto/pedersen/pederson_hash.rs

use std::sync::OnceLock;

use ark_ec::{short_weierstrass::Affine, CurveConfig, CurveGroup};
use grumpkin::GrumpkinParameters;

use crate::generator::generators::derive_generators;

use super::commitment::commit_native_with_index;

/**
 * @brief Given a vector of fields, generate a pedersen commitment using the indexed generators.
 *
 * @details This method uses `Curve::BaseField` members as inputs. This aligns with what we expect when creating
 * grumpkin commitments to field elements inside a BN254 SNARK circuit.
 * @param inputs
 * @param context
 * @return Curve::AffineElement
 */
//TODO: confirm we can do this with scalar field
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

    use ark_ff::MontFp;
    use ark_std::One;
    use grumpkin::Fq;

    //reference: https://github.com/AztecProtocol/barretenberg/blob/master/cpp/src/barretenberg/crypto/pedersen_hash/pedersen.test.cpp
    #[test]
    fn hash_one() {
        let res = hash_with_index(&[Fq::one(), Fq::one()], 0);

        // 07ebfbf4df29888c6cd6dca13d4bb9d1a923013ddbbcbdc3378ab8845463297b
        assert_eq!(
            res,
            MontFp!("3583137940367543141169889198758850326673923325182598243450662697654714313083")
        );
    }

    #[test]
    fn test_hash_with_index() {
        let res = hash_with_index(&[Fq::one(), Fq::one()], 5);
        // 1c446df60816b897cda124524e6b03f36df0cec333fad87617aab70d7861daa6
        assert_eq!(
            res,
            MontFp!(
                "12785664284086914537273210116175139764153812914951498056047869066787449592486"
            )
        );
    }
}
