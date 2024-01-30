use dep::aztec::{
    protocol_types::{
        address::AztecAddress,
        traits::{Deserialize, Serialize}
    },
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    oracle::{
        rand::rand,
        nullifier_key::get_nullifier_secret_key,
        get_public_key::get_public_key,
    },
    log::emit_encrypted_log,
    hash::pedersen_hash,
    context::PrivateContext,
};

global VALUE_NOTE_LEN: Field = 3; // 3 plus a header.

// docs:start:value-note-def
struct ValueNote {
    value: Field,
    owner: AztecAddress,
    randomness: Field,
    header: NoteHeader,
}
// docs:end:value-note-def

impl Serialize<VALUE_NOTE_LEN> for ValueNote {
    fn serialize(self) -> [Field; VALUE_NOTE_LEN] {
        [self.value, self.owner.to_field(), self.randomness]
    }
}

impl Deserialize<VALUE_NOTE_LEN> for ValueNote {
    fn deserialize(serialized_note: [Field; VALUE_NOTE_LEN]) -> Self {
        ValueNote {
            value: serialized_note[0],
            owner: AztecAddress::from_field(serialized_note[1]),
            randomness: serialized_note[2],
            header: NoteHeader::empty(),
        }
    }
}

impl NoteInterface for ValueNote {

    fn compute_note_hash(self) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash(self.serialize(),0)
    }

    // docs:start:nullifier

    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(self);
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
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(self);
        let secret = get_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    fn get_header(self) -> NoteHeader {
        self.header
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            encryption_pub_key,
            self.serialize(),
        );
    }
}

impl ValueNote {
    pub fn new(value: Field, owner: AztecAddress) -> Self {
        let randomness = rand();
        let header = NoteHeader::empty();
        ValueNote {
            value,
            owner,
            randomness,
            header,
        }
    }
}
