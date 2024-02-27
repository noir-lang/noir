use dep::std::option::Option;
use crate::note::note_getter_options::{Select, Sort, Comparator, NoteStatus};
use dep::protocol_types::{constants::MAX_NOTES_PER_PAGE};
use crate::note::note_interface::NoteInterface;

// docs:start:NoteViewerOptions
struct NoteViewerOptions<Note, N> {
    selects: BoundedVec<Option<Select>, N>,
    sorts: BoundedVec<Option<Sort>, N>,
    limit: u32,
    offset: u32,
    status: u8,
}
// docs:end:NoteViewerOptions

impl<Note, N> NoteViewerOptions<Note, N> {
    pub fn new() -> NoteViewerOptions<Note, N> where Note: NoteInterface<N> {
        NoteViewerOptions {
            selects: BoundedVec::new(),
            sorts: BoundedVec::new(),
            limit: MAX_NOTES_PER_PAGE as u32,
            offset: 0,
            status: NoteStatus.ACTIVE
        }
    }

    // This method adds a `Select` criterion to the options.
    // It takes a field_index indicating which field to select,
    // a value representing the specific value to match in that field, and
    // a comparator (For possible values of comparators, please see the Comparator enum from note_getter_options)
    pub fn select(&mut self, field_index: u8, value: Field, comparator: Option<u8>) -> Self {
        self.selects.push(Option::some(Select::new(field_index, value, comparator.unwrap_or(Comparator.EQ))));
        *self
    }

    pub fn sort(&mut self, field_index: u8, order: u8) -> Self {
        self.sorts.push(Option::some(Sort::new(field_index, order)));
        *self
    }

    pub fn set_limit(&mut self, limit: u32) -> Self {
        assert(limit <= MAX_NOTES_PER_PAGE as u32);
        self.limit = limit;
        *self
    }

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
