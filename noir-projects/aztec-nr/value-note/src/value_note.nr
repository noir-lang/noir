use dep::aztec::{
    protocol_types::{address::AztecAddress, traits::{Deserialize, Serialize}},
    note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_consumption},
    oracle::{unsafe_rand::unsafe_rand, nullifier_key::get_nullifier_secret_key, get_public_key::get_public_key},
    log::emit_encrypted_log, hash::pedersen_hash, context::PrivateContext
};

global VALUE_NOTE_LEN: Field = 3; // 3 plus a header.

// docs:start:value-note-def
#[aztec(note)]
struct ValueNote {
    value: Field,
    owner: AztecAddress,
    randomness: Field,
}
// docs:end:value-note-def

impl NoteInterface<VALUE_NOTE_LEN> for ValueNote {
    // docs:start:nullifier

    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
    }

    // docs:end:nullifier

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
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

impl ValueNote {
    pub fn new(value: Field, owner: AztecAddress) -> Self {
        let randomness = unsafe_rand();
        let header = NoteHeader::empty();
        ValueNote { value, owner, randomness, header }
    }
}

impl Serialize<7> for ValueNote {
    fn serialize(self) -> [Field; 7] {
        let header = self.header.serialize();

        [self.value, self.owner.to_field(), self.randomness, header[0], header[1], header[2], header[3]]
    }
}
