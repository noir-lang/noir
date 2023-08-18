use crate::constants_gen::MAX_READ_REQUESTS_PER_CALL;
use crate::types::option::Option;

struct SortOrderEnum {
    DESC: u8,
    ASC: u8,
    NADA: u8,
}

global SortOrder = SortOrderEnum {
    DESC: 1,
    ASC: 2,
    NADA: 0,
};

struct Sort {
    field_index: u8,
    order: u8,
}

impl Sort {
    fn new(field_index: u8, order: u8) -> Self {
        Sort { field_index, order }
    }

    fn nada() -> Self {
        Sort { field_index: 0, order: SortOrder.NADA }
    }
}

struct NoteGetterOptions<Note, N, S, P> {
    sort_by: [Sort; N],
    offset: u32,
    filter: fn ([Option<Note>; MAX_READ_REQUESTS_PER_CALL], P) -> [Option<Note>; S],
    filter_args: P,
}

impl<Note, N, S, P> NoteGetterOptions<Note, N, S, P> {
    fn new(
        filter: fn ([Option<Note>; MAX_READ_REQUESTS_PER_CALL], P) -> [Option<Note>; S],
        filter_args: P,
        sort_by: [Sort; N],
        offset: u32,
    ) -> Self {
        NoteGetterOptions {
            sort_by,
            offset,
            filter,
            filter_args,
        }
    }

    fn with_filter(
        filter: fn ([Option<Note>; MAX_READ_REQUESTS_PER_CALL], P) -> [Option<Note>; S],
        filter_args: P,
    ) -> Self {
         NoteGetterOptions {
            sort_by: [Sort::nada(); N],
            offset: 0,
            filter,
            filter_args,
        }
    }

    fn with_filter_and_offset(
        filter: fn ([Option<Note>; MAX_READ_REQUESTS_PER_CALL], P) -> [Option<Note>; S],
        filter_args: P,
        offset: u32,
    ) -> Self {
         NoteGetterOptions {
            sort_by: [Sort::nada(); N],
            offset,
            filter,
            filter_args,
        }
    }
}
