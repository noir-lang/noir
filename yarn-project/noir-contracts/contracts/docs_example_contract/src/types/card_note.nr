use dep::protocol_types::address::AztecAddress;
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    oracle::{
        get_secret_key::get_secret_key,
        get_public_key::get_public_key,
    },
    log::emit_encrypted_log,
    hash::pedersen_hash,
    context::PrivateContext,
};

global CARD_NOTE_LEN: Field = 3;

// docs:start:state_vars-CardNote
struct CardNote {
    points: u8,
    secret: Field,
    owner: AztecAddress,
    header: NoteHeader,
}
// docs:end:state_vars-CardNote

impl CardNote {
    pub fn new(points: u8, secret: Field, owner: AztecAddress) -> Self {
        CardNote {
            points,
            secret,
            owner,
            header: NoteHeader::empty(),
        }
    }

    pub fn serialize(self) -> [Field; CARD_NOTE_LEN] {
        [self.points as Field, self.secret, self.owner.to_field()]
    }

    pub fn deserialize(serialized_note: [Field; CARD_NOTE_LEN]) -> Self {
        CardNote {
            points: serialized_note[0] as u8,
            secret: serialized_note[1],
            owner: AztecAddress::from_field(serialized_note[2]),
            header: NoteHeader::empty(),
        }
    }

    pub fn compute_note_hash(self) -> Field {
        pedersen_hash([
            self.points as Field, 
            self.secret,
            self.owner.to_field(),
        ],0)
    }

    pub fn compute_nullifier(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(CardNoteMethods, self);
        let secret = get_secret_key(self.owner);
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

fn compute_nullifier(note: CardNote) -> Field {
    note.compute_nullifier()
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
    get_header,
    set_header,
    broadcast,
};
