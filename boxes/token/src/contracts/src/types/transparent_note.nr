// docs:start:token_types_all
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    hash::{compute_secret_hash, pedersen_hash},
    context::PrivateContext,
};

global TRANSPARENT_NOTE_LEN: Field = 2;

// Transparent note represents a note that is created in the clear (public execution),
// but can only be spent by those that know the preimage of the "secret_hash"
struct TransparentNote {
    amount: Field,
    secret_hash: Field,
    // the secret is just here for ease of use and won't be (de)serialized
    secret: Field,
    // header is just here to satisfy the NoteInterface
    header: NoteHeader,
}

impl TransparentNote {

    // CONSTRUCTORS

    pub fn new(amount: Field, secret_hash: Field) -> Self {
        TransparentNote {
            amount: amount,
            secret_hash: secret_hash,
            secret: 0,
            header: NoteHeader::empty(),
        }
    }

    // new oracle call primitive
    // get me the secret corresponding to this hash
    pub fn new_from_secret(amount: Field, secret: Field) -> Self {
        TransparentNote {
            amount: amount,
            secret_hash: compute_secret_hash(secret),
            secret: secret,
            header: NoteHeader::empty(),
        }
    }


    // STANDARD NOTE_INTERFACE FUNCTIONS

    pub fn serialize(self) -> [Field; TRANSPARENT_NOTE_LEN] {
        [self.amount, self.secret_hash]
    }

    pub fn deserialize(serialized_note: [Field; TRANSPARENT_NOTE_LEN]) -> Self {
        TransparentNote {
            amount: serialized_note[0],
            secret_hash: serialized_note[1],
            secret: 0,
            header: NoteHeader::empty(),
        }
    }

    pub fn compute_note_hash(self) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            self.amount,
            self.secret_hash,
        ],0)
    }

    pub fn compute_nullifier(self, _context: &mut PrivateContext) -> Field {
        self.compute_nullifier_without_context()
    }

    pub fn compute_nullifier_without_context(self) -> Field {
        let siloed_note_hash = compute_note_hash_for_read_or_nullify(TransparentNoteMethods, self);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([self.secret, siloed_note_hash],0)
    }

    pub fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }


    // CUSTOM FUNCTIONS FOR THIS NOTE TYPE

    pub fn knows_secret(self, secret: Field) {
        let hash = compute_secret_hash(secret);
        assert(self.secret_hash == hash);
    }
}

fn deserialize(serialized_note: [Field; TRANSPARENT_NOTE_LEN]) -> TransparentNote {
    TransparentNote::deserialize(serialized_note)
}

fn serialize(note: TransparentNote) -> [Field; TRANSPARENT_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: TransparentNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: TransparentNote, context: &mut PrivateContext) -> Field {
    note.compute_nullifier(context)
}

fn compute_nullifier_without_context(note: TransparentNote) -> Field {
    note.compute_nullifier_without_context()
}

fn get_header(note: TransparentNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut TransparentNote, header: NoteHeader) {
    note.set_header(header)
}

fn broadcast(context: &mut PrivateContext, slot: Field, note: TransparentNote) {
    assert(false, "TransparentNote does not support broadcast");
}

global TransparentNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    compute_nullifier_without_context,
    get_header,
    set_header,
    broadcast,
};
// docs:end:token_types_all