// docs:start:encrypted_import
use dep::aztec::log::emit_encrypted_log;
// docs:end:encrypted_import
use dep::aztec::{
    protocol_types::{address::AztecAddress, traits::Empty},
    note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_consumption},
    oracle::{rand::rand, nullifier_key::get_nullifier_secret_key, get_public_key::get_public_key},
    hash::pedersen_hash, context::PrivateContext
};

global ADDRESS_NOTE_LEN: Field = 3;

// docs:start:address_note_def
// Stores an address
#[aztec(note)]
struct AddressNote {
    address: AztecAddress,
    owner: AztecAddress,
    randomness: Field,
}

impl NoteInterface<ADDRESS_NOTE_LEN>  for AddressNote {

    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
    }

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        // docs:start:encrypted
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            Self::get_note_type_id(),
            encryption_pub_key,
            self.serialize_content(),
        );
        // docs:end:encrypted
    }
}

impl AddressNote {
    pub fn new(address: AztecAddress, owner: AztecAddress) -> Self {
        let randomness = rand();
        AddressNote { address, owner, randomness, header: NoteHeader::empty() }
    }
    // docs:end:address_note_def
}
