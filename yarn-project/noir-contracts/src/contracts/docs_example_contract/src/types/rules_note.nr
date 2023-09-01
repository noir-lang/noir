use dep::aztec::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::compute_note_hash_for_read_or_nullify,
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

    fn serialise(self) -> [Field; RULES_NOTE_LEN] {
        [self.min_points as Field, self.max_points as Field]
    }

    fn deserialise(preimage: [Field; RULES_NOTE_LEN]) -> Self {
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

fn deserialise(preimage: [Field; RULES_NOTE_LEN]) -> RulesNote {
    RulesNote::deserialise(preimage)
}

fn serialise(note: RulesNote) -> [Field; RULES_NOTE_LEN] {
    note.serialise()
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
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
};
