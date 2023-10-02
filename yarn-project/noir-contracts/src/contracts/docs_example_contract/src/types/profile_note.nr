use dep::aztec::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
};

global PROFILE_NOTE_LEN: Field = 2;

struct ProfileNote {
    avatar: Field,
    xp: Field,
    header: NoteHeader,
}

impl ProfileNote {
    fn new(avatar: Field, xp: Field) -> Self {
        ProfileNote {
            avatar,
            xp,
            header: NoteHeader::empty(),
        }
    }

    fn serialize(self) -> [Field; PROFILE_NOTE_LEN] {
        [self.avatar, self.xp]
    }

    fn deserialize(preimage: [Field; PROFILE_NOTE_LEN]) -> Self {
        ProfileNote {
            avatar: preimage[1],
            xp: preimage[0],
            header: NoteHeader::empty(),
        }
    }

    fn compute_note_hash(self) -> Field {
        dep::std::hash::pedersen([
            self.avatar, 
            self.xp,
        ])[0]
    }

    fn compute_nullifier(_self: Self) -> Field {
        assert(false); // Not allowed.
        0
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }
}

fn deserialize(preimage: [Field; PROFILE_NOTE_LEN]) -> ProfileNote {
    ProfileNote::deserialize(preimage)
}

fn serialize(note: ProfileNote) -> [Field; PROFILE_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: ProfileNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: ProfileNote) -> Field {
    note.compute_nullifier()
}

fn get_header(note: ProfileNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut ProfileNote, header: NoteHeader) {
    note.set_header(header)
}

global ProfileNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
};
