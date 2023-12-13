use dep::protocol_types::address::AztecAddress;
use dep::std::option::Option;
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
    },
    oracle::get_public_key::get_public_key,
    log::emit_encrypted_log,
    hash::pedersen_hash,
    context::PrivateContext,
};

global RULES_NOTE_LEN: Field = 2;

struct RulesNote {
    min_points: u8,
    max_points: u8,
    maybe_owner: Option<AztecAddress>,
    header: NoteHeader,
}

impl RulesNote {
    pub fn new(min_points: u8, max_points: u8, maybe_owner: Option<AztecAddress>) -> Self {
        RulesNote {
            min_points,
            max_points,
            maybe_owner,
            header: NoteHeader::empty(),
        }
    }

    pub fn serialize(self) -> [Field; RULES_NOTE_LEN] {
        [self.min_points as Field, self.max_points as Field]
    }

    pub fn deserialize(serialized_note: [Field; RULES_NOTE_LEN]) -> Self {
        RulesNote {
            min_points: serialized_note[0] as u8,
            max_points: serialized_note[1] as u8,
            maybe_owner: Option::none(),
            header: NoteHeader::empty(),
        }
    }

    pub fn compute_note_hash(self) -> Field {
        pedersen_hash([
            self.min_points as Field, 
            self.max_points as Field, 
        ],0)
    }

    pub fn compute_nullifier(_self: Self) -> Field {
        // Not used
        0
    }

    pub fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    pub fn set_owner(&mut self, owner: AztecAddress) {
        self.maybe_owner = Option::some(owner);
    }

    // Broadcasts the note as an encrypted log on L1.
    pub fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        assert(self.maybe_owner.is_some(), "Note owner must be set when the broadcast flow is triggered.");
        let owner = self.maybe_owner.unwrap_unchecked();

        let encryption_pub_key = get_public_key(owner);
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            encryption_pub_key,
            self.serialize(),
        );
    }
}

fn deserialize(serialized_note: [Field; RULES_NOTE_LEN]) -> RulesNote {
    RulesNote::deserialize(serialized_note)
}

fn serialize(note: RulesNote) -> [Field; RULES_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: RulesNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: RulesNote) -> Field {
    note.compute_nullifier()
}

fn get_header(note: RulesNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut RulesNote, header: NoteHeader) {
    note.set_header(header)
}

// Broadcasts the note as an encrypted log on L1.
fn broadcast(context: &mut PrivateContext, slot: Field, note: RulesNote) {
    note.broadcast(context, slot);
}

global RulesNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
    broadcast,
};
