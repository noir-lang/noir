use crate::{
    note_hash_read_request_reset::{NoteHashSettledReadHint, NoteHashReadRequestHints},
    reset::read_request::{PendingReadHint, ReadRequestStatus}
};
use dep::types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_TX;

struct NoteHashReadRequestHintsBuilder<let NUM_PENDING_HINTS: u32, let NUM_SETTLED_HINTS: u32> {
    read_request_statuses: [ReadRequestStatus; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    pending_read_hints: BoundedVec<PendingReadHint, NUM_PENDING_HINTS>,
    settled_read_hints: BoundedVec<NoteHashSettledReadHint, NUM_SETTLED_HINTS>,
}

impl<let NUM_PENDING_HINTS: u32, let NUM_SETTLED_HINTS: u32> NoteHashReadRequestHintsBuilder<NUM_PENDING_HINTS, NUM_SETTLED_HINTS> {
    pub fn new() -> Self {
        NoteHashReadRequestHintsBuilder {
            read_request_statuses: [ReadRequestStatus::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
            pending_read_hints: BoundedVec { storage: [PendingReadHint::nada(MAX_NOTE_HASH_READ_REQUESTS_PER_TX); NUM_PENDING_HINTS], len: 0 },
            settled_read_hints: BoundedVec {
                storage: [NoteHashSettledReadHint::nada(MAX_NOTE_HASH_READ_REQUESTS_PER_TX); NUM_SETTLED_HINTS],
                len: 0
            }
        }
    }

    pub fn to_hints(self) -> NoteHashReadRequestHints<NUM_PENDING_HINTS, NUM_SETTLED_HINTS> {
        NoteHashReadRequestHints {
            read_request_statuses: self.read_request_statuses,
            pending_read_hints: self.pending_read_hints.storage,
            settled_read_hints: self.settled_read_hints.storage
        }
    }
}
