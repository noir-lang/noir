use dep::aztec::context::PrivateContext;
use dep::aztec::{
    note::{note_header::NoteHeader, note_interface::NoteInterface},
    protocol_types::grumpkin_point::GrumpkinPoint
};

global TEST_NOTE_LENGTH = 1;
// TEST_NOTE_LENGTH * 32 + 32(storage_slot as bytes) + 32(note_type_id as bytes)
global TEST_NOTE_BYTES_LENGTH: Field = 1 * 32 + 64;

struct TestNote {
  header: NoteHeader,
  value: Field,
}

impl NoteInterface<TEST_NOTE_LENGTH, TEST_NOTE_BYTES_LENGTH> for TestNote {
    fn serialize_content(self) -> [Field; TEST_NOTE_LENGTH] {
      [self.value]
    }

    fn deserialize_content(fields: [Field; TEST_NOTE_LENGTH]) -> Self {
      Self {
        value: fields[0],
        header: NoteHeader::empty(),
      }
    }

    fn compute_note_content_hash(self) -> Field {
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

    fn compute_nullifier(self, _context: &mut PrivateContext) -> Field {
      0
    }

    fn compute_nullifier_without_context(self) -> Field {
      0
    }

    fn broadcast(self, context: &mut PrivateContext, slot: Field, ivpk_m: GrumpkinPoint) {
      assert(
        false, "TestNote does not support broadcast."
      );
    }

    fn to_be_bytes(self, storage_slot: Field) -> [u8; TEST_NOTE_BYTES_LENGTH] {
      let serialized_note = self.serialize_content();

        let mut buffer: [u8; TEST_NOTE_BYTES_LENGTH] = [0; TEST_NOTE_BYTES_LENGTH];

        let storage_slot_bytes = storage_slot.to_be_bytes(32);
        let note_type_id_bytes = TestNote::get_note_type_id().to_be_bytes(32);

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

impl TestNote {
    pub fn new(value: Field) -> Self {
        TestNote { value, header: NoteHeader::empty() }
    }
}
