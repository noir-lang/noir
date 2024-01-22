use crate::context::PrivateContext;
use crate::note::note_header::NoteHeader;

// docs:start:NoteInterface
struct NoteInterface<Note, N> {
    deserialize: fn ([Field; N]) -> Note,

    serialize: fn (Note) -> [Field; N],

    compute_note_hash: fn (Note) -> Field,

    compute_nullifier: fn (Note, &mut PrivateContext) -> Field,

    compute_nullifier_without_context: fn (Note) -> Field,

    get_header: fn (Note) -> NoteHeader,

    set_header: fn (&mut Note, NoteHeader) -> (),

    broadcast: fn (&mut PrivateContext, Field, Note) -> (),
}
// docs:end:NoteInterface