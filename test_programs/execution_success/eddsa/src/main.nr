use std::compat;
use std::ec::consts::te::baby_jubjub;
use std::ec::tecurve::affine::Point as TEPoint;
use std::eddsa::{eddsa_poseidon_verify, eddsa_to_pub, eddsa_verify};
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
