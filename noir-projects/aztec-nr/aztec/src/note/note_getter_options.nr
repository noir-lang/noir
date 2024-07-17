use std::option::Option;
use dep::protocol_types::{constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, traits::ToField};
use crate::note::note_interface::NoteInterface;

struct PropertySelector {
    index: u8,
    offset: u8,
    length: u8,
}

struct ComparatorEnum {
    EQ: u8,
    NEQ: u8,
    LT: u8,
    LTE: u8,
    GT: u8,
    GTE: u8,
}

global Comparator = ComparatorEnum {
    EQ: 1,
    NEQ: 2,
    LT: 3,
    LTE: 4,
    GT: 5,
    GTE: 6,
};

struct Select {
    property_selector: PropertySelector,
    value: Field,
    comparator: u8,
}

impl Select {
    pub fn new(property_selector: PropertySelector, value: Field, comparator: u8) -> Self {
        Select { property_selector, value, comparator }
    }
}

struct SortOrderEnum {
    DESC: u8,
    ASC: u8,
}

global SortOrder = SortOrderEnum {
    DESC: 1,
    ASC: 2,
};

struct Sort {
    property_selector: PropertySelector,
    order: u8,
}

impl Sort {
    pub fn new(property_selector: PropertySelector, order: u8) -> Self {
        Sort { property_selector, order }
    }
}

struct NoteStatusEnum {
    ACTIVE: u8,
    ACTIVE_OR_NULLIFIED: u8,
}

global NoteStatus = NoteStatusEnum {
    ACTIVE: 1,
    ACTIVE_OR_NULLIFIED: 2,
    // TODO 4217: add 'NULLIFIED'
};

fn return_all_notes<Note, let N: u32>(
    notes: [Option<Note>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    _p: Field
) -> [Option<Note>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] {
    notes
}

// docs:start:NoteGetterOptions
struct NoteGetterOptions<Note, let N: u32, let M: u32, FILTER_ARGS> {
    selects: BoundedVec<Option<Select>, N>,
    sorts: BoundedVec<Option<Sort>, N>,
    limit: u32,
    offset: u32,
    filter: fn ([Option<Note>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL], FILTER_ARGS) -> [Option<Note>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    filter_args: FILTER_ARGS,
    status: u8,
}
// docs:end:NoteGetterOptions

// When retrieving notes using the NoteGetterOptions, the configurations are applied in a specific sequence to ensure precise and controlled data retrieval.
// The database-level configurations are applied first:
// `selects` to specify fields, `sorts` to establish sorting criteria, `offset` to skip items, and `limit` to cap the result size.
// And finally, a custom filter to refine the outcome further.
impl<Note, let N: u32, let M: u32, FILTER_ARGS> NoteGetterOptions<Note, N, M, FILTER_ARGS> {
    // This function initializes a NoteGetterOptions that simply returns the maximum number of notes allowed in a call.
    pub fn new() -> NoteGetterOptions<Note, N, M, Field> where Note: NoteInterface<N, M> {
        NoteGetterOptions {
            selects: BoundedVec::new(),
            sorts: BoundedVec::new(),
            limit: MAX_NOTE_HASH_READ_REQUESTS_PER_CALL as u32,
            offset: 0,
            filter: return_all_notes,
            filter_args: 0,
            status: NoteStatus.ACTIVE
        }
    }

    // This function initializes a NoteGetterOptions with a filter, which takes the notes returned from the database and filter_args as its parameters.
    // `filter_args` allows you to provide additional data or context to the custom filter.
    pub fn with_filter(
        filter: fn([Option<Note>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL], FILTER_ARGS) -> [Option<Note>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
        filter_args: FILTER_ARGS
    ) -> Self where Note: NoteInterface<N, M> {
        NoteGetterOptions {
            selects: BoundedVec::new(),
            sorts: BoundedVec::new(),
            limit: MAX_NOTE_HASH_READ_REQUESTS_PER_CALL as u32,
            offset: 0,
            filter,
            filter_args,
            status: NoteStatus.ACTIVE
        }
    }

    // This method adds a `Select` criterion to the options.
    // It takes a property_selector indicating which field to select, 
    // a value representing the specific value to match in that field, and 
    // a comparator (For possible values of comparators, please see the Comparator enum above)
    pub fn select<T>(
        &mut self,
        property_selector: PropertySelector,
        value: T,
        comparator: Option<u8>
    ) -> Self where T: ToField {
        self.selects.push(
            Option::some(
                Select::new(
                    property_selector,
                    value.to_field(),
                    comparator.unwrap_or(Comparator.EQ)
                )
            )
        );
        *self
    }

    // This method adds a `Sort` criterion to the options.
    // It takes a field_index indicating which field to sort by and an order (SortOrder) to determine the sorting direction.
    pub fn sort(&mut self, property_selector: PropertySelector, order: u8) -> Self {
        self.sorts.push(Option::some(Sort::new(property_selector, order)));
        *self
    }

    // This method lets you set a limit for the maximum number of notes to be retrieved in a single query result.
    pub fn set_limit(&mut self, limit: u32) -> Self {
        assert(limit <= MAX_NOTE_HASH_READ_REQUESTS_PER_CALL as u32);
        // By requesting that the limit is a constant, we guarantee that it will be possible to loop over it, reducing
        // gate counts when a limit has been set.
        assert_constant(limit);

        self.limit = limit;
        *self
    }

    // This method sets the offset value, which determines where to start retrieving notes in the query results.
    pub fn set_offset(&mut self, offset: u32) -> Self {
        self.offset = offset;
        *self
    }

    // This method sets the status value, which determines whether to retrieve active or nullified notes.
    pub fn set_status(&mut self, status: u8) -> Self {
        self.status = status;
        *self
    }
}
