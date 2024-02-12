use dep::aztec::{
    note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_consumption},
    hash::pedersen_hash,
    oracle::{nullifier_key::get_nullifier_secret_key, get_public_key::get_public_key},
    log::emit_encrypted_log, context::PrivateContext, protocol_types::{address::AztecAddress}
};

global PUBLIC_KEY_NOTE_LEN: Field = 3;

// Stores a public key composed of two fields
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
struct PublicKeyNote {
    x: Field,
    y: Field,
    owner: AztecAddress, // We store the owner address only to get the secret key to compute the nullifier and to broadcast
    header: NoteHeader,
}

impl NoteInterface<PUBLIC_KEY_NOTE_LEN> for PublicKeyNote {
    fn serialize_content(self) -> [Field; PUBLIC_KEY_NOTE_LEN] {
        [self.x, self.y, self.owner.to_field()]
    }

    fn deserialize_content(serialized_note: [Field; PUBLIC_KEY_NOTE_LEN]) -> PublicKeyNote {
        PublicKeyNote {
            x: serialized_note[0],
            y: serialized_note[1],
            owner: AztecAddress::from_field(serialized_note[2]),
            header: NoteHeader::empty()
        }
    }

    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let unique_siloed_note_hash = compute_note_hash_for_consumption(self);
        let secret = context.request_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            unique_siloed_note_hash,
            secret.low,
            secret.high,
        ],0)
    }

    fn compute_nullifier_without_context(self) -> Field {
        let unique_siloed_note_hash = compute_note_hash_for_consumption(self);
        let secret = get_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            unique_siloed_note_hash,
            secret.low,
            secret.high,
        ],0)
    }

    fn compute_note_content_hash(note: PublicKeyNote) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash(note.serialize_content(), 0)
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    fn get_header(self) -> NoteHeader {
        self.header
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            Self::get_note_type_id(),
            encryption_pub_key,
            self.serialize_content(),
        );
    }

    fn get_note_type_id() -> Field {
        // TODO(#4519): autogenerate
        // python -c "print(int(''.join(str(ord(c)) for c in 'PublicKeyNote')))"
        8011798108105997510112178111116101
    }
}

impl PublicKeyNote {
    pub fn new(x: Field, y: Field, owner: AztecAddress) -> Self {
        PublicKeyNote { x, y, owner, header: NoteHeader::empty() }
    }
}
