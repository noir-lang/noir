use crate::note::note_header::NoteHeader;

struct NoteInterface<Note, N> {
    deserialise: fn ([Field; N]) -> Note,

    serialise: fn (Note) -> [Field; N],

    compute_note_hash: fn (Note) -> Field,

    compute_nullifier: fn (Note) -> Field,

    dummy: fn () -> Note,

    is_dummy: fn (Note) -> bool,

    get_header: fn (Note) -> NoteHeader,

    set_header: fn (&mut Note, NoteHeader) -> (),
}
