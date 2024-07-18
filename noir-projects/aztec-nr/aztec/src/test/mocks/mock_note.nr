use crate::{context::PrivateContext, note::{note_header::NoteHeader, note_interface::NoteInterface}};

use dep::protocol_types::{address::AztecAddress, traits::Eq};

global MOCK_NOTE_LENGTH = 1;
// MOCK_NOTE_LENGTH * 32 + 32(storage_slot as bytes) + 32(note_type_id as bytes)
global MOCK_NOTE_BYTES_LENGTH: Field = 1 * 32 + 64;

struct MockNote {
  header: NoteHeader,
  value: Field,
}

impl NoteInterface<MOCK_NOTE_LENGTH, MOCK_NOTE_BYTES_LENGTH> for MockNote {
    fn serialize_content(self) -> [Field; MOCK_NOTE_LENGTH] {
      [self.value]
    }

    fn deserialize_content(fields: [Field; MOCK_NOTE_LENGTH]) -> Self {
      Self {
        value: fields[0],
        header: NoteHeader::empty(),
      }
    }

    fn compute_note_content_hash(_self: Self) -> Field {
      0
    }

    fn get_header(self) -> NoteHeader {
      self.header
    }

    fn set_header(&mut self, header: NoteHeader) -> () {
      self.header = header;
    }

    fn get_note_type_id() -> Field {
      0
    }

    fn compute_note_hash_and_nullifier(_self: Self, _context: &mut PrivateContext) -> (Field, Field) {
      (0, 0)
    }

    fn compute_note_hash_and_nullifier_without_context(_self: Self) -> (Field, Field) {
      (0, 0)
    }

    fn to_be_bytes(self, storage_slot: Field) -> [u8; MOCK_NOTE_BYTES_LENGTH] {
      let serialized_note = self.serialize_content();

        let mut buffer: [u8; MOCK_NOTE_BYTES_LENGTH] = [0; MOCK_NOTE_BYTES_LENGTH];

        let storage_slot_bytes = storage_slot.to_be_bytes(32);
        let note_type_id_bytes = MockNote::get_note_type_id().to_be_bytes(32);

        for i in 0..32 {
            buffer[i] = storage_slot_bytes[i];
            buffer[32 + i] = note_type_id_bytes[i];
        }

        for i in 0..serialized_note.len() {
            let bytes = serialized_note[i].to_be_bytes(32);
            for j in 0..32 {
                buffer[64 + i * 32 + j] = bytes[j];
            }
        }
        buffer
    }
}

impl Eq for MockNote {
    fn eq(self, other: Self) -> bool {
      (self.header == other.header) & 
      (self.value == other.value)
    }
}

struct MockNoteBuilder {
    value: Field,
    contract_address: Option<AztecAddress>,
    storage_slot: Option<Field>,
}

impl MockNoteBuilder {
    fn new(value: Field) -> Self {
        MockNoteBuilder { value, contract_address: Option::none(), storage_slot: Option::none() }
    }

    fn contract_address(&mut self, contract_address: AztecAddress) -> &mut Self {
        self.contract_address = Option::some(contract_address);
        self
    }

    fn storage_slot(&mut self, storage_slot: Field) -> &mut Self {
        self.storage_slot = Option::some(storage_slot);
        self
    }

    fn build(self) -> MockNote {
        let mut header = NoteHeader::empty();

        if self.contract_address.is_some() {
            header.contract_address = self.contract_address.unwrap();
        }

        if self.storage_slot.is_some() {
            header.storage_slot = self.storage_slot.unwrap();
        }

        MockNote { value: self.value, header }
    }
}

impl MockNote {
    pub fn new(value: Field) -> MockNoteBuilder {
        MockNoteBuilder::new(value)
    }
}
