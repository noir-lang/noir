use std::compat;
use std::ec::consts::te::baby_jubjub;
use std::ec::tecurve::affine::Point as TEPoint;
use std::hash::Hasher;
use std::hash::poseidon::PoseidonHasher;
use std::hash::poseidon2::Poseidon2Hasher;

fn main(msg: pub Field, _priv_key_a: Field, _priv_key_b: Field) {
    // Skip this test for non-bn254 backends
    if compat::is_bn254() {
        let bjj = baby_jubjub();

        let pub_key_a = bjj.curve.mul(_priv_key_a, bjj.curve.gen);
        let pub_key_b = bjj.curve.mul(_priv_key_b, bjj.curve.gen);
        let (pub_key_a_x, pub_key_a_y) = eddsa_to_pub(_priv_key_a);
        let (pub_key_b_x, pub_key_b_y) = eddsa_to_pub(_priv_key_b);
        assert(TEPoint::new(pub_key_a_x, pub_key_a_y) == pub_key_a);
        assert(TEPoint::new(pub_key_b_x, pub_key_b_y) == pub_key_b);
        // Manually computed as fields can't use modulo. Importantantly the commitment is within
        // the subgroup order. Note that choice of hash is flexible for this step.
        // let r_a = hash::pedersen_commitment([_priv_key_a, msg])[0] % bjj.suborder; // modulus computed manually
        let r_a = 1414770703199880747815475415092878800081323795074043628810774576767372531818;
        // let r_b = hash::pedersen_commitment([_priv_key_b, msg])[0] % bjj.suborder; // modulus computed manually
        let r_b = 571799555715456644614141527517766533395606396271089506978608487688924659618;

        let r8_a = bjj.curve.mul(r_a, bjj.base8);
        let r8_b = bjj.curve.mul(r_b, bjj.base8);
        // let h_a: [Field; 6] = hash::poseidon::bn254::hash_5([
        //     r8_a.x,
        //     r8_a.y,
        //     pub_key_a.x,
        //     pub_key_a.y,
        //     msg,
        // ]);
        // let h_b: [Field; 6] = hash::poseidon::bn254::hash_5([
        //     r8_b.x,
        //     r8_b.y,
        //     pub_key_b.x,
        //     pub_key_b.y,
        //     msg,
        // ]);
        // let s_a = (r_a + _priv_key_a * h_a) % bjj.suborder; // modulus computed manually
        let s_a = 30333430637424319196043722294837632681219980330991241982145549329256671548;
        // let s_b = (r_b + _priv_key_b * h_b) % bjj.suborder; // modulus computed manually
        let s_b = 1646085314320208098241070054368798527940102577261034947654839408482102287019;
        // User A verifies their signature over the message
        assert(eddsa_poseidon_verify(pub_key_a.x, pub_key_a.y, s_a, r8_a.x, r8_a.y, msg));
        // User B's signature over the message can't be used with user A's pub key
        assert(!eddsa_poseidon_verify(pub_key_a.x, pub_key_a.y, s_b, r8_b.x, r8_b.y, msg));
        // User A's signature over the message can't be used with another message
        assert(!eddsa_poseidon_verify(pub_key_a.x, pub_key_a.y, s_a, r8_a.x, r8_a.y, msg + 1));
        // Using a different hash should fail
        assert(
            !eddsa_verify::<Poseidon2Hasher>(pub_key_a.x, pub_key_a.y, s_a, r8_a.x, r8_a.y, msg),
        );
    }
}

// Returns true if signature is valid
pub fn eddsa_poseidon_verify(
    pub_key_x: Field,
    pub_key_y: Field,
    signature_s: Field,
    signature_r8_x: Field,
    signature_r8_y: Field,
    message: Field,
) -> bool {
    eddsa_verify::<PoseidonHasher>(
        pub_key_x,
        pub_key_y,
        signature_s,
        signature_r8_x,
        signature_r8_y,
        message,
    )
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
    let bjj = baby_jubjub();

    let pub_key = TEPoint::new(pub_key_x, pub_key_y);
    assert(bjj.curve.contains(pub_key));

    let signature_r8 = TEPoint::new(signature_r8_x, signature_r8_y);
    assert(bjj.curve.contains(signature_r8));
    // Ensure S < Subgroup Order
    assert(signature_s.lt(bjj.suborder));
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

// Returns the public key of the given secret key as (pub_key_x, pub_key_y)
pub fn eddsa_to_pub(secret: Field) -> (Field, Field) {
    let bjj = baby_jubjub();
    let pub_key = bjj.curve.mul(secret, bjj.curve.gen);
    (pub_key.x, pub_key.y)
}
