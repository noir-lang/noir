use crate::{
    nullifier_read_request_reset::{NullifierSettledReadHint, NullifierReadRequestHints},
    reset::read_request::{PendingReadHint, ReadRequestStatus}
};
use dep::types::constants::MAX_NULLIFIER_READ_REQUESTS_PER_TX;

struct NullifierReadRequestHintsBuilder<let NUM_PENDING_HINTS: u32, let NUM_SETTLED_HINTS: u32> {
    read_request_statuses: [ReadRequestStatus; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    pending_read_hints: BoundedVec<PendingReadHint, NUM_PENDING_HINTS>,
    settled_read_hints: BoundedVec<NullifierSettledReadHint, NUM_SETTLED_HINTS>,
}

impl<let NUM_PENDING_HINTS: u32, let NUM_SETTLED_HINTS: u32> NullifierReadRequestHintsBuilder<NUM_PENDING_HINTS, NUM_SETTLED_HINTS> {
    pub fn new() -> Self {
        NullifierReadRequestHintsBuilder {
            read_request_statuses: [ReadRequestStatus::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_TX],
            pending_read_hints: BoundedVec { storage: [PendingReadHint::nada(MAX_NULLIFIER_READ_REQUESTS_PER_TX); NUM_PENDING_HINTS], len: 0 },
            settled_read_hints: BoundedVec {
                storage: [NullifierSettledReadHint::nada(MAX_NULLIFIER_READ_REQUESTS_PER_TX); NUM_SETTLED_HINTS],
                len: 0
            }
        }
    }

    pub fn to_hints(self) -> NullifierReadRequestHints<NUM_PENDING_HINTS, NUM_SETTLED_HINTS> {
        NullifierReadRequestHints {
            read_request_statuses: self.read_request_statuses,
            pending_read_hints: self.pending_read_hints.storage,
            settled_read_hints: self.settled_read_hints.storage
        }
    }
}
