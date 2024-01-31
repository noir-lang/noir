// docs:start:encrypted_import
use dep::aztec::log::emit_encrypted_log;
// docs:end:encrypted_import
use dep::aztec::{
    protocol_types::{
        address::AztecAddress,
        traits::{Serialize, Deserialize, Empty}
    },
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_consumption,
    },
    oracle::{
        rand::rand,
        nullifier_key::get_nullifier_secret_key,
        get_public_key::get_public_key,
    },
    hash::pedersen_hash,
    context::PrivateContext
};

global ADDRESS_NOTE_LEN: Field = 3;

// docs:start:address_note_def
// Stores an address
struct AddressNote {
    address: AztecAddress,
    owner: AztecAddress,
    randomness: Field,
    header: NoteHeader,
}

impl Serialize<ADDRESS_NOTE_LEN> for AddressNote {
    fn serialize(self) -> [Field; ADDRESS_NOTE_LEN]{
        [self.address.to_field(), self.owner.to_field(), self.randomness]
    }
}

impl Deserialize<ADDRESS_NOTE_LEN> for AddressNote {
    fn deserialize(serialized_note: [Field; ADDRESS_NOTE_LEN]) -> Self {
        AddressNote {
            address: AztecAddress::from_field(serialized_note[0]),
            owner: AztecAddress::from_field(serialized_note[1]),
            randomness: serialized_note[2],
            header: NoteHeader::empty(),
        }
    }
}

impl NoteInterface for AddressNote {
    fn compute_note_content_hash(self) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash(self.serialize(), 0)
    }

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

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    fn get_header(note: Self) -> NoteHeader {
        note.header
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        // docs:start:encrypted
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            encryption_pub_key,
            self.serialize(),
        );
        // docs:end:encrypted
    }
}

impl AddressNote {
    pub fn new(address: AztecAddress, owner: AztecAddress) -> Self {
        let randomness = rand();
        AddressNote {
            address,
            owner,
            randomness,
            header: NoteHeader::empty(),
        }
    }
// docs:end:address_note_def
}
