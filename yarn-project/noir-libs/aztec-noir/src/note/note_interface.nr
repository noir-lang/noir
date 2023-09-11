use crate::note::note_header::NoteHeader;

// docs:start:NoteInterface
struct NoteInterface<Note, N> {
    deserialise: fn ([Field; N]) -> Note,

    serialise: fn (Note) -> [Field; N],

    compute_note_hash: fn (Note) -> Field,

    compute_nullifier: fn (Note) -> Field,

    get_header: fn (Note) -> NoteHeader,

    set_header: fn (&mut Note, NoteHeader) -> (),
}
// docs:end:NoteInterface