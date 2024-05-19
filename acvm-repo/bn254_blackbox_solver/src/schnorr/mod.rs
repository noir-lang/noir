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
