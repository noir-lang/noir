use crate::{
    nullifier_read_request_reset::{NullifierSettledReadHint, NullifierReadRequestHints},
    read_request_reset::{PendingReadHint, ReadRequestStatus}
};
use dep::types::constants::MAX_NULLIFIER_READ_REQUESTS_PER_TX;

struct NullifierReadRequestHintsBuilder {
    read_request_statuses: [ReadRequestStatus; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    pending_read_hints: BoundedVec<PendingReadHint, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    settled_read_hints: BoundedVec<NullifierSettledReadHint, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
}

impl NullifierReadRequestHintsBuilder {
    pub fn new(read_request_len: u64) -> Self {
        NullifierReadRequestHintsBuilder {
            read_request_statuses: [ReadRequestStatus::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_TX],
            pending_read_hints: BoundedVec { storage: [PendingReadHint::nada(read_request_len); MAX_NULLIFIER_READ_REQUESTS_PER_TX], len: 0 },
            settled_read_hints: BoundedVec {
                storage: [NullifierSettledReadHint::nada(read_request_len); MAX_NULLIFIER_READ_REQUESTS_PER_TX],
                len: 0
            }
        }
    }

    pub fn to_hints(self) -> NullifierReadRequestHints {
        NullifierReadRequestHints {
            read_request_statuses: self.read_request_statuses,
            pending_read_hints: self.pending_read_hints.storage,
            settled_read_hints: self.settled_read_hints.storage
        }
    }
}
