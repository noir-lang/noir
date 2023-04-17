use crate::notes::value_note::Note;
use crate::types::point::Point;
use crate::utils::arr_copy_slice;

// Data returned when getting a note
struct NoteGetterData {
    note: Note,
    leaf_index: Field,
    sibling_path: [Field; crate::PRIVATE_DATA_TREE_HEIGHT],
    root: Field,
}

fn note_getter_data_len() -> comptime Field {
    6 + 1 + crate::PRIVATE_DATA_TREE_HEIGHT + 1
}

fn make_note_getter_data(fields: [Field], start: comptime Field) -> NoteGetterData {
    NoteGetterData {
        note: Note {
            is_real: fields[start] as bool,
            nonce: fields[start + 1],
            owner: Point::new(fields[start + 2], fields[start + 3]),
            randomness: fields[start + 4],
            value: fields[start + 5],
        },
        leaf_index: fields[start + 6],
        sibling_path: arr_copy_slice(fields, [0; crate::PRIVATE_DATA_TREE_HEIGHT], 7),
        root: fields[start + 15],
    }
}