use dep::std::option::Option;
use dep::protocol_types::constants::MAX_NOTES_PER_PAGE;
use crate::note::note_getter_options::{Select, Sort, Comparator};
use crate::types::vec::BoundedVec;

// docs:start:NoteViewerOptions
struct NoteViewerOptions<Note, N> {
    selects: BoundedVec<Option<Select>, N>,
    sorts: BoundedVec<Option<Sort>, N>,
    limit: u32,
    offset: u32,
}
// docs:end:NoteViewerOptions

impl<Note, N> NoteViewerOptions<Note, N> {
    pub fn new() -> NoteViewerOptions<Note, N> {
        NoteViewerOptions {
            selects: BoundedVec::new(Option::none()),
            sorts: BoundedVec::new(Option::none()),
            limit: MAX_NOTES_PER_PAGE as u32,
            offset: 0,
        }
    }
       
    // This method adds a `Select` criterion to the options.
    // It takes a field_index indicating which field to select, 
    // a value representing the specific value to match in that field, and 
    // a comparator (For possible values of comparators, please see the Comparator enum from note_getter_options)
    pub fn select(&mut self, field_index: u8, value: Field, comparator: Option<u3>) -> Self {
        self.selects.push(Option::some(Select::new(field_index, value, comparator.unwrap_or(Comparator.EQ))));
        *self
    }

    pub fn sort(&mut self, field_index: u8, order: u2) -> Self {
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
}
