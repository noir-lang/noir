use crate::context::Context;
use crate::note::note_getter_options::NoteGetterOptions;
use crate::note::note_hash::compute_inner_note_hash;
use crate::note::note_interface::NoteInterface;
use crate::oracle;
use crate::constants_gen::MAX_READ_REQUESTS_PER_CALL;
use crate::constants_gen::GET_NOTE_ORACLE_RETURN_LENGTH;
use crate::constants_gen::MAX_NOTES_PER_PAGE;
use crate::constants_gen::VIEW_NOTE_ORACLE_RETURN_LENGTH;

fn ensure_note_exists<Note, N>(
    mut context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    note: Note,
) -> Context {
    let mut note_hash = 0;
    let is_dummy = note_interface.is_dummy;
    if is_dummy(note) == false {
        let compute_note_hash = note_interface.compute_note_hash;
        note_hash = compute_inner_note_hash(storage_slot, compute_note_hash(note));
    };
    context.push_read_request(note_hash)
}

fn get_note<Note, N>(
    mut context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
) -> (Context, Note) {
    let note = oracle::notes::get_note(storage_slot, note_interface);
    context = ensure_note_exists(context, storage_slot, note_interface, note);
    (context, note)
}

fn get_notes<Note, N, S, P>(
    mut context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteGetterOptions<Note, N, S, P>,
) -> (Context, [Note; S]) {
    let notes = get_notes_internal(storage_slot, note_interface, options);
    let is_dummy = note_interface.is_dummy;
    for i in 0..notes.len() {
        let note = notes[i];
        let mut note_hash = 0;
        if is_dummy(note) == false {
            let compute_note_hash = note_interface.compute_note_hash;
            note_hash = compute_inner_note_hash(storage_slot, compute_note_hash(note));
        };
        context = context.push_read_request(note_hash);
    };

    (context, notes)
}

// TODO: Should be unconstrained.
fn get_notes_internal<Note, N, S, P>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteGetterOptions<Note, N, S, P>,
) -> [Note; S] {
    let dummy = note_interface.dummy;
    let sort_by = options.sort_by;
    let mut sort_by_indices = [0; N];
    let mut sort_order = [0; N];
    for i in 0..sort_by.len() {
        sort_by_indices[i] = sort_by[i].field_index;
        sort_order[i] = sort_by[i].order;
    };
    let offset = options.offset;
    let dummy_notes = [dummy(); MAX_READ_REQUESTS_PER_CALL];
    let zero_fields = [0; GET_NOTE_ORACLE_RETURN_LENGTH];
    let notes = oracle::notes::get_notes(
        storage_slot,
        note_interface,
        sort_by_indices,
        sort_order,
        MAX_READ_REQUESTS_PER_CALL as u32,
        offset,
        dummy_notes,
        zero_fields,
    );

    let filter = options.filter;
    let filter_args = options.filter_args;
    filter(notes, filter_args)
}

fn view_notes<Note, N>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    limit: u32,
    offset: u32,
) -> [Note; MAX_NOTES_PER_PAGE] {
    let dummy = note_interface.dummy;
    let sort_by = [0; N];
    let sort_order = [0; N];
    let dummy_notes = [dummy(); MAX_NOTES_PER_PAGE];
    let zero_fields = [0; VIEW_NOTE_ORACLE_RETURN_LENGTH];
    oracle::notes::get_notes(storage_slot, note_interface, sort_by, sort_order, limit, offset, dummy_notes, zero_fields)
}