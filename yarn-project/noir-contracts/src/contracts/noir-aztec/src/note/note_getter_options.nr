use crate::abi::MAX_READ_REQUESTS;

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
    skip: u32,
    filter: fn ([Note; MAX_READ_REQUESTS], P) -> [Note; S],
    filter_args: P,
}

impl<Note, N, S, P> NoteGetterOptions<Note, N, S, P> {
    fn new(
        filter: fn ([Note; MAX_READ_REQUESTS], P) -> [Note; S],
        filter_args: P,
        sort_by: [Sort; N],
        skip: u32,
    ) -> Self {
        NoteGetterOptions {
            sort_by,
            skip,
            filter,
            filter_args,
        }
    }

    fn with_filter(
        filter: fn ([Note; MAX_READ_REQUESTS], P) -> [Note; S],
        filter_args: P,
    ) -> Self {
         NoteGetterOptions {
            sort_by: [Sort::nada(); N],
            skip: 0,
            filter,
            filter_args,
        }
    }
}
