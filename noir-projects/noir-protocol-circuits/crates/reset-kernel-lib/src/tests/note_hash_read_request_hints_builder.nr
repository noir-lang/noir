use crate::{
    note_hash_read_request_reset::{NoteHashSettledReadHint, NoteHashReadRequestHints},
    reset::read_request::{PendingReadHint, ReadRequestStatus}
};
use dep::types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_TX;

struct NoteHashReadRequestHintsBuilder {
    read_request_statuses: [ReadRequestStatus; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    pending_read_hints: BoundedVec<PendingReadHint, MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    settled_read_hints: BoundedVec<NoteHashSettledReadHint, MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
}

impl NoteHashReadRequestHintsBuilder {
    pub fn new(read_request_len: u64) -> Self {
        NoteHashReadRequestHintsBuilder {
            read_request_statuses: [ReadRequestStatus::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
            pending_read_hints: BoundedVec { storage: [PendingReadHint::nada(read_request_len); MAX_NOTE_HASH_READ_REQUESTS_PER_TX], len: 0 },
            settled_read_hints: BoundedVec {
                storage: [NoteHashSettledReadHint::nada(read_request_len); MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
                len: 0
            }
        }
    }

    pub fn to_hints(self) -> NoteHashReadRequestHints {
        NoteHashReadRequestHints {
            read_request_statuses: self.read_request_statuses,
            pending_read_hints: self.pending_read_hints.storage,
            settled_read_hints: self.settled_read_hints.storage
        }
    }
}
