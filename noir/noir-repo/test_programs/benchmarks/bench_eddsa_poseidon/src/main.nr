use std::eddsa::eddsa_poseidon_verify;

fn main(
    msg: pub Field,
    pub_key_x: Field,
    pub_key_y: Field,
    r8_x: Field,
    r8_y: Field,
    s: Field
) -> pub bool {
    eddsa_poseidon_verify(pub_key_x, pub_key_y, s, r8_x, r8_y, msg)
}
