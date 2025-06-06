fn main(
    // Public key of note
    // all notes have the same denomination
    priv_key: Field,
    // Merkle membership proof
    note_root: pub Field,
    index: Field,
    note_hash_path: [Field; 3],
    // Receiver public key
    to_pubkey_x: Field,
    to_pubkey_y: Field,
) -> pub [Field; 2] {
    let priv_key_as_scalar = std::embedded_curve_ops::EmbeddedCurveScalar { lo: priv_key, hi: 0 };
    // Compute public key from private key to show ownership
    let pubkey = std::embedded_curve_ops::fixed_base_scalar_mul(priv_key_as_scalar);
    // Compute input note commitment
    let note_commitment = std::hash::pedersen_commitment([pubkey.x, pubkey.y]);
    // Compute input note nullifier
    let nullifier = std::hash::pedersen_commitment([note_commitment.x, index, priv_key]);
    // Compute output note nullifier
    let receiver_note_commitment = std::hash::pedersen_commitment([to_pubkey_x, to_pubkey_y]);
    // Check that the input note nullifier is in the root
    assert(note_root == compute_merkle_root(note_commitment.x, index, note_hash_path));

    [nullifier.x, receiver_note_commitment.x]
}

fn compute_merkle_root<let N: u32>(leaf: Field, index: Field, hash_path: [Field; N]) -> Field {
    let index_bits: [u1; N] = index.to_le_bits();
    let mut current = leaf;
    for i in 0..N {
        let path_bit = index_bits[i] != 0;
        let (hash_left, hash_right) = if path_bit {
            (hash_path[i], current)
        } else {
            (current, hash_path[i])
        };
        current = std::hash::pedersen_hash([hash_left, hash_right]);
    }
    current
}
