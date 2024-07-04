use crate::{
    abis::read_request::ScopedReadRequest, address::AztecAddress,
    abis::side_effect::{Ordered, OrderedValue, Readable, Scoped},
    constants::{NOTE_HASH_LENGTH, SCOPED_NOTE_HASH_LENGTH}, traits::{Empty, Serialize, Deserialize},
    utils::{arrays::array_concat, reader::Reader}
};

struct NoteHash {
    value: Field,
    counter: u32,
}

impl Ordered for NoteHash {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl Eq for NoteHash {
    fn eq(self, other: NoteHash) -> bool {
        (self.value == other.value)
            & (self.counter == other.counter) 
    }
}

impl Empty for NoteHash {
    fn empty() -> Self {
        NoteHash {
            value: 0,
            counter: 0,
        }
    }
}

impl Serialize<NOTE_HASH_LENGTH> for NoteHash {
    fn serialize(self) -> [Field; NOTE_HASH_LENGTH] {
        [self.value, self.counter as Field]
    }
}

impl Deserialize<NOTE_HASH_LENGTH> for NoteHash {
    fn deserialize(values: [Field; NOTE_HASH_LENGTH]) -> Self {
        Self {
            value: values[0],
            counter: values[1] as u32,
        }
    }
}

impl NoteHash {
    pub fn scope(self, contract_address: AztecAddress) -> ScopedNoteHash {
        ScopedNoteHash { note_hash: self, contract_address }
    }
}

struct ScopedNoteHash {
    note_hash: NoteHash,
    contract_address: AztecAddress,
}

impl Scoped<NoteHash> for ScopedNoteHash {
    fn inner(self) -> NoteHash {
        self.note_hash
    }
    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

impl Ordered for ScopedNoteHash {
    fn counter(self) -> u32 {
        self.note_hash.counter
    }
}

impl OrderedValue<Field> for ScopedNoteHash {
    fn value(self) -> Field {
        self.note_hash.value
    }
    fn counter(self) -> u32 {
        self.note_hash.counter
    }
}

impl Eq for ScopedNoteHash {
    fn eq(self, other: ScopedNoteHash) -> bool {
        (self.note_hash == other.note_hash)
            & (self.contract_address == other.contract_address)
    }
}

impl Empty for ScopedNoteHash {
    fn empty() -> Self {
        ScopedNoteHash {
            note_hash: NoteHash::empty(),
            contract_address: AztecAddress::zero(),
        }
    }
}

impl Serialize<SCOPED_NOTE_HASH_LENGTH> for ScopedNoteHash {
    fn serialize(self) -> [Field; SCOPED_NOTE_HASH_LENGTH] {
        array_concat(self.note_hash.serialize(), [self.contract_address.to_field()])
    }
}

impl Deserialize<SCOPED_NOTE_HASH_LENGTH> for ScopedNoteHash {
    fn deserialize(values: [Field; SCOPED_NOTE_HASH_LENGTH]) -> Self {
        let mut reader = Reader::new(values);
        let res = Self {
            note_hash: reader.read_struct(NoteHash::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        res
    }
}

impl Readable for ScopedNoteHash {
    fn assert_match_read_request(self, read_request: ScopedReadRequest) {
        assert_eq(self.note_hash.value, read_request.value(), "Value of the note hash does not match read request");
        assert_eq(self.contract_address, read_request.contract_address, "Contract address of the note hash does not match read request");
        assert(
            read_request.counter() > self.note_hash.counter, "Read request counter must be greater than the counter of the note hash"
        );
    }
}

impl ScopedNoteHash {
    pub fn expose_to_public(self) -> NoteHash {
        // Hide the actual counter when exposing it to the public kernel.
        NoteHash { value: self.note_hash.value, counter: 0 }
    }
}

#[test]
fn serialization_of_empty() {
    let item = NoteHash::empty();
    let serialized = item.serialize();
    let deserialized = NoteHash::deserialize(serialized);
    assert(item.eq(deserialized));
}

#[test]
fn serialization_of_empty_scoped() {
    let item = ScopedNoteHash::empty();
    let serialized = item.serialize();
    let deserialized = ScopedNoteHash::deserialize(serialized);
    assert(item.eq(deserialized));
}
