// docs:start:token_types_all
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_consumption,
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

impl NoteInterface<TRANSPARENT_NOTE_LEN> for TransparentNote {
    fn serialize_content(self) -> [Field; TRANSPARENT_NOTE_LEN] {
        [self.amount, self.secret_hash]
    }

    fn deserialize_content(serialized_note: [Field; TRANSPARENT_NOTE_LEN]) -> Self {
        TransparentNote {
            amount: serialized_note[0],
            secret_hash: serialized_note[1],
            secret: 0,
            header: NoteHeader::empty(),
        }
    }

    fn compute_note_content_hash(self) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash(self.serialize_content(), 0)
    }

    fn compute_nullifier(self, _context: &mut PrivateContext) -> Field {
        self.compute_nullifier_without_context()
    }

    fn compute_nullifier_without_context(self) -> Field {
        let siloed_note_hash = compute_note_hash_for_consumption(self);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([self.secret, siloed_note_hash],0)
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    fn get_header(self) -> NoteHeader {
        self.header
    }

    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        assert(false, "TransparentNote does not support broadcast");
    }
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

    // CUSTOM FUNCTIONS FOR THIS NOTE TYPE

    pub fn knows_secret(self, secret: Field) {
        let hash = compute_secret_hash(secret);
        assert(self.secret_hash == hash);
    }
}
// docs:end:token_types_all