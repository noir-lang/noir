use crate::reset::{mutable_data_read_request::LeafDataReadHint, read_request::{PendingReadHint, ReadRequestStatus}};
use dep::types::constants::MAX_PUBLIC_DATA_READS_PER_TX;

// The MAX_PUBLIC_DATA_READS_PER_TX for pending_read_hints and leaf_data_read_hints can change if we create various circuits that deal with different number of reads.
struct PublicDataReadRequestHints {
    read_request_statuses: [ReadRequestStatus; MAX_PUBLIC_DATA_READS_PER_TX],
    pending_read_hints: [PendingReadHint; MAX_PUBLIC_DATA_READS_PER_TX],
    leaf_data_read_hints: [LeafDataReadHint; MAX_PUBLIC_DATA_READS_PER_TX],
}
