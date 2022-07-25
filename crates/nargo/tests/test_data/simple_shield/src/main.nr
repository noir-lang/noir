use dep::std;

fn main(
  // Public key of note
  // all notes have the same denomination
  priv_key : Field, 

  // Merkle membership proof
  note_root : pub Field, 
  index : Field,
  note_hash_path : [3]Field,

  // Receiver public key
  to_pubkey_x : Field, 
  to_pubkey_y : Field, 
) -> pub [2]Field {

    // Compute public key from private key to show ownership
    let pubkey = std::scalar_mul::fixed_base(priv_key);
    let pubkey_x = pubkey[0];
    let pubkey_y = pubkey[1];

    // Compute input note commitment
    let note_commitment = std::hash::pedersen([pubkey_x, pubkey_y]);
    
    // Compute input note nullifier
    let nullifier = std::hash::pedersen([note_commitment[0], index, priv_key]);

    // Compute output note nullifier
    let receiver_note_commitment = std::hash::pedersen([to_pubkey_x, to_pubkey_y]);

    // Check that the input note nullifier is in the root
    let is_member = std::merkle::check_membership(note_root, note_commitment[0], index, note_hash_path);
    constrain is_member == 1;

    [nullifier[0], receiver_note_commitment[0]]
}
