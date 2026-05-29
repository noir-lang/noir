fn main(
    device_secret: Field,
    commitment: pub Field,
    challenge: pub Field,
    user_id_hash: pub Field,
) -> pub Field {
    let computed_commitment = device_secret * device_secret + user_id_hash;
    assert(computed_commitment == commitment);

    device_secret * challenge + user_id_hash
}
