use dep::protocol_types::address::AztecAddress;
use dep::protocol_types::traits::{Empty, Eq, Serialize};

struct NoteHeader {
    contract_address: AztecAddress,
    nonce: Field,
    storage_slot: Field,
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
    // Check the nonce to see whether a note is transient or not.
    note_hash_counter: u32, // a note_hash_counter of 0 means non-transient
}

impl Empty for NoteHeader {
    fn empty() -> Self {
        NoteHeader { contract_address: AztecAddress::zero(), nonce: 0, storage_slot: 0, note_hash_counter: 0 }
    }
}

impl Eq for NoteHeader {
    fn eq(self, other: Self) -> bool {
        (self.contract_address == other.contract_address) & 
        (self.nonce == other.nonce) & 
        (self.storage_slot == other.storage_slot)& 
        (self.note_hash_counter == other.note_hash_counter)
    }
}

impl NoteHeader {
    pub fn new(contract_address: AztecAddress, nonce: Field, storage_slot: Field) -> Self {
        NoteHeader { contract_address, nonce, storage_slot, note_hash_counter: 0 }
    }
}

impl Serialize<4> for NoteHeader {
    fn serialize(self) -> [Field; 4] {
        [self.contract_address.to_field(), self.nonce, self.storage_slot, self.note_hash_counter as Field]
    }
}
