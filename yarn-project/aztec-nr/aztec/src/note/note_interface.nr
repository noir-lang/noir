use crate::context::PrivateContext;
use crate::note::note_header::NoteHeader;

// docs:start:note_interface
trait NoteInterface<N> {
    fn serialize_content(self) -> [Field; N];

    fn deserialize_content(fields: [Field; N]) -> Self;

    fn compute_note_content_hash(self) -> Field;

    fn get_header(self) -> NoteHeader;

    fn set_header(&mut self, header: NoteHeader) -> ();

    fn compute_nullifier(self, context: &mut PrivateContext) -> Field;

    fn compute_nullifier_without_context(self) -> Field;

    fn broadcast(self, context: &mut PrivateContext, slot: Field) -> ();
}
// docs:end:note_interface

