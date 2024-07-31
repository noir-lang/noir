use dep::protocol_types::{address::AztecAddress, traits::{Empty, Eq, Serialize}};

global NOTE_HEADER_LENGTH: Field = 4;

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

impl Serialize<NOTE_HEADER_LENGTH> for NoteHeader {
    /// The following method is used by implementations of the Serialize trait for notes --> the implementation
    // of the Serialize trait for notes needs to be implemented when the note is passed as an argument to a contract
    // function --> in that situation the serialize method is called by aztec-nr when computing an arguments hash.
    fn serialize(self) -> [Field; NOTE_HEADER_LENGTH] {
        // Note: If you change this function don't forget to update implementations of Serialize trait for notes.
        // (Serialize trait needs to be implemented for a note when it's passed as an argument to a contract function
        // --> then it's used when computing args hash.)
        [self.contract_address.to_field(), self.nonce, self.storage_slot, self.note_hash_counter as Field]
    }
}
