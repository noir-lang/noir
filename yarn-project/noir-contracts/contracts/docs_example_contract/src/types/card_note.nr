use dep::aztec::protocol_types::address::AztecAddress;
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    oracle::{
        nullifier_key::get_nullifier_secret_key,
        get_public_key::get_public_key,
    },
    log::emit_encrypted_log,
    hash::pedersen_hash,
    context::PrivateContext,
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
        CardNote {
            points,
            randomness,
            owner,
            header: NoteHeader::empty(),
        }
    }

    pub fn serialize(self) -> [Field; CARD_NOTE_LEN] {
        [self.points as Field, self.randomness, self.owner.to_field()]
    }

    pub fn deserialize(serialized_note: [Field; CARD_NOTE_LEN]) -> Self {
        CardNote {
            points: serialized_note[0] as u8,
            randomness: serialized_note[1],
            owner: AztecAddress::from_field(serialized_note[2]),
            header: NoteHeader::empty(),
        }
    }

    pub fn compute_note_hash(self) -> Field {
        pedersen_hash([
            self.points as Field, 
            self.randomness,
            self.owner.to_field(),
        ],0)
    }

    pub fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(CardNoteMethods, self);
        let secret = context.request_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.high,
            secret.low,
        ],0)
    }

    pub fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(CardNoteMethods, self);
        let secret = get_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.high,
            secret.low,
        ],0)
    }

    pub fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    // Broadcasts the note as an encrypted log on L1.
    pub fn broadcast(self, context: &mut PrivateContext, slot: Field) {
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

fn deserialize(serialized_note: [Field; CARD_NOTE_LEN]) -> CardNote {
    CardNote::deserialize(serialized_note)
}

fn serialize(note: CardNote) -> [Field; CARD_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: CardNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: CardNote, context: &mut PrivateContext) -> Field {
    note.compute_nullifier(context)
}

fn compute_nullifier_without_context(note: CardNote) -> Field {
    note.compute_nullifier_without_context()
}

fn get_header(note: CardNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut CardNote, header: NoteHeader) {
    note.set_header(header)
}

// Broadcasts the note as an encrypted log on L1.
fn broadcast(context: &mut PrivateContext, slot: Field, note: CardNote) {
    note.broadcast(context, slot);
}

global CardNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    compute_nullifier_without_context,
    get_header,
    set_header,
    broadcast,
};
