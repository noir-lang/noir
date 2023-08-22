use dep::std::option::Option;
use crate::types::vec::BoundedVec;
use crate::constants_gen::MAX_READ_REQUESTS_PER_CALL;

struct Select {
    field_index: u8,
    value: Field,
}

impl Select {
    fn new(field_index: u8, value: Field) -> Self {
        Select { field_index, value }
    }
}

struct SortOrderEnum {
    DESC: u2,
    ASC: u2,
    NADA: u2,
}

global SortOrder = SortOrderEnum {
    DESC: 1,
    ASC: 2,
    NADA: 0,
};

struct Sort {
    field_index: u8,
    order: u2,
}

impl Sort {
    fn new(field_index: u8, order: u2) -> Self {
        Sort { field_index, order }
    }

    fn nada() -> Self {
        Sort { field_index: 0, order: SortOrder.NADA }
    }
}

fn return_all_notes<Note, N>(notes: [Option<Note>; MAX_READ_REQUESTS_PER_CALL], _p: Field) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL] {
    notes
}

// docs:start:NoteGetterOptions
struct NoteGetterOptions<Note, N, FILTER_ARGS> {
    selects: BoundedVec<Option<Select>, N>,
    sorts: BoundedVec<Option<Sort>, N>,
    limit: u32,
    offset: u32,
    filter: fn ([Option<Note>; MAX_READ_REQUESTS_PER_CALL], FILTER_ARGS) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL],
    filter_args: FILTER_ARGS,
}
// docs:end:NoteGetterOptions

// When retrieving notes using the NoteGetterOptions, the configurations are applied in a specific sequence to ensure precise and controlled data retrieval.
// The database-level configurations are applied first:
// `selects` to specify fields, `sorts` to establish sorting criteria, `offset` to skip items, and `limit` to cap the result size.
// And finally, a custom filter to refine the outcome further.
impl<Note, N, FILTER_ARGS> NoteGetterOptions<Note, N, FILTER_ARGS> {
    // This function initialises a NoteGetterOptions that simply returns the maximum number of notes allowed in a call.
    fn new() -> NoteGetterOptions<Note, N, Field> {
        NoteGetterOptions {
            selects: BoundedVec::new(Option::none()),
            sorts: BoundedVec::new(Option::none()),
            limit: MAX_READ_REQUESTS_PER_CALL as u32,
            offset: 0,
            filter: return_all_notes,
            filter_args: 0,
        }
    }

    // This function initialises a NoteGetterOptions with a filter, which takes the notes returned from the database and filter_args as its parameters.
    // `filter_args` allows you to provide additional data or context to the custom filter.
    fn with_filter(
        filter: fn ([Option<Note>; MAX_READ_REQUESTS_PER_CALL], FILTER_ARGS) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL],
        filter_args: FILTER_ARGS,
    ) -> Self {
         NoteGetterOptions {
            selects: BoundedVec::new(Option::none()),
            sorts: BoundedVec::new(Option::none()),
            limit: MAX_READ_REQUESTS_PER_CALL as u32,
            offset: 0,
            filter,
            filter_args,
        }
    }

    // This method adds a `Select` criterion to the options.
    // It takes a field_index indicating which field to select and a value representing the specific value to match in that field.
    fn select(&mut self, field_index: u8, value: Field) -> Self {
        self.selects.push(Option::some(Select::new(field_index, value)));
        *self
    }

    // This method adds a `Sort` criterion to the options.
    // It takes a field_index indicating which field to sort by and an order (SortOrder) to determine the sorting direction.
    fn sort(&mut self, field_index: u8, order: u2) -> Self {
        self.sorts.push(Option::some(Sort::new(field_index, order)));
        *self
    }

    // This method lets you set a limit for the maximum number of notes to be retrieved in a single query result. 
    fn set_limit(&mut self, limit: u32) -> Self {
        assert(limit <= MAX_READ_REQUESTS_PER_CALL as u32);
        self.limit = limit;
        *self
    }

    // This method sets the offset value, which determines where to start retrieving notes in the query results. 
    fn set_offset(&mut self, offset: u32) -> Self {
        self.offset = offset;
        *self
    }
}
