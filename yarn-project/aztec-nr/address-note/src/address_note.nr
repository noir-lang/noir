// docs:start:encrypted_import
use dep::aztec::log::emit_encrypted_log;
// docs:end:encrypted_import
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_siloed_note_hash,
    },
    oracle::{
        get_secret_key::get_secret_key,
        get_public_key::get_public_key,
    },
    hash::pedersen_hash,
    context::PrivateContext,
};

global ADDRESS_NOTE_LEN: Field = 2;

// Stores an address
struct AddressNote {
    address: Field,
    owner: Field,
    header: NoteHeader,
}

impl AddressNote {
    pub fn new(address: Field, owner: Field) -> Self {
        AddressNote {
            address,
            owner,
            header: NoteHeader::empty(),
        }
    }

    pub fn serialize(self) -> [Field; ADDRESS_NOTE_LEN]{
        [self.address, self.owner]
    }

    pub fn compute_nullifier(self) -> Field {
        let siloed_note_hash = compute_siloed_note_hash(AddressNoteMethods, self);
        let secret = get_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            siloed_note_hash,
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

fn deserialize(preimage: [Field; ADDRESS_NOTE_LEN]) -> AddressNote {
    AddressNote {
        address: preimage[0],
        owner: preimage[1],
        header: NoteHeader::empty(),
    }
}

fn serialize(note: AddressNote) -> [Field; ADDRESS_NOTE_LEN]{
    note.serialize()
}

fn compute_note_hash(note: AddressNote) -> Field {
    // TODO(#1205) Should use a non-zero generator index.
    pedersen_hash(note.serialize(),0)
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
