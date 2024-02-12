use dep::aztec::{
    note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_consumption},
    oracle::{nullifier_key::get_nullifier_secret_key, get_public_key::get_public_key},
    log::emit_encrypted_log, hash::pedersen_hash, context::PrivateContext,
    protocol_types::{address::AztecAddress, traits::Empty}
};

// Shows how to create a custom note

global CARD_NOTE_LEN: Field = 3;

// docs:start:state_vars-CardNote
struct CardNote {
    points: u8,
    randomness: Field,
    owner: AztecAddress,
    header: NoteHeader,
}
// docs:end:state_vars-CardNote

impl CardNote {
    pub fn new(points: u8, randomness: Field, owner: AztecAddress) -> Self {
        CardNote { points, randomness, owner, header: NoteHeader::empty() }
    }
}

impl NoteInterface<CARD_NOTE_LEN> for CardNote {
    fn serialize_content(self) -> [Field; CARD_NOTE_LEN] {
        [self.points as Field, self.randomness, self.owner.to_field()]
    }

    fn deserialize_content(serialized_note: [Field; CARD_NOTE_LEN]) -> Self {
        CardNote {
            points: serialized_note[0] as u8,
            randomness: serialized_note[1],
            owner: AztecAddress::from_field(serialized_note[2]),
            header: NoteHeader::empty(),
        }
    }

    fn compute_note_content_hash(self) -> Field {
        pedersen_hash(self.serialize_content(), 0)
    }

    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.high,
            secret.low,
        ],0)
    }

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.high,
            secret.low,
        ],0)
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    fn get_header(note: CardNote) -> NoteHeader {
        note.header
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

    fn get_note_type_id() -> Field {
        // TODO(#4519): autogenerate
        // python -c "print(int(''.join(str(ord(c)) for c in 'CardNote')))"
        679711410078111116101
    }
}
