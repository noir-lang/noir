use dep::std::option::Option;
use crate::constants_gen::MAX_NOTES_PER_PAGE;
use crate::note::note_getter_options::{Select, Sort};
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
    fn new() -> NoteViewerOptions<Note, N> {
        NoteViewerOptions {
            selects: BoundedVec::new(Option::none()),
            sorts: BoundedVec::new(Option::none()),
            limit: MAX_NOTES_PER_PAGE as u32,
            offset: 0,
        }
    }

   fn select(&mut self, field_index: u8, value: Field) -> Self {
        self.selects.push(Option::some(Select::new(field_index, value)));
        *self
    }

    fn sort(&mut self, field_index: u8, order: u2) -> Self {
        self.sorts.push(Option::some(Sort::new(field_index, order)));
        *self
    }

    fn set_limit(&mut self, limit: u32) -> Self {
        assert(limit <= MAX_NOTES_PER_PAGE as u32);
        self.limit = limit;
        *self
    }

    fn set_offset(&mut self, offset: u32) -> Self {
        self.offset = offset;
        *self
    }
}
