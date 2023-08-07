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
    note_header::NoteHeader,
    utils::compute_note_hash_for_read_or_nullify,
    utils::compute_unique_siloed_note_hash,
    utils::compute_inner_note_hash,
    utils::compute_siloed_note_hash,
};
use crate::messaging::get_commitment_getter_data::make_commitment_getter_data;
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

    let note_hash_for_read_request = compute_note_hash_for_read_or_nullify(note_interface, *note);
    context.push_read_request(note_hash_for_read_request);
}

// Ensure a note's hash exists in the tree without retrieving the entire
// notes via the oracle.
// Modifies the note by populating it with header info.
fn ensure_note_hash_exists<Note, N>(
    context: &mut Context,
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
        contract_address: context.inputs.call_context.storage_contract_address,
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
    context: &mut Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
) -> Note {
    let note = get_note_internal(storage_slot, note_interface);

    check_note_header(*context, storage_slot, note_interface, note);

    let note_hash_for_read_request = compute_note_hash_for_read_or_nullify(note_interface, note);

    context.push_read_request(note_hash_for_read_request);
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
        let mut note_hash_for_read_request = 0;
        if opt_note.is_some() {
            let note = opt_note.unwrap_unchecked();
            check_note_header(*context, storage_slot, note_interface, note);
            note_hash_for_read_request = compute_note_hash_for_read_or_nullify(note_interface, note);
        };
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1410): test to ensure
        // failure if malicious oracle injects 0 nonce here for a "pre-existing" note.
        context.push_read_request(note_hash_for_read_request);
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