use crate::hash::poseidon;
use crate::ec::consts::te::baby_jubjub;
use crate::ec::tecurve::affine::Point as TEPoint;

// Returns true if signature is valid
pub fn eddsa_poseidon_verify(
    pub_key_x: Field,
    pub_key_y: Field,
    signature_s: Field,
    signature_r8_x: Field,
    signature_r8_y: Field,
    message: Field
) -> bool {
    // Verifies by testing:
    // S * B8 = R8 + H(R8, A, m) * A8
    let bjj = baby_jubjub();

    let pub_key = TEPoint::new(pub_key_x, pub_key_y);
    assert(bjj.curve.contains(pub_key));

    let signature_r8 = TEPoint::new(signature_r8_x, signature_r8_y);
    assert(bjj.curve.contains(signature_r8));
    // Ensure S < Subgroup Order
    assert(signature_s.lt(bjj.suborder));
    // Calculate the h = H(R, A, msg)
    let hash: Field = poseidon::bn254::hash_5([signature_r8_x, signature_r8_y, pub_key_x, pub_key_y, message]);
    // Calculate second part of the right side:  right2 = h*8*A
    // Multiply by 8 by doubling 3 times. This also ensures that the result is in the subgroup.
    let pub_key_mul_2 = bjj.curve.add(pub_key, pub_key);
    let pub_key_mul_4 = bjj.curve.add(pub_key_mul_2, pub_key_mul_2);
    let pub_key_mul_8 = bjj.curve.add(pub_key_mul_4, pub_key_mul_4);
    // We check that A8 is not zero.
    assert(!pub_key_mul_8.is_zero());
    // Compute the right side: R8 + h * A8
    let right = bjj.curve.add(signature_r8, bjj.curve.mul(hash, pub_key_mul_8));
    // Calculate left side of equation left = S * B8
    let left = bjj.curve.mul(signature_s, bjj.base8);

    left.eq(right)
}
