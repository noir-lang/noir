use dep::std::hash::pedersen;
use dep::aztec::note::note_interface::NoteInterface;
use dep::aztec::note::note_header::NoteHeader;
use dep::aztec::note::utils::compute_unique_siloed_note_hash;
use dep::aztec::oracle::get_secret_key::get_secret_key;

global ECDSA_PUBLIC_KEY_NOTE_LEN: Field = 5;

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

    // Serialise the note as 5 fields where:
    // [0] = x[0..31] (upper bound excluded)
    // [1] = x[31]
    // [2] = y[0..31]
    // [3] = y[31]
    // [4] = owner
    fn serialise(self) -> [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] {
        let mut x: Field = 0;
        let mut y: Field = 0;
        let mut mul: Field = 1;

        for i in 1..32 {
          let bytex: Field = self.x[31 - i] as Field;
          x = x + (bytex * mul);
          let bytey: Field = self.y[31 - i] as Field;
          y = y + (bytey * mul);
          mul *= 256;
        }

        let last_x = self.x[31] as Field;
        let last_y = self.y[31] as Field;
        
        let res: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] = [x, last_x, y, last_y, self.owner];
        res
    }

    fn compute_nullifier(self) -> Field {
        let unique_siloed_note_hash = compute_unique_siloed_note_hash(EcdsaPublicKeyNoteInterface, self);
        let secret = get_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        dep::std::hash::pedersen([
            unique_siloed_note_hash,
            secret.high,
            secret.low,
        ])[0]
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }
}

fn deserialise(preimage: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN]) -> EcdsaPublicKeyNote {
    let mut x: [u8; 32] = [0;32];
    let mut y: [u8; 32] = [0;32];

    let part_x = preimage[0].to_be_bytes(32);
    for i in 0..31 { x[i] = part_x[i + 1]; }
    x[31] = preimage[1].to_be_bytes(32)[31];

    let part_y = preimage[2].to_be_bytes(32);
    for i in 0..31 { y[i] = part_y[i + 1]; }
    y[31] = preimage[3].to_be_bytes(32)[31];

    EcdsaPublicKeyNote {
        x,
        y,
        owner: preimage[4],
        header: NoteHeader::empty(),
    }
}

fn serialise(note: EcdsaPublicKeyNote) -> [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] {
    note.serialise()
}

fn compute_note_hash(note: EcdsaPublicKeyNote) -> Field {
    // TODO(#1205) Should use a non-zero generator index.
    dep::std::hash::pedersen(note.serialise())[0]
}

fn compute_nullifier(note: EcdsaPublicKeyNote) -> Field {
    note.compute_nullifier()
}

fn get_header(note: EcdsaPublicKeyNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut EcdsaPublicKeyNote, header: NoteHeader) {
    note.set_header(header);
}

global EcdsaPublicKeyNoteInterface = NoteInterface {
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
};
