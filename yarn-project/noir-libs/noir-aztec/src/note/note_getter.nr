use crate::constants_gen::{
    MAX_READ_REQUESTS_PER_CALL,
    GET_NOTE_ORACLE_RETURN_LENGTH,
    GET_NOTES_ORACLE_RETURN_LENGTH,
    MAX_NOTES_PER_PAGE,
    VIEW_NOTE_ORACLE_RETURN_LENGTH,
};
use crate::context::Context;
use crate::note::{
    note_getter_options::NoteGetterOptions,
    note_interface::NoteInterface,
    utils::compute_unique_note_hash,
};
use crate::oracle;

fn check_note_header<Note, N>(
    context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    note: Note,
) {
    let get_header = note_interface.get_header;
    let header = get_header(note);
    let contract_address = context.inputs.call_context.storage_contract_address;
    assert(header.contract_address == contract_address);
    assert(header.storage_slot == storage_slot);
}

fn ensure_note_exists<Note, N>(
    mut context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    note: Note,
) -> (Context, Note) {
    let mut note_with_header = note;
    let mut unique_note_hash = 0;
    let is_dummy = note_interface.is_dummy;
    if is_dummy(note) == false {
        // Get a note from oracle and early out if it doesn't exist.
        let saved_note = get_note_internal(storage_slot, note_interface);
        assert(is_dummy(saved_note) == false);

        // Only copy over the header to the original note to make sure the preimage is the same.
        let get_header = note_interface.get_header;
        let set_header = note_interface.set_header;
        let header = get_header(saved_note);
        note_with_header = set_header(note, header);

        check_note_header(context, storage_slot, note_interface, note_with_header);

        unique_note_hash = compute_unique_note_hash(note_interface, note_with_header);
    };
    context = context.push_read_request(unique_note_hash);
    (context, note_with_header)
}

fn get_note<Note, N>(
    mut context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
) -> (Context, Note) {
    let note = get_note_internal(storage_slot, note_interface);
    let mut unique_note_hash = 0;
    let is_dummy = note_interface.is_dummy;
    if is_dummy(note) == false {
        check_note_header(context, storage_slot, note_interface, note);
        unique_note_hash = compute_unique_note_hash(note_interface, note);
    };
    context = context.push_read_request(unique_note_hash);
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
        let mut unique_note_hash = 0;
        if is_dummy(note) == false {
            check_note_header(context, storage_slot, note_interface, note);
            unique_note_hash = compute_unique_note_hash(note_interface, note);
        };
        context = context.push_read_request(unique_note_hash);
    };
    (context, notes)
}

// TODO: Should be unconstrained.
fn get_note_internal<Note, N>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
) -> Note {
    let dummy = note_interface.dummy;
    let dummy_note = [dummy()];
    let zero_fields = [0; GET_NOTE_ORACLE_RETURN_LENGTH];
    oracle::notes::get_notes(
        storage_slot,
        note_interface,
        [],
        [],
        1, // limit
        0, // offset
        dummy_note,
        zero_fields,
    )[0]
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
    let dummy_notes = [dummy(); MAX_READ_REQUESTS_PER_CALL];
    let zero_fields = [0; GET_NOTES_ORACLE_RETURN_LENGTH];
    let notes = oracle::notes::get_notes(
        storage_slot,
        note_interface,
        sort_by_indices,
        sort_order,
        MAX_READ_REQUESTS_PER_CALL as u32,
        options.offset,
        dummy_notes,
        zero_fields,
    );

    let filter = options.filter;
    let filter_args = options.filter_args;
    filter(notes, filter_args)
}

// TODO: Should be unconstrained.
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