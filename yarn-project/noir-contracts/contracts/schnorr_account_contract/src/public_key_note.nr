use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    hash::pedersen_hash,
    oracle::{
        nullifier_key::get_nullifier_secret_key,
        get_public_key::get_public_key,
    },
    log::emit_encrypted_log,
    context::PrivateContext,
};
use dep::aztec::protocol_types::address::AztecAddress;

global PUBLIC_KEY_NOTE_LEN: Field = 3;

// Stores a public key composed of two fields
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
struct PublicKeyNote {
    x: Field,
    y: Field,
    owner: AztecAddress, // We store the owner address only to get the secret key to compute the nullifier and to broadcast
    header: NoteHeader,
}

impl PublicKeyNote {
    pub fn new(x: Field, y: Field, owner: AztecAddress) -> Self {
        PublicKeyNote {
            x,
            y,
            owner,
            header: NoteHeader::empty(),
        }
    }

    // serialize the note as 3 fields
    pub fn serialize(self) -> [Field; PUBLIC_KEY_NOTE_LEN] {
        [self.x, self.y, self.owner.to_field()]
    }

    pub fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let unique_siloed_note_hash = compute_note_hash_for_read_or_nullify(PublicKeyNoteMethods, self);
        let secret = context.request_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            unique_siloed_note_hash,
            secret.low,
            secret.high,
        ],0)
    }

    pub fn compute_nullifier_without_context(self) -> Field {
        let unique_siloed_note_hash = compute_note_hash_for_read_or_nullify(PublicKeyNoteMethods, self);
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

fn deserialize(serialized_note: [Field; PUBLIC_KEY_NOTE_LEN]) -> PublicKeyNote {
    PublicKeyNote {
        x: serialized_note[0],
        y: serialized_note[1],
        owner: AztecAddress::from_field(serialized_note[2]),
        header: NoteHeader::empty()
    }
}

fn serialize(note: PublicKeyNote) -> [Field; PUBLIC_KEY_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: PublicKeyNote) -> Field {
    // TODO(#1205) Should use a non-zero generator index.
    pedersen_hash(note.serialize(), 0)
}

fn compute_nullifier(note: PublicKeyNote, context: &mut PrivateContext) -> Field {
    note.compute_nullifier(context)
}

fn compute_nullifier_without_context(note: PublicKeyNote) -> Field {
    note.compute_nullifier_without_context()
}

fn get_header(note: PublicKeyNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut PublicKeyNote, header: NoteHeader) {
    note.set_header(header);
}

// Broadcasts the note as an encrypted log on L1.
fn broadcast(context: &mut PrivateContext, slot: Field, note: PublicKeyNote) {
    note.broadcast(context, slot);
}

global PublicKeyNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    compute_nullifier_without_context,
    get_header,
    set_header,
    broadcast,
};
