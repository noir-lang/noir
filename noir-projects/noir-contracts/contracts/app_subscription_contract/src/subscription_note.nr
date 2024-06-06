use dep::aztec::prelude::{AztecAddress, PrivateContext, NoteHeader, NoteInterface};
use dep::aztec::{
    protocol_types::{constants::GENERATOR_INDEX__NOTE_NULLIFIER, grumpkin_point::GrumpkinPoint, hash::poseidon2_hash},
    note::utils::compute_note_hash_for_consumption, keys::getters::get_nsk_app
};

global SUBSCRIPTION_NOTE_LEN: Field = 3;
// ADDRESS_NOTE_LEN * 32 + 32(storage_slot as bytes) + 32(note_type_id as bytes)
global SUBSCRIPTION_NOTE_BYTES_LEN: Field = 3 * 32 + 64;

// Stores a public key composed of two fields
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
#[aztec(note)]
struct SubscriptionNote {
    // The nullifying public key hash is used with the nsk_app to ensure that the note can be privately spent.
    npk_m_hash: Field,
    expiry_block_number: Field,
    remaining_txs: Field,
}

impl NoteInterface<SUBSCRIPTION_NOTE_LEN, SUBSCRIPTION_NOTE_BYTES_LEN> for SubscriptionNote {
    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nsk_app(self.npk_m_hash);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_nsk_app(self.npk_m_hash);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field, ovpk_m: GrumpkinPoint, ivpk_m: GrumpkinPoint) {
        context.encrypt_and_emit_note(
            (*context).this_address(),
            slot,
            ovpk_m,
            ivpk_m,
            self,
        );
    }
}

impl SubscriptionNote {
    pub fn new(npk_m_hash: Field, expiry_block_number: Field, remaining_txs: Field) -> Self {
        SubscriptionNote { npk_m_hash, expiry_block_number, remaining_txs, header: NoteHeader::empty() }
    }
}
