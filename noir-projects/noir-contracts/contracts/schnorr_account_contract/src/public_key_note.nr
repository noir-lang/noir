use dep::aztec::prelude::{AztecAddress, NoteHeader, NoteInterface, PrivateContext, emit_encrypted_log};
use dep::aztec::{
    note::utils::compute_note_hash_for_consumption, hash::poseidon2_hash,
    oracle::{nullifier_key::get_app_nullifier_secret_key, get_public_key::get_public_key},
    protocol_types::constants::GENERATOR_INDEX__NOTE_NULLIFIER,
};

global PUBLIC_KEY_NOTE_LEN: Field = 3;

// Stores a public key composed of two fields
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
#[aztec(note)]
struct PublicKeyNote {
    x: Field,
    y: Field,
    owner: AztecAddress, // We store the owner address only to get the secret key to compute the nullifier and to broadcast
}

impl NoteInterface<PUBLIC_KEY_NOTE_LEN> for PublicKeyNote {
    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_app_nullifier_secret_key(self.owner);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_app_nullifier_secret_key(self.owner);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            Self::get_note_type_id(),
            encryption_pub_key,
            self.serialize_content(),
        );
    }
}

impl PublicKeyNote {
    pub fn new(x: Field, y: Field, owner: AztecAddress) -> Self {
        PublicKeyNote { x, y, owner, header: NoteHeader::empty() }
    }
}
