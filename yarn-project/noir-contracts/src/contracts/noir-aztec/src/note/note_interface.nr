struct NoteInterface<Note, N> {
    deserialise: fn ([Field; N]) -> Note,

    serialise: fn (Note) -> [Field; N],

    compute_note_hash: fn (Note) -> Field,

    compute_nullifier: fn (Note, Field) -> Field,

    dummy: fn () -> Note,

    is_dummy: fn (Note) -> bool,
}
