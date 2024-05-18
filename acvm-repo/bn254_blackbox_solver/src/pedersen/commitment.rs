// Taken from: https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/crypto/pedersen/pederson.rs

use ark_ec::{short_weierstrass::Affine, AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use grumpkin::{Fq, Fr, GrumpkinParameters};

use crate::generator::generators::{derive_generators, DEFAULT_DOMAIN_SEPARATOR};

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
pub(crate) fn commit_native_with_index(
    inputs: &[Fq],
    starting_index: u32,
) -> Affine<GrumpkinParameters> {
    let generators =
        derive_generators(DEFAULT_DOMAIN_SEPARATOR, inputs.len() as u32, starting_index);

    inputs.iter().enumerate().fold(Affine::zero(), |mut acc, (i, input)| {
        //TODO: this is a sketch conversion do better
        acc = (acc + (generators[i] * Fr::from_bigint(input.into_bigint()).unwrap()).into_affine())
            .into_affine();
        acc
    })
}

#[cfg(test)]
mod test {

    use ark_ec::short_weierstrass::Affine;
    use ark_ff::MontFp;
    use ark_std::{One, Zero};
    use grumpkin::Fq;

    use crate::pedersen::commitment::commit_native_with_index;

    #[test]
    fn commitment() {
        let res = commit_native_with_index(&[Fq::one(), Fq::one()], 0);
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

    #[test]
    fn commitment_with_zero() {
        let res = commit_native_with_index(&[Fq::zero(), Fq::one()], 0);
        let expected = Affine::new(
            // 054aa86a73cb8a34525e5bbed6e43ba1198e860f5f3950268f71df4591bde402
            MontFp!("2393473289045184898987089634332637236754766663897650125720167164137088869378"),
            // 209dcfbf2cfb57f9f6046f44d71ac6faf87254afc7407c04eb621a6287cac126
            MontFp!(
                "14752839959415467457196082350231122454649853219840744672802853620609001898278"
            ),
        );

        assert_eq!(res, expected);
    }
}
