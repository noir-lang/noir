use crate::notes::value_note::Note;
use crate::types::point::Point;

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
        sibling_path: [
            fields[start + 7],
            fields[start + 8],
            fields[start + 9],
            fields[start + 10],
            fields[start + 11],
            fields[start + 12],
            fields[start + 13],
            fields[start + 14],
        ],
        root: fields[start + 15],
    }
}