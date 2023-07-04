use dep::aztec::note::note_interface::NoteInterface;
use dep::aztec::oracle::rand::rand;
use dep::aztec::oracle::get_secret_key::get_secret_key;
use dep::aztec::types::point::Point;
use crate::generators;

global VALUE_NOTE_LEN: Field = 6;

struct ValueNote {
    value: Field,
    owner: Point,
    randomness: Field,
    nonce: Field,
    is_real: bool
}

impl ValueNote {
    fn new(value: Field, owner: Point) -> Self {
        let randomness = rand();
        let nonce = 3;      // Guaranteed to be random
        ValueNote {
            value,
            owner,
            randomness,
            nonce,
            is_real: true,
        }
    }

    fn serialise(self) -> [Field; VALUE_NOTE_LEN] {
        [self.value, self.owner.x, self.owner.y, self.randomness, self.nonce, self.is_real as Field]
    }

    fn dummy() -> Self {
        ValueNote {
            value: 0,
            owner: Point { x: 0, y: 0 },
            randomness: 0,
            nonce: 0,
            is_real: false,
        }
    }

    fn is_dummy(self) -> bool {
        !self.is_real
    }

    fn validate(self, sender: Point) {
        let sender_equals = (sender.x == self.owner.x) & (sender.y == self.owner.y);
        assert((!self.is_real) | sender_equals);
    }
}

fn deserialise(preimage: [Field; VALUE_NOTE_LEN]) -> ValueNote {
    ValueNote {
        value: preimage[0],
        owner: Point {
            x: preimage[1],
            y: preimage[2]
        },
        randomness: preimage[3],
        nonce: preimage[4],
        is_real: preimage[5] as bool,
    }
}

fn serialise(note: ValueNote) -> [Field; VALUE_NOTE_LEN] {
    note.serialise()
}

fn compute_note_hash(note: ValueNote) -> Field {
    dep::std::hash::pedersen([
        generators::VALUE_NOTE,
        note.value, 
        note.owner.x, 
        note.owner.y, 
        note.randomness,
        note.nonce,
        note.is_real as Field,
    ])[0]
}

fn compute_nullifier(note: ValueNote, final_note_hash: Field) -> Field {
    if (!note.is_real) {
        0
    } else {
        let secret = get_secret_key(note.owner);
        dep::std::hash::pedersen([
            final_note_hash,
            secret, 
        ])[0]
    }
}

fn dummy() -> ValueNote {
    ValueNote::dummy()
}

fn is_dummy(note: ValueNote) -> bool {
    note.is_dummy()
}

global ValueNoteInterface = NoteInterface {
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    dummy,
    is_dummy,
};
