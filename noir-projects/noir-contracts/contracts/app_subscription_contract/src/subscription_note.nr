use dep::aztec::{
    note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_consumption},
    hash::pedersen_hash,
    oracle::{nullifier_key::get_nullifier_secret_key, get_public_key::get_public_key},
    log::emit_encrypted_log, context::PrivateContext, protocol_types::{address::AztecAddress}
};

global SUBSCRIPTION_NOTE_LEN: Field = 3;

// Stores a public key composed of two fields
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
struct SubscriptionNote {
    owner: AztecAddress,
    expiry_block_number: Field,
    remaining_txs: Field,
    header: NoteHeader,
}

impl NoteInterface<SUBSCRIPTION_NOTE_LEN> for SubscriptionNote {
    fn serialize_content(self) -> [Field; SUBSCRIPTION_NOTE_LEN] {
        [self.owner.to_field(), self.expiry_block_number, self.remaining_txs]
    }

    fn deserialize_content(serialized_note: [Field; SUBSCRIPTION_NOTE_LEN]) -> SubscriptionNote {
        SubscriptionNote {
            owner: AztecAddress::from_field(serialized_note[0]),
            expiry_block_number: serialized_note[1],
            remaining_txs: serialized_note[2],
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

    fn compute_note_content_hash(note: SubscriptionNote) -> Field {
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
        1
    }
}

impl SubscriptionNote {
    pub fn new(owner: AztecAddress, expiry_block_number: Field, remaining_txs: Field) -> Self {
        SubscriptionNote { owner, expiry_block_number, remaining_txs, header: NoteHeader::empty() }
    }
}
