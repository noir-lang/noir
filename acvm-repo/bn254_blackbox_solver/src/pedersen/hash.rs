// Taken from: https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/crypto/pedersen/pederson_hash.rs

use ark_ec::{
    short_weierstrass::{Affine, SWCurveConfig},
    CurveConfig, CurveGroup,
};
use grumpkin::GrumpkinParameters;

use crate::generator::{derive_generators, GeneratorContext};

use super::commitment::commit_native;

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
pub(crate) fn hash(
    inputs: &[grumpkin::Fq],
    context: &mut GeneratorContext<GrumpkinParameters>,
) -> <GrumpkinParameters as CurveConfig>::BaseField {
    let res: Affine<GrumpkinParameters> = (length_generator(0)
        * <GrumpkinParameters as CurveConfig>::ScalarField::from(inputs.len() as u64))
    .into_affine();
    (res + commit_native(inputs, context)).x
}

pub(crate) fn hash_with_index(
    inputs: &[grumpkin::Fq],
    starting_index: u32,
    context: &mut GeneratorContext<GrumpkinParameters>,
) -> <GrumpkinParameters as CurveConfig>::BaseField {
    let res: Affine<GrumpkinParameters> = (length_generator(starting_index)
        * <GrumpkinParameters as CurveConfig>::ScalarField::from(inputs.len() as u64))
    .into_affine();
    (res + commit_native(inputs, context)).x
}

//Note: this can be abstracted to a lazy_static!()
fn length_generator<E: SWCurveConfig>(starting_index: u32) -> Affine<E> {
    derive_generators::<E>("pedersen_hash_length".as_bytes(), 1, starting_index)[0]
}

#[cfg(test)]
pub(crate) mod test {
    use crate::generator::GENERATOR_CONTEXT;

    use super::*;

    use ark_ff::MontFp;
    use ark_std::One;
    use grumpkin::Fq;

    //reference: https://github.com/AztecProtocol/barretenberg/blob/master/cpp/src/barretenberg/crypto/pedersen_hash/pedersen.test.cpp
    #[test]
    fn hash_one() {
        let res = hash(&[Fq::one(), Fq::one()], &mut GENERATOR_CONTEXT.lock().unwrap());
        //TODO: double check that the generators are the same. They could be slightly different due to the way we canonically invert y
        //TODO: compute correct value from generators
        // 07ebfbf4df29888c6cd6dca13d4bb9d1a923013ddbbcbdc3378ab8845463297b
        assert_eq!(
            res,
            MontFp!("3583137940367543141169889198758850326673923325182598243450662697654714313083")
        );
    }

    #[test]
    fn hash_with_index() {
        let res = hash(&[Fq::one(), Fq::one()], &mut GENERATOR_CONTEXT.lock().unwrap());
        //TODO: double check that the generators are the same. They could be slightly different due to the way we canonically invert y
        //TODO: compute correct value from generators
        // 07ebfbf4df29888c6cd6dca13d4bb9d1a923013ddbbcbdc3378ab8845463297b
        assert_eq!(
            res,
            MontFp!("3583137940367543141169889198758850326673923325182598243450662697654714313083")
        );
    }
}
