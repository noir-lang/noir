use dep::aztec::prelude::{AztecAddress, NoteInterface, NoteHeader, PrivateContext};
use dep::aztec::{
    keys::getters::get_ivpk_m, note::{utils::compute_note_hash_for_consumption},
    keys::getters::get_nsk_app,
    protocol_types::{
    traits::Empty, grumpkin_point::GrumpkinPoint, constants::GENERATOR_INDEX__NOTE_NULLIFIER,
    hash::poseidon2_hash
}
};

// Shows how to create a custom note

global CARD_NOTE_LEN: Field = 3;

// docs:start:state_vars-CardNote
#[aztec(note)]
struct CardNote {
    points: u8,
    randomness: Field,
    // The nullifying public key hash is used with the nsk_app to ensure that the note can be privately spent.
    npk_m_hash: Field,
}
// docs:end:state_vars-CardNote

impl CardNote {
    pub fn new(points: u8, randomness: Field, npk_m_hash: Field) -> Self {
        CardNote { points, randomness, npk_m_hash, header: NoteHeader::empty() }
    }
}

impl NoteInterface<CARD_NOTE_LEN> for CardNote {
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
    fn broadcast(self, context: &mut PrivateContext, slot: Field, ivpk_m: GrumpkinPoint) {
        context.emit_note_encrypted_log(
            (*context).this_address(),
            slot,
            Self::get_note_type_id(),
            ivpk_m,
            self,
        );
    }
}
