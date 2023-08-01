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
use crate::types::option::Option;

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
    context: &mut Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    note: &mut Note,
) {
    let saved_note = get_note_internal(storage_slot, note_interface);

    // Only copy over the header to the original note to make sure the preimage is the same.
    let get_header = note_interface.get_header;
    let set_header = note_interface.set_header;
    let note_header = get_header(saved_note);
    set_header(note, note_header);

    check_note_header(*context, storage_slot, note_interface, *note);

    let unique_note_hash = compute_unique_note_hash(note_interface, *note);

    context.push_read_request(unique_note_hash);
}

fn get_note<Note, N>(
    context: &mut Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
) -> Note {
    let note = get_note_internal(storage_slot, note_interface);

    check_note_header(*context, storage_slot, note_interface, note);
    
    let unique_note_hash = compute_unique_note_hash(note_interface, note);

    context.push_read_request(unique_note_hash);
    note
}

fn get_notes<Note, N, S, P>(
    context: &mut Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteGetterOptions<Note, N, S, P>,
) -> [Option<Note>; S] {
    let opt_notes = get_notes_internal(storage_slot, note_interface, options);
    for i in 0..opt_notes.len() {
        let opt_note = opt_notes[i];
        let mut unique_note_hash = 0;
        if opt_note.is_some() {
            let note = opt_note.unwrap_unchecked();
            check_note_header(*context, storage_slot, note_interface, note);
            unique_note_hash = compute_unique_note_hash(note_interface, note);
        };
        context.push_read_request(unique_note_hash);
    };
    opt_notes
}

unconstrained fn get_note_internal<Note, N>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
) -> Note {
    let dummy = note_interface.dummy;
    let placeholder_note = [Option::none(dummy())];
    let placeholder_fields = [0; GET_NOTE_ORACLE_RETURN_LENGTH];
    oracle::notes::get_notes(
        storage_slot,
        note_interface,
        [],
        [],
        1, // limit
        0, // offset
        placeholder_note,
        placeholder_fields,
    )[0].unwrap() // Notice: we don't allow dummies to be returned from get_note (singular).
}

unconstrained fn get_notes_internal<Note, N, S, P>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteGetterOptions<Note, N, S, P>,
) -> [Option<Note>; S] {
    let dummy = note_interface.dummy;
    let sort_by = options.sort_by;
    let mut sort_by_indices = [0; N];
    let mut sort_order = [0; N];
    for i in 0..sort_by.len() {
        sort_by_indices[i] = sort_by[i].field_index;
        sort_order[i] = sort_by[i].order;
    };
    let placeholder_opt_notes = [Option::none(dummy()); MAX_READ_REQUESTS_PER_CALL];
    let placeholder_fields = [0; GET_NOTES_ORACLE_RETURN_LENGTH];
    let opt_notes = oracle::notes::get_notes(
        storage_slot,
        note_interface,
        sort_by_indices,
        sort_order,
        MAX_READ_REQUESTS_PER_CALL as u32,
        options.offset,
        placeholder_opt_notes,
        placeholder_fields,
    );

    let filter = options.filter;
    let filter_args = options.filter_args;
    filter(opt_notes, filter_args)
}

unconstrained fn view_notes<Note, N>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    limit: u32,
    offset: u32,
) -> [Option<Note>; MAX_NOTES_PER_PAGE] {
    let dummy = note_interface.dummy;
    let sort_by = [0; N];
    let sort_order = [0; N];
    let dummy_notes = [Option::none(dummy()); MAX_NOTES_PER_PAGE];
    let zero_fields = [0; VIEW_NOTE_ORACLE_RETURN_LENGTH];
    oracle::notes::get_notes(storage_slot, note_interface, sort_by, sort_order, limit, offset, dummy_notes, zero_fields)
}