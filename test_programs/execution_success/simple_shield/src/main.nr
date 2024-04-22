use dep::std;

fn main(
    // Public key of note
  // all notes have the same denomination
    priv_key: field,
    // Merkle membership proof
    note_root: pub field,
    index: field,
    note_hash_path: [field; 3],
    // Receiver public key
    to_pubkey_x: field,
    to_pubkey_y: field
) -> pub [field; 2] {
    // Compute public key from private key to show ownership
    let pubkey = std::scalar_mul::fixed_base_embedded_curve(priv_key, 0);
    let pubkey_x = pubkey[0];
    let pubkey_y = pubkey[1];
    // Compute input note commitment
    let note_commitment = std::hash::pedersen_commitment([pubkey_x, pubkey_y]);
    // Compute input note nullifier
    let nullifier = std::hash::pedersen_commitment([note_commitment.x, index, priv_key]);
    // Compute output note nullifier
    let receiver_note_commitment = std::hash::pedersen_commitment([to_pubkey_x, to_pubkey_y]);
    // Check that the input note nullifier is in the root
    assert(note_root == std::merkle::compute_merkle_root(note_commitment.x, index, note_hash_path));

    [nullifier.x, receiver_note_commitment.x]
}
