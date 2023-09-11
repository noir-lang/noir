use dep::std::option::Option;
use crate::constants_gen::{
    MAX_READ_REQUESTS_PER_CALL,
    GET_NOTE_ORACLE_RETURN_LENGTH,
    GET_NOTES_ORACLE_RETURN_LENGTH,
    MAX_NOTES_PER_PAGE,
    VIEW_NOTE_ORACLE_RETURN_LENGTH,
};
use crate::context::PrivateContext;
use crate::note::{
    note_getter_options::{NoteGetterOptions, Select, Sort},
    note_interface::NoteInterface,
    note_header::NoteHeader,
    note_viewer_options::NoteViewerOptions,
    utils::compute_note_hash_for_read_or_nullify,
    utils::compute_unique_siloed_note_hash,
    utils::compute_inner_note_hash,
    utils::compute_siloed_note_hash,
};
use crate::messaging::get_commitment_getter_data::make_commitment_getter_data;
use crate::oracle;
use crate::types::vec::BoundedVec;

fn check_note_header<Note, N>(
    context: PrivateContext,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    note: Note,
) {
    let get_header = note_interface.get_header;
    let header = get_header(note);
    let contract_address = context.this_address();
    assert(header.contract_address == contract_address);
    assert(header.storage_slot == storage_slot);
}

fn ensure_note_exists<Note, N>(
    context: &mut PrivateContext,
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

    let note_hash_for_read_request = compute_note_hash_for_read_or_nullify(note_interface, *note);
    context.push_read_request(note_hash_for_read_request);
}

// Ensure a note's hash exists in the tree without retrieving the entire
// notes via the oracle.
// Modifies the note by populating it with header info.
fn ensure_note_hash_exists<Note, N>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    note: &mut Note,
) {
    // Initialize header of note. Must be done before computing note hashes as it initializes the:
    // - storage slot (used in inner note hash)
    // - the contract address (used in siloed note hash)
    // - and the nonce (used in the unique siloed note hash)
    let set_header = note_interface.set_header;
    let note_header = NoteHeader {
        contract_address: (*context).this_address(),
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): should be
        // real nonce (once public kernel applies nonces).
        nonce: 0,
        storage_slot
    };
    set_header(note, note_header);

    // Get a note from oracle and early out if it doesn't exist.
    let inner_note_hash = compute_inner_note_hash(note_interface, *note);

    let raw_oracle_ret = oracle::get_commitment::get_commitment(inner_note_hash);
    let deserialized_oracle_ret = make_commitment_getter_data(raw_oracle_ret, 0);
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): should be
    // unique_siloed_note_hash once public kernel applies nonces
    let saved_siloed_note_hash = deserialized_oracle_ret.message;

    assert(saved_siloed_note_hash != 0); // TODO(dbanks12): necessary?

    check_note_header(*context, storage_slot, note_interface, *note);

    // Ensure that the note hash retrieved from oracle matches the one computed from note.
    let computed_siloed_note_hash = compute_siloed_note_hash(note_interface, *note);
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): should be
    // compute_note_hash_for_read_or_nullify once public kernel applies nonces
    assert(computed_siloed_note_hash == saved_siloed_note_hash);

    context.push_read_request(computed_siloed_note_hash);
}

fn get_note<Note, N>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
) -> Note {
    let note = get_note_internal(storage_slot, note_interface);

    check_note_header(*context, storage_slot, note_interface, note);

    let note_hash_for_read_request = compute_note_hash_for_read_or_nullify(note_interface, note);

    context.push_read_request(note_hash_for_read_request);
    note
}

fn get_notes<Note, N, FILTER_ARGS>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteGetterOptions<Note, N, FILTER_ARGS>,
) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL] {
    let opt_notes = get_notes_internal(storage_slot, note_interface, options);
    for i in 0..opt_notes.len() {
        let opt_note = opt_notes[i];
        if opt_note.is_some() {
            let note = opt_note.unwrap_unchecked();
            check_note_header(*context, storage_slot, note_interface, note);

            let note_hash_for_read_request = compute_note_hash_for_read_or_nullify(note_interface, note);
            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1410): test to ensure
            // failure if malicious oracle injects 0 nonce here for a "pre-existing" note.
            context.push_read_request(note_hash_for_read_request);
        };
    };
    opt_notes
}

unconstrained fn get_note_internal<Note, N>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
) -> Note {
    let placeholder_note = [Option::none()];
    let placeholder_fields = [0; GET_NOTE_ORACLE_RETURN_LENGTH];
    oracle::notes::get_notes(
        storage_slot,
        note_interface,
        0,
        [],
        [],
        [],
        [],
        1, // limit
        0, // offset
        placeholder_note,
        placeholder_fields,
    )[0].unwrap() // Notice: we don't allow dummies to be returned from get_note (singular).
}

unconstrained fn get_notes_internal<Note, N, FILTER_ARGS>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteGetterOptions<Note, N, FILTER_ARGS>,
) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL] {
    let (num_selects, select_by, select_values, sort_by, sort_order) = flatten_options(options.selects, options.sorts);
    let placeholder_opt_notes = [Option::none(); MAX_READ_REQUESTS_PER_CALL];
    let placeholder_fields = [0; GET_NOTES_ORACLE_RETURN_LENGTH];
    let opt_notes = oracle::notes::get_notes(
        storage_slot,
        note_interface,
        num_selects,
        select_by,
        select_values,
        sort_by,
        sort_order,
        options.limit,
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
    options: NoteViewerOptions<Note, N>,
) -> [Option<Note>; MAX_NOTES_PER_PAGE] {
    let (num_selects, select_by, select_values, sort_by, sort_order) = flatten_options(options.selects, options.sorts);
    let placeholder_opt_notes = [Option::none(); MAX_NOTES_PER_PAGE];
    let placeholder_fields = [0; VIEW_NOTE_ORACLE_RETURN_LENGTH];
    oracle::notes::get_notes(
        storage_slot,
        note_interface,
        num_selects,
        select_by,
        select_values,
        sort_by,
        sort_order,
        options.limit,
        options.offset,
        placeholder_opt_notes,
        placeholder_fields,
    )
}

unconstrained fn flatten_options<Note, N>(
    selects: BoundedVec<Option<Select>, N>,
    sorts: BoundedVec<Option<Sort>, N>,
) -> (u8, [u8; N], [Field; N], [u8; N], [u2; N]) {
    let mut num_selects = 0;
    let mut select_by = [0; N];
    let mut select_values = [0; N];
    for i in 0..selects.len {
        let select = selects.get(i);
        if select.is_some() {
            select_by[num_selects] = select.unwrap_unchecked().field_index;
            select_values[num_selects] = select.unwrap_unchecked().value;
            num_selects += 1;
        };
    };

    let mut sort_by = [0; N];
    let mut sort_order = [0; N];
    for i in 0..sorts.len {
        let sort = sorts.get(i);
        if sort.is_some() {
            sort_by[i] = sort.unwrap_unchecked().field_index;
            sort_order[i] = sort.unwrap_unchecked().order;
        };
    };

    (num_selects, select_by, select_values, sort_by, sort_order)
}