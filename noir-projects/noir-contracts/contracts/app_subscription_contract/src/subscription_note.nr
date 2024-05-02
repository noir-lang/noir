use dep::aztec::prelude::{AztecAddress, PrivateContext, NoteHeader, NoteInterface};
use dep::aztec::{
    protocol_types::constants::GENERATOR_INDEX__NOTE_NULLIFIER, note::utils::compute_note_hash_for_consumption,
    hash::poseidon2_hash,
    oracle::{nullifier_key::get_app_nullifier_secret_key, get_public_key::get_public_key}
};

global SUBSCRIPTION_NOTE_LEN: Field = 3;

// Stores a public key composed of two fields
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
#[aztec(note)]
struct SubscriptionNote {
    owner: AztecAddress,
    expiry_block_number: Field,
    remaining_txs: Field,
}

impl NoteInterface<SUBSCRIPTION_NOTE_LEN> for SubscriptionNote {
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
        context.emit_encrypted_log(
            (*context).this_address(),
            slot,
            Self::get_note_type_id(),
            encryption_pub_key,
            self.serialize_content(),
        );
    }
}

impl SubscriptionNote {
    pub fn new(owner: AztecAddress, expiry_block_number: Field, remaining_txs: Field) -> Self {
        SubscriptionNote { owner, expiry_block_number, remaining_txs, header: NoteHeader::empty() }
    }
}
