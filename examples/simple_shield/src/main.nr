

use dep::std;

fn main(
  // Public key of note
  // all notes have the same denomination
  priv_key : Field, 

  // Merkle membership proof
  note_root : pub Field, 
  index : Field,
  note_hash_path : [32]Field,

  // Receiver public key
  to_pubkey_x : Field, 
  to_pubkey_y : Field, 
) {

    // Compute public key from private key to show ownership
    //
    let pubkey = std::scalar_mul::fixed_base(priv_key);
    priv pubkey_x = pubkey[0];
    priv pubkey_y = pubkey[1];

    // Compute input note commitment
    priv note_commitment = std::hash::pedersen([pubkey_x, pubkey_y]);
    
    // Compute input note nullifier
    priv nullifier = std::hash::pedersen([note_commitment, index, priv_key]);

    // Compute output note nullifier
    priv receiver_note_commitment = std::hash::pedersen([to_pubkey_x, to_pubkey_y]);

    // Check that the input note nullifier is in the root
    priv is_member = std::merkle::check_membership(note_root, note_commitment, index, note_hash_path);
    constrain is_member == 1;

    std::set_as_public(receiver_note_commitment);
    std::set_as_public(nullifier);
}