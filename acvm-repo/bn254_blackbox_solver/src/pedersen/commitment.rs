// Taken from: https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/crypto/pedersen/pederson.rs

use ark_ec::{short_weierstrass::Affine, AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use grumpkin::{Fq, Fr, GrumpkinParameters};

use crate::generator::GeneratorContext;

/**
 * @brief Given a vector of fields, generate a pedersen commitment using the indexed generators.
 *
 * @details This method uses `Curve::BaseField` members as inputs. This aligns with what we expect when creating
 * grumpkin commitments to field elements inside a BN254 SNARK circuit.
 * @param inputs
 * @param context
 * @return Curve::AffineElement
 */
// NOTE: this could be generalized using SWCurveConfig but since we perform the operation over grumpkin its explicit
pub(crate) fn commit_native(
    inputs: &[Fq],
    context: &mut GeneratorContext<GrumpkinParameters>,
) -> Affine<GrumpkinParameters> {
    let generators = context.generators.get(inputs.len(), context.offset, context.domain_separator);

    inputs.iter().enumerate().fold(Affine::zero(), |mut acc, (i, input)| {
        //TODO: this is a sketch conversion do better
        acc = (acc + (generators[i] * Fr::from_bigint(input.into_bigint()).unwrap()).into_affine())
            .into_affine();
        acc
    })
}

#[cfg(test)]
mod test {
    use super::commit_native;
    use crate::generator::GENERATOR_CONTEXT;

    use ark_ec::short_weierstrass::Affine;
    use ark_ff::MontFp;
    use ark_std::One;
    use grumpkin::Fq;

    //TODO: double check that the generators are the same. They could be slightly different due to the way we canonically
    // decide how to invert y which was done to prevent a headache of having to deseialize an Fq element... Long story.
    #[test]
    fn commitment() {
        let res = commit_native(&[Fq::one(), Fq::one()], &mut GENERATOR_CONTEXT.lock().unwrap());
        let expected = Affine::new(
            // 2f7a8f9a6c96926682205fb73ee43215bf13523c19d7afe36f12760266cdfe15
            MontFp!(
                "21475250338311530111088781112432132511855209292730670949974692984887182229013"
            ),
            // 01916b316adbbf0e10e39b18c1d24b33ec84b46daddf72f43878bcc92b6057e6
            MontFp!("709245492126126701709902506217603794644991322680146492508959813283461748710"),
        );

        assert_eq!(res, expected);
    }
}
