use dep::aztec::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::compute_note_hash_for_read_or_nullify,
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

    fn serialise(self) -> [Field; PROFILE_NOTE_LEN] {
        [self.avatar, self.xp]
    }

    fn deserialise(preimage: [Field; PROFILE_NOTE_LEN]) -> Self {
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

fn deserialise(preimage: [Field; PROFILE_NOTE_LEN]) -> ProfileNote {
    ProfileNote::deserialise(preimage)
}

fn serialise(note: ProfileNote) -> [Field; PROFILE_NOTE_LEN] {
    note.serialise()
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
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
};
