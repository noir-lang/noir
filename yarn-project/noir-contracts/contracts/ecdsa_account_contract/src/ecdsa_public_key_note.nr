use dep::aztec::protocol_types::address::AztecAddress;
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    oracle::{
        nullifier_key::get_nullifier_secret_key,
        get_public_key::get_public_key,
    },
    log::emit_encrypted_log,
    hash::pedersen_hash,
    context::PrivateContext,
};

global ECDSA_PUBLIC_KEY_NOTE_LEN: Field = 5;

// Stores an ECDSA public key composed of two 32-byte elements
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
struct EcdsaPublicKeyNote {
    x: [u8; 32],
    y: [u8; 32],
    owner: AztecAddress, // We store the owner address only to get the secret key to compute the nullifier
    header: NoteHeader,
}

impl EcdsaPublicKeyNote {
    pub fn new(x: [u8; 32], y: [u8; 32], owner: AztecAddress) -> Self {
        EcdsaPublicKeyNote {
            x,
            y,
            owner,
            header: NoteHeader::empty(),
        }
    }

    // serialize the note as 5 fields where:
    // [0] = x[0..31] (upper bound excluded)
    // [1] = x[31]
    // [2] = y[0..31]
    // [3] = y[31]
    // [4] = owner
    pub fn serialize(self) -> [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] {
        let mut x: Field = 0;
        let mut y: Field = 0;
        let mut mul: Field = 1;

        for i in 1..32 {
          let byte_x: Field = self.x[31 - i] as Field;
          x = x + (byte_x * mul);
          let byte_y: Field = self.y[31 - i] as Field;
          y = y + (byte_y * mul);
          mul *= 256;
        }

        let last_x = self.x[31] as Field;
        let last_y = self.y[31] as Field;
        
        [x, last_x, y, last_y, self.owner.to_field()]
    }

    pub fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let unique_siloed_note_hash = compute_note_hash_for_read_or_nullify(EcdsaPublicKeyNoteInterface, self);
        let secret = context.request_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            unique_siloed_note_hash,
            secret.low,
            secret.high,
        ],0)
    }

    pub fn compute_nullifier_without_context(self) -> Field {
        let unique_siloed_note_hash = compute_note_hash_for_read_or_nullify(EcdsaPublicKeyNoteInterface, self);
        let secret = get_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            unique_siloed_note_hash,
            secret.low,
            secret.high,
        ],0)
    }

    pub fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    // Broadcasts the note as an encrypted log on L1.
    pub fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            encryption_pub_key,
            self.serialize(),
        );
    }
}

fn deserialize(serialized_note: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN]) -> EcdsaPublicKeyNote {
    let mut x: [u8; 32] = [0; 32];
    let mut y: [u8; 32] = [0; 32];

    let part_x = serialized_note[0].to_be_bytes(32);
    for i in 0..31 {
        x[i] = part_x[i + 1];
    }
    x[31] = serialized_note[1].to_be_bytes(32)[31];

    let part_y = serialized_note[2].to_be_bytes(32);
    for i in 0..31 {
        y[i] = part_y[i + 1];
    }
    y[31] = serialized_note[3].to_be_bytes(32)[31];

    EcdsaPublicKeyNote { x, y, owner: AztecAddress::from_field(serialized_note[4]), header: NoteHeader::empty() }
}

fn serialize(note: EcdsaPublicKeyNote) -> [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: EcdsaPublicKeyNote) -> Field {
    // TODO(#1205) Should use a non-zero generator index.
    pedersen_hash(note.serialize(), 0)
}

fn compute_nullifier(note: EcdsaPublicKeyNote, context: &mut PrivateContext) -> Field {
    note.compute_nullifier(context)
}

fn compute_nullifier_without_context(note: EcdsaPublicKeyNote) -> Field {
    note.compute_nullifier_without_context()
}

fn get_header(note: EcdsaPublicKeyNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut EcdsaPublicKeyNote, header: NoteHeader) {
    note.set_header(header);
}

// Broadcasts the note as an encrypted log on L1.
fn broadcast(context: &mut PrivateContext, slot: Field, note: EcdsaPublicKeyNote) {
    note.broadcast(context, slot);
}

global EcdsaPublicKeyNoteInterface = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    compute_nullifier_without_context,
    get_header,
    set_header,
    broadcast,
};
