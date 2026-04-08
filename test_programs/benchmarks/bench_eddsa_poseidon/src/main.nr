use poseidon::poseidon::PoseidonHasher;
use std::default::Default;
use std::hash::Hasher;

use edwards::{bjj::BabyJubJub, Curve, CurveTrait, ScalarField};

// Baby JubJub subgroup order
global SUBORDER: Field =
    2736030358979909402780800718157159386076813972158567259200215660948447373041;

// Generator * 8
global BASE8_X: Field =
    5299619240641551281634865583518297030282874472190772894086521144482721001553;
global BASE8_Y: Field =
    16950150798460657717958625567821834550301663161624707787222815936182638968203;

fn main(
    msg: pub Field,
    pub_key_x: Field,
    pub_key_y: Field,
    r8_x: Field,
    r8_y: Field,
    s: Field,
) -> pub bool {
    eddsa_verify::<PoseidonHasher>(pub_key_x, pub_key_y, s, r8_x, r8_y, msg)
}

pub fn eddsa_verify<H>(
    pub_key_x: Field,
    pub_key_y: Field,
    signature_s: Field,
    signature_r8_x: Field,
    signature_r8_y: Field,
    message: Field,
) -> bool
where
    H: Hasher + Default,
{
    // Verifies by testing:
    // S * B8 = R8 + H(R8, A, m) * A8

    // BabyJubJub::new asserts the point is on the curve
    let pub_key = BabyJubJub::new(pub_key_x, pub_key_y);
    let signature_r8 = BabyJubJub::new(signature_r8_x, signature_r8_y);
    // Ensure S < Subgroup Order
    assert(signature_s.lt(SUBORDER));
    // Calculate the h = H(R, A, msg)
    let mut hasher = H::default();
    hasher.write(signature_r8_x);
    hasher.write(signature_r8_y);
    hasher.write(pub_key_x);
    hasher.write(pub_key_y);
    hasher.write(message);
    let hash: Field = hasher.finish();
    // Calculate second part of the right side:  right2 = h*8*A
    // Multiply by 8 by doubling 3 times. This also ensures that the result is in the subgroup.
    let pub_key_mul_2 = pub_key.dbl();
    let pub_key_mul_4 = pub_key_mul_2.dbl();
    let pub_key_mul_8 = pub_key_mul_4.dbl();
    // We check that A8 is not zero.
    assert(!pub_key_mul_8.is_zero());
    // Compute the right side: R8 + h * A8
    let right = signature_r8 + pub_key_mul_8.mul(ScalarField::<64>::from(hash));
    // Calculate left side of equation left = S * B8
    let base8: BabyJubJub = Curve { x: BASE8_X, y: BASE8_Y };
    let left = base8.mul(ScalarField::<63>::from(signature_s));

    left == right
}
