use dep::std::option::Option;
use dep::protocol_types::constants::{
    MAX_READ_REQUESTS_PER_CALL,
    GET_NOTE_ORACLE_RETURN_LENGTH,
    GET_NOTES_ORACLE_RETURN_LENGTH,
    MAX_NOTES_PER_PAGE,
    VIEW_NOTE_ORACLE_RETURN_LENGTH,
};
use crate::context::PrivateContext;
use crate::note::{
    note_getter_options::{NoteGetterOptions, Select, Sort, SortOrder, Comparator},
    note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions,
    utils::compute_note_hash_for_read_or_nullify,
};
use crate::oracle;
use crate::types::vec::BoundedVec;

fn check_note_header<Note, N>(
    context: PrivateContext,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    note: Note
) {
    let get_header = note_interface.get_header;
    let header = get_header(note);
    let contract_address = context.this_address();
    assert(header.contract_address.eq(contract_address));
    assert(header.storage_slot == storage_slot);
}

fn check_note_fields<N>(fields: [Field; N], selects: BoundedVec<Option<Select>, N>) {
    for i in 0..selects.len {
        let select = selects.get_unchecked(i).unwrap_unchecked();

        // Values are computed ahead of time because circuits evaluate all branches
        let isEqual = fields[select.field_index] == select.value;
        let isLt = fields[select.field_index].lt(select.value);

        if (select.comparator == Comparator.EQ) {
            assert(isEqual, "Mismatch return note field.");
        } else if (select.comparator == Comparator.NEQ) {
            assert(!isEqual, "Mismatch return note field.");
        } else if (select.comparator == Comparator.LT) {
            assert(isLt, "Mismatch return note field.");
        } else if (select.comparator == Comparator.LTE) {
            assert(isLt | isEqual, "Mismatch return note field.");
        } else if (select.comparator == Comparator.GT) {
            assert(!isLt & !isEqual, "Mismatch return note field.");
        } else if (select.comparator == Comparator.GTE) {
            assert(!isLt, "Mismatch return note field.");
        }
    }
}

fn check_notes_order<N>(
    fields_0: [Field; N],
    fields_1: [Field; N],
    sorts: BoundedVec<Option<Sort>, N>
) {
    for i in 0..sorts.len {
        let sort = sorts.get_unchecked(i).unwrap_unchecked();
        let eq = fields_0[sort.field_index] == fields_1[sort.field_index];
        let lt = fields_0[sort.field_index] as u120 < fields_1[sort.field_index] as u120;
        if sort.order == SortOrder.ASC {
            assert(eq | lt, "Return notes not sorted in ascending order.");
        } else if !eq {
            assert(!lt, "Return notes not sorted in descending order.");
        }
    }
}

pub fn get_note<Note, N>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>
) -> Note {
    let note = get_note_internal(storage_slot, note_interface);

    check_note_header(*context, storage_slot, note_interface, note);

    let note_hash_for_read_request = compute_note_hash_for_read_or_nullify(note_interface, note);

    context.push_read_request(note_hash_for_read_request);
    note
}

pub fn get_notes<Note, N, FILTER_ARGS>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteGetterOptions<Note, N, FILTER_ARGS>
) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL] {
    let opt_notes = get_notes_internal(storage_slot, note_interface, options);
    let mut num_notes = 0;
    let mut prev_fields = [0; N];
    for i in 0..opt_notes.len() {
        let opt_note = opt_notes[i];
        if opt_note.is_some() {
            let note = opt_note.unwrap_unchecked();
            let serialize = note_interface.serialize;
            let fields = serialize(note);
            check_note_header(*context, storage_slot, note_interface, note);
            check_note_fields(fields, options.selects);
            if i != 0 {
                check_notes_order(prev_fields, fields, options.sorts);
            }
            prev_fields = fields;

            let note_hash_for_read_request = compute_note_hash_for_read_or_nullify(note_interface, note);
            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1410): test to ensure
            // failure if malicious oracle injects 0 nonce here for a "pre-existing" note.
            context.push_read_request(note_hash_for_read_request);

            num_notes += 1;
        };
    }
    if options.limit != 0 {
        assert(num_notes <= options.limit, "Invalid number of return notes.");
    }
    opt_notes
}

unconstrained fn get_note_internal<Note, N>(storage_slot: Field, note_interface: NoteInterface<Note, N>) -> Note {
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
        [],
        1, // limit
        0, // offset
        placeholder_note,
        placeholder_fields
    )[0].unwrap() // Notice: we don't allow dummies to be returned from get_note (singular).
}

unconstrained fn get_notes_internal<Note, N, FILTER_ARGS>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteGetterOptions<Note, N, FILTER_ARGS>
) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL] {
    let (num_selects, select_by, select_values, select_comparators, sort_by, sort_order) = flatten_options(options.selects, options.sorts);
    let placeholder_opt_notes = [Option::none(); MAX_READ_REQUESTS_PER_CALL];
    let placeholder_fields = [0; GET_NOTES_ORACLE_RETURN_LENGTH];
    let opt_notes = oracle::notes::get_notes(
        storage_slot,
        note_interface,
        num_selects,
        select_by,
        select_values,
        select_comparators,
        sort_by,
        sort_order,
        options.limit,
        options.offset,
        placeholder_opt_notes,
        placeholder_fields
    );

    let filter = options.filter;
    let filter_args = options.filter_args;
    filter(opt_notes, filter_args)
}

unconstrained pub fn view_notes<Note, N>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    options: NoteViewerOptions<Note, N>
) -> [Option<Note>; MAX_NOTES_PER_PAGE] {
    let (num_selects, select_by, select_values, select_comparators, sort_by, sort_order) = flatten_options(options.selects, options.sorts);
    let placeholder_opt_notes = [Option::none(); MAX_NOTES_PER_PAGE];
    let placeholder_fields = [0; VIEW_NOTE_ORACLE_RETURN_LENGTH];
    oracle::notes::get_notes(
        storage_slot,
        note_interface,
        num_selects,
        select_by,
        select_values,
        select_comparators,
        sort_by,
        sort_order,
        options.limit,
        options.offset,
        placeholder_opt_notes,
        placeholder_fields
    )
}

unconstrained fn flatten_options<Note, N>(
    selects: BoundedVec<Option<Select>, N>,
    sorts: BoundedVec<Option<Sort>, N>
) -> (u8, [u8; N], [Field; N], [u3; N], [u8; N], [u2; N]) {
    let mut num_selects = 0;
    let mut select_by = [0; N];
    let mut select_values = [0; N];
    let mut select_comparators = [0; N];

    for i in 0..selects.len {
        let select = selects.get(i);
        if select.is_some() {
            select_by[num_selects] = select.unwrap_unchecked().field_index;
            select_values[num_selects] = select.unwrap_unchecked().value;
            select_comparators[num_selects] = select.unwrap_unchecked().comparator;
            num_selects += 1;
        };
    }

    let mut sort_by = [0; N];
    let mut sort_order = [0; N];
    for i in 0..sorts.len {
        let sort = sorts.get(i);
        if sort.is_some() {
            sort_by[i] = sort.unwrap_unchecked().field_index;
            sort_order[i] = sort.unwrap_unchecked().order;
        };
    }

    (num_selects, select_by, select_values, select_comparators, sort_by, sort_order)
}
