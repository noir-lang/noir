use dep::aztec::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
};

global RULES_NOTE_LEN: Field = 2;

struct RulesNote {
    min_points: u8,
    max_points: u8,
    header: NoteHeader,
}

impl RulesNote {
    fn new(min_points: u8, max_points: u8) -> Self {
        RulesNote {
            min_points,
            max_points,
            header: NoteHeader::empty(),
        }
    }

    fn serialize(self) -> [Field; RULES_NOTE_LEN] {
        [self.min_points as Field, self.max_points as Field]
    }

    fn deserialize(preimage: [Field; RULES_NOTE_LEN]) -> Self {
        RulesNote {
            min_points: preimage[0] as u8,
            max_points: preimage[1] as u8,
            header: NoteHeader::empty(),
        }
    }

    fn compute_note_hash(self) -> Field {
        dep::std::hash::pedersen([
            self.min_points as Field, 
            self.max_points as Field, 
        ])[0]
    }

    fn compute_nullifier(_self: Self) -> Field {
        // Not used
        0
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }
}

fn deserialize(preimage: [Field; RULES_NOTE_LEN]) -> RulesNote {
    RulesNote::deserialize(preimage)
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

global RulesNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
};
