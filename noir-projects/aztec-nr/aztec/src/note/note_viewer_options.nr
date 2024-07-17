use std::option::Option;
use crate::note::note_getter_options::{PropertySelector, Select, Sort, Comparator, NoteStatus};
use dep::protocol_types::traits::ToField;
use crate::note::note_interface::NoteInterface;
use crate::note::constants::MAX_NOTES_PER_PAGE;

// docs:start:NoteViewerOptions
struct NoteViewerOptions<Note, let N: u32, let M: u32> {
    selects: BoundedVec<Option<Select>, N>,
    sorts: BoundedVec<Option<Sort>, N>,
    limit: u32,
    offset: u32,
    status: u8,
}
// docs:end:NoteViewerOptions

impl<Note, let N: u32, let M: u32> NoteViewerOptions<Note, N, M> {
    pub fn new() -> NoteViewerOptions<Note, N, M> where Note: NoteInterface<N, M> {
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

    pub fn sort(&mut self, property_selector: PropertySelector, order: u8) -> Self {
        self.sorts.push(Option::some(Sort::new(property_selector, order)));
        *self
    }

    pub fn set_limit(&mut self, limit: u32) -> Self {
        assert(limit <= MAX_NOTES_PER_PAGE as u32);
        // By requesting that the limit is a constant, we guarantee that it will be possible to loop over it, reducing
        // gate counts when a limit has been set. This isn't required in unconstrained code, but we still keep this
        // requirement here for API consistency.
        assert_constant(limit);
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
