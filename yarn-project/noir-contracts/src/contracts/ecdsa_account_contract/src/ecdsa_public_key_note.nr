use dep::std::hash::pedersen;
use dep::aztec::note::note_interface::NoteInterface;
use dep::aztec::note::note_header::NoteHeader;
use dep::aztec::note::utils::compute_siloed_note_hash;
use dep::aztec::oracle::get_secret_key::get_secret_key;
use dep::aztec::oracle::get_public_key::get_public_key;

global ECDSA_PUBLIC_KEY_NOTE_LEN: Field = 65;

// Stores an ECDSA public key composed of two 32-byte elements
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
struct EcdsaPublicKeyNote {
    x: [u8; 32],
    y: [u8; 32],
    owner: Field, // We store the owner address only to get the secret key to compute the nullifier
    header: NoteHeader,
}

impl EcdsaPublicKeyNote {
    fn new(x: [u8; 32], y: [u8; 32], owner: Field) -> Self {
        EcdsaPublicKeyNote {
            x,
            y,
            owner,
            header: NoteHeader::empty(),
        }
    }

    fn serialise(self) -> [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] {
        let mut res: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] = [0; ECDSA_PUBLIC_KEY_NOTE_LEN];
        for i in 0..32 {
          res[i] = self.x[i] as Field;
          res[i + 32] = self.y[i] as Field;
        }
        res[64] = self.owner;
        res
    }

    fn compute_nullifier(self) -> Field {
        let siloed_note_hash = compute_siloed_note_hash(EcdsaPublicKeyNoteInterface, self);
        let owner_nullifying_public_key = get_public_key(self.owner);
        let secret = get_secret_key(owner_nullifying_public_key);
        dep::std::hash::pedersen([
            siloed_note_hash,
            secret,
        ])[0]
    }

    fn set_header(mut self: Self, header: NoteHeader) -> Self {
        self.header = header;
        self
    }

    fn dummy() -> Self {
        EcdsaPublicKeyNote {
            x: [0;32],
            y: [0;32],
            owner: 0,
            header: NoteHeader::empty(),
        }
    }

    fn is_dummy(self) -> bool {
        (self.x == [0;32]) & (self.y == [0;32]) & (self.owner == 0)
    }
}

fn deserialise(preimage: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN]) -> EcdsaPublicKeyNote {
    let mut x: [u8; 32] = [0;32];
    let mut y: [u8; 32] = [0;32];
    for i in 0..32 {
        x[i] = preimage[i] as u8;
        y[i] = preimage[i + 32] as u8;
    }

    EcdsaPublicKeyNote {
        x,
        y,
        owner: preimage[64],
        header: NoteHeader::empty(),
    }
}

fn serialise(note: EcdsaPublicKeyNote) -> [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] {
    note.serialise()
}

fn compute_note_hash(note: EcdsaPublicKeyNote) -> Field {
    dep::std::hash::pedersen(note.serialise())[0]
}

fn compute_nullifier(note: EcdsaPublicKeyNote) -> Field {
    note.compute_nullifier()
}

fn dummy() -> EcdsaPublicKeyNote {
    EcdsaPublicKeyNote::dummy()
}

fn is_dummy(note: EcdsaPublicKeyNote) -> bool {
    note.is_dummy()
}

fn get_header(note: EcdsaPublicKeyNote) -> NoteHeader {
    note.header
}

fn set_header(note: EcdsaPublicKeyNote, header: NoteHeader) -> EcdsaPublicKeyNote {
    note.set_header(header)
}

global EcdsaPublicKeyNoteInterface = NoteInterface {
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    dummy,
    is_dummy,
    get_header,
    set_header,
};
