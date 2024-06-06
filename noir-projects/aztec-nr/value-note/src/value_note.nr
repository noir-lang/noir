use dep::aztec::{
    protocol_types::{
    address::AztecAddress, grumpkin_point::GrumpkinPoint, traits::{Deserialize, Serialize},
    constants::GENERATOR_INDEX__NOTE_NULLIFIER, hash::poseidon2_hash
},
    note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_consumption},
    oracle::unsafe_rand::unsafe_rand, keys::getters::get_nsk_app, context::PrivateContext
};

global VALUE_NOTE_LEN: Field = 3; // 3 plus a header.
// VALUE_NOTE_LEN * 32 + 32(storage_slot as bytes) + 32(note_type_id as bytes)
global VALUE_NOTE_BYTES_LEN: Field = 3 * 32 + 64;

// docs:start:value-note-def
#[aztec(note)]
struct ValueNote {
    value: Field,
   // The nullifying public key hash is used with the nsk_app to ensure that the note can be privately spent.
    npk_m_hash: Field,
    randomness: Field,
}
// docs:end:value-note-def

impl NoteInterface<VALUE_NOTE_LEN, VALUE_NOTE_BYTES_LEN> for ValueNote {
    // docs:start:nullifier

    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nsk_app(self.npk_m_hash);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    // docs:end:nullifier

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

impl ValueNote {
    pub fn new(value: Field, npk_m_hash: Field) -> Self {
        let randomness = unsafe_rand();
        let header = NoteHeader::empty();
        ValueNote { value, npk_m_hash, randomness, header }
    }
}

impl Serialize<7> for ValueNote {
    fn serialize(self) -> [Field; 7] {
        let header = self.header.serialize();

        [self.value, self.npk_m_hash, self.randomness, header[0], header[1], header[2], header[3]]
    }
}
