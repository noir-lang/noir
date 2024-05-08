use dep::aztec::context::PrivateContext;
use dep::aztec::note::{note_header::NoteHeader, note_interface::NoteInterface};

global TEST_NOTE_LENGTH = 1;

struct TestNote {
  header: NoteHeader,
  value: Field,
}

impl NoteInterface<TEST_NOTE_LENGTH> for TestNote {
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

    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
      assert(
        false, "TestNote does not support broadcast."
      );
    }
}

impl TestNote {
    pub fn new(value: Field) -> Self {
        TestNote { value, header: NoteHeader::empty() }
    }
}
