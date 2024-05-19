use acvm_blackbox_solver::blake2s;
use ark_ec::{
    short_weierstrass::{Affine, SWCurveConfig},
    AffineRepr, CurveConfig, CurveGroup,
};
use ark_ff::{BigInteger, PrimeField, Zero};
use grumpkin::{Fq, GrumpkinParameters};

pub(crate) fn verify_signature(
    pub_key_x: Fq,
    pub_key_y: Fq,
    sig_s_bytes: [u8; 32],
    sig_e_bytes: [u8; 32],
    message: &[u8],
) -> bool {
    let pub_key = Affine::<GrumpkinParameters>::new_unchecked(pub_key_x, pub_key_y);

    // TODO: Check for correct subgroup isn't done in Barretenberg, is it necessary?
    if !pub_key.is_on_curve()
        || !pub_key.is_in_correct_subgroup_assuming_on_curve()
        || pub_key.is_zero()
    {
        return false;
    }

    let sig_s =
        <GrumpkinParameters as CurveConfig>::ScalarField::from_be_bytes_mod_order(&sig_s_bytes);
    let sig_e =
        <GrumpkinParameters as CurveConfig>::ScalarField::from_be_bytes_mod_order(&sig_e_bytes);

    if sig_s.is_zero() || sig_e.is_zero() {
        return false;
    }

    // R = g^{sig.s} • pub^{sig.e}
    let r = pub_key * sig_e + GrumpkinParameters::GENERATOR * sig_s;
    if r.is_zero() {
        // this result implies k == 0, which would be catastrophic for the prover.
        // it is a cheap check that ensures this doesn't happen.
        return false;
    }

    // compare the _hashes_ rather than field elements modulo r
    // e = H(pedersen(r, pk.x, pk.y), m), where r = x(R)
    let target_e_bytes = schnorr_generate_challenge(message, pub_key, r.into_affine());

    sig_e_bytes == target_e_bytes
}

fn schnorr_generate_challenge(
    message: &[u8],
    pubkey: Affine<GrumpkinParameters>,
    r: Affine<GrumpkinParameters>,
) -> [u8; 32] {
    // create challenge message pedersen_commitment(R.x, pubkey)

    let pedersen_hash = crate::pedersen::hash::hash_with_index(
        &[*r.x().unwrap(), *pubkey.x().unwrap(), *pubkey.y().unwrap()],
        0,
    );

    let mut hash_input: Vec<u8> = pedersen_hash.into_bigint().to_bytes_be();
    hash_input.extend(message);

    blake2s(&hash_input).unwrap()
}

#[cfg(test)]
mod schnorr_tests {
    use ark_ff::MontFp;

    use super::verify_signature;

    #[test]
    fn verifies_valid_signature() {
        // 0x04b260954662e97f00cab9adb773a259097f7a274b83b113532bce27fa3fb96a
        let pub_key_x: grumpkin::Fq =
            MontFp!("2124416763957513755957069320378814719427254224313784354193701269410464905578");
        // 0x2fd51571db6c08666b0edfbfbc57d432068bccd0110a39b166ab243da0037197
        let pub_key_y: grumpkin::Fq = MontFp!(
            "21635190314466406102464795369176917324283837527799356152433238205601767715223"
        );
        let sig_s_bytes: [u8; 32] = [
            1, 13, 119, 112, 212, 39, 233, 41, 84, 235, 255, 93, 245, 172, 186, 83, 157, 253, 76,
            77, 33, 128, 178, 15, 214, 67, 105, 107, 177, 234, 77, 48,
        ];
        let sig_e_bytes: [u8; 32] = [
            27, 237, 155, 84, 39, 84, 247, 27, 22, 8, 176, 230, 24, 115, 145, 220, 254, 122, 135,
            179, 171, 4, 214, 202, 64, 199, 19, 84, 239, 138, 124, 12,
        ];
        let message: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        assert_eq!(verify_signature(pub_key_x, pub_key_y, sig_s_bytes, sig_e_bytes, message), true)
    }

    #[test]
    fn rejects_zero_e() {
        // 0x04b260954662e97f00cab9adb773a259097f7a274b83b113532bce27fa3fb96a
        let pub_key_x: grumpkin::Fq =
            MontFp!("2124416763957513755957069320378814719427254224313784354193701269410464905578");
        // 0x2fd51571db6c08666b0edfbfbc57d432068bccd0110a39b166ab243da0037197
        let pub_key_y: grumpkin::Fq = MontFp!(
            "21635190314466406102464795369176917324283837527799356152433238205601767715223"
        );
        let sig_s_bytes: [u8; 32] = [
            1, 13, 119, 112, 212, 39, 233, 41, 84, 235, 255, 93, 245, 172, 186, 83, 157, 253, 76,
            77, 33, 128, 178, 15, 214, 67, 105, 107, 177, 234, 77, 48,
        ];
        let sig_e_bytes: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let message: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        assert_eq!(verify_signature(pub_key_x, pub_key_y, sig_s_bytes, sig_e_bytes, message), false)
    }

    #[test]
    fn rejects_zero_s() {
        // 0x04b260954662e97f00cab9adb773a259097f7a274b83b113532bce27fa3fb96a
        let pub_key_x: grumpkin::Fq =
            MontFp!("2124416763957513755957069320378814719427254224313784354193701269410464905578");
        // 0x2fd51571db6c08666b0edfbfbc57d432068bccd0110a39b166ab243da0037197
        let pub_key_y: grumpkin::Fq = MontFp!(
            "21635190314466406102464795369176917324283837527799356152433238205601767715223"
        );
        let sig_s_bytes: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let sig_e_bytes: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let message: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        assert_eq!(verify_signature(pub_key_x, pub_key_y, sig_s_bytes, sig_e_bytes, message), false)
    }
}
