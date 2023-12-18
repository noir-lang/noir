// docs:start:encrypted_import
use dep::aztec::log::emit_encrypted_log;
// docs:end:encrypted_import
use dep::protocol_types::address::AztecAddress;
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    oracle::{
        rand::rand,
        get_secret_key::get_secret_key,
        get_public_key::get_public_key,
    },
    hash::pedersen_hash,
    context::PrivateContext,
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


    pub fn serialize(self) -> [Field; ADDRESS_NOTE_LEN]{
        [self.address.to_field(), self.owner.to_field(), self.randomness]
    }

    pub fn deserialize(serialized_note: [Field; ADDRESS_NOTE_LEN]) -> Self {
        AddressNote {
            address: AztecAddress::from_field(serialized_note[0]),
            owner: AztecAddress::from_field(serialized_note[1]),
            randomness: serialized_note[2],
            header: NoteHeader::empty(),
        }
    }

    pub fn compute_note_hash(self) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash(self.serialize(), 0)
    }

    pub fn compute_nullifier(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(AddressNoteMethods, self);
        let secret = get_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            note_hash_for_nullify,
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

fn deserialize(serialized_note: [Field; ADDRESS_NOTE_LEN]) -> AddressNote {
    AddressNote::deserialize(serialized_note)
}

fn serialize(note: AddressNote) -> [Field; ADDRESS_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: AddressNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: AddressNote) -> Field {
    note.compute_nullifier()
}

fn get_header(note: AddressNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut AddressNote, header: NoteHeader) {
    note.set_header(header);
}

// Broadcasts the note as an encrypted log on L1.
fn broadcast(context: &mut PrivateContext, slot: Field, note: AddressNote) {
    note.broadcast(context, slot);
}

global AddressNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
    broadcast,
};
