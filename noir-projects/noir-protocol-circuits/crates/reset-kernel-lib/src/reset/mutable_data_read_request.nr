use crate::reset::read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus};
use dep::types::{
    abis::{public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest},
    traits::{Empty, is_empty}
};

trait LeafDataHint {
    fn leaf_slot(self) -> Field;
    fn value(self) -> Field;
    fn override_counter(self) -> u32;
}

struct LeafDataReadHint {
    read_request_index: u64,
    data_hint_index: u64,
}

impl LeafDataReadHint {
    pub fn nada(read_request_len: u64) -> Self {
        LeafDataReadHint { read_request_index: read_request_len, data_hint_index: 0 }
    }
}

fn validate_pending_read_requests<READ_REQUEST_LEN, PENDING_VALUE_LEN, NUM_PENDING_READS>(
    read_requests: [PublicDataRead; READ_REQUEST_LEN],
    data_writes: [PublicDataUpdateRequest; PENDING_VALUE_LEN],
    hints: [PendingReadHint; NUM_PENDING_READS]
) {
    for i in 0..NUM_PENDING_READS {
        let read_request_index = hints[i].read_request_index;
        if read_request_index != READ_REQUEST_LEN {
            let read_request = read_requests[read_request_index];
            let pending_value = data_writes[hints[i].pending_value_index];
            assert(
                read_request.leaf_slot.eq(pending_value.leaf_slot), "Hinted slot of data write does not match read request"
            );
            assert(
                read_request.value.eq(pending_value.new_value), "Hinted value of data write does not match read request"
            );
            // TODO: Add counters and verify the following:
            // assert(
            //     read_request.counter > pending_value.counter, "Read request counter must be greater than the counter of the data write"
            // );
            // assert((read_request.counter < pending_value.next_counter) | (pending_value.next_counter == 0), "Read request counter must be less than the counter of the next data write");
        }
    }
}

fn validate_leaf_data_read_requests<READ_REQUEST_LEN, NUM_LEAF_DATA_HINTS, H, NUM_LEAF_DATA_READS>(
    read_requests: [PublicDataRead; READ_REQUEST_LEN],
    leaf_data_hints: [H; NUM_LEAF_DATA_HINTS],
    hints: [LeafDataReadHint; NUM_LEAF_DATA_READS]
) where H: LeafDataHint {
    for i in 0..NUM_LEAF_DATA_READS {
        let read_request_index = hints[i].read_request_index;
        if read_request_index != READ_REQUEST_LEN {
            let read_request = read_requests[read_request_index];
            let data_hint = leaf_data_hints[hints[i].data_hint_index];
            assert(
                read_request.leaf_slot == data_hint.leaf_slot(), "Hinted slot does not match read request"
            );
            assert(read_request.value == data_hint.value(), "Hinted value does not match read request");
            // TODO: Add counters and verify the following:
            // assert((read_request.counter < data_hint.override_counter) | (data_hint.override_counter == 0), "Hinted leaf is overridden before the read request");
        }
    }
}

fn ensure_all_read_requests_are_verified<READ_REQUEST_LEN, NUM_PENDING_READS, NUM_LEAF_DATA_READS>(
    read_requests: [PublicDataRead; READ_REQUEST_LEN],
    read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN],
    pending_read_hints: [PendingReadHint; NUM_PENDING_READS],
    leaf_data_read_hints: [LeafDataReadHint; NUM_LEAF_DATA_READS]
) {
    for i in 0..READ_REQUEST_LEN {
        let read_request = read_requests[i];
        if !is_empty(read_request) {
            let status = read_request_statuses[i];
            if status.state == ReadRequestState.PENDING {
                assert(
                    pending_read_hints[status.hint_index].read_request_index == i, "Hinted pending read request does not match status"
                );
            } else if status.state == ReadRequestState.SETTLED {
                assert(
                    leaf_data_read_hints[status.hint_index].read_request_index == i, "Hinted settled read request does not match status"
                );
            } else {
                assert(false, "Read request status must be PENDING or SETTLED");
            }
        }
    }
}

pub fn reset_mutable_data_read_requests<READ_REQUEST_LEN, PENDING_VALUE_LEN, H, NUM_LEAF_DATA_HINTS, NUM_PENDING_READS, NUM_LEAF_DATA_READS>(
    read_requests: [PublicDataRead; READ_REQUEST_LEN],
    read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN],
    data_writes: [PublicDataUpdateRequest; PENDING_VALUE_LEN],
    leaf_data_hints: [H; NUM_LEAF_DATA_HINTS],
    pending_read_hints: [PendingReadHint; NUM_PENDING_READS],
    leaf_data_read_hints: [LeafDataReadHint; NUM_LEAF_DATA_READS]
) where H: LeafDataHint {
    validate_pending_read_requests(read_requests, data_writes, pending_read_hints);

    validate_leaf_data_read_requests(read_requests, leaf_data_hints, leaf_data_read_hints);

    ensure_all_read_requests_are_verified(
        read_requests,
        read_request_statuses,
        pending_read_hints,
        leaf_data_read_hints
    );
}

mod tests {
    use crate::reset::{
        mutable_data_read_request::{
        ensure_all_read_requests_are_verified, reset_mutable_data_read_requests, LeafDataHint,
        LeafDataReadHint, validate_pending_read_requests, validate_leaf_data_read_requests
    },
        read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus}
    };
    use dep::types::{abis::{public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest}};

    struct TestLeafDataHint {
        leaf_slot: Field,
        value: Field,
    }

    impl LeafDataHint for TestLeafDataHint {
        fn leaf_slot(self) -> Field {
            self.leaf_slot
        }

        fn value(self) -> Field {
            self.value
        }
        
        fn override_counter(_self: Self) -> u32 {
            0
        }
    }

    global data_writes = [
        PublicDataUpdateRequest { leaf_slot: 22, new_value: 200 },
        PublicDataUpdateRequest { leaf_slot: 11, new_value: 100 },
        PublicDataUpdateRequest { leaf_slot: 33, new_value: 300 },
        PublicDataUpdateRequest { leaf_slot: 44, new_value: 400 }
    ];

    global leaf_data_hints = [
        TestLeafDataHint { leaf_slot: 7, value: 70 },
        TestLeafDataHint { leaf_slot: 6, value: 60 },
        TestLeafDataHint { leaf_slot: 5, value: 50 },
    ];

    fn create_pending_read_requests<N>(data_write_indices: [u64; N]) -> ([PublicDataRead; N], [PendingReadHint; N]) {
        let read_requests = data_write_indices.map(
            |data_write_index: u64| PublicDataRead { leaf_slot: data_writes[data_write_index].leaf_slot, value: data_writes[data_write_index].new_value }
        );
        let mut hints = BoundedVec::new();
        for i in 0..N {
            hints.push(PendingReadHint { read_request_index: i, pending_value_index: data_write_indices[i] });
        }
        (read_requests, hints.storage)
    }

    fn create_leaf_data_read_requests<N>(data_hint_indices: [u64; N]) -> ([PublicDataRead; N], [LeafDataReadHint; N]) {
        let read_requests = data_hint_indices.map(
            |data_hint_index: u64| PublicDataRead { leaf_slot: leaf_data_hints[data_hint_index].leaf_slot, value: leaf_data_hints[data_hint_index].value }
        );
        let mut hints = BoundedVec::new();
        for i in 0..N {
            hints.push(LeafDataReadHint { read_request_index: i, data_hint_index: data_hint_indices[i] });
        }
        (read_requests, hints.storage)
    }

    #[test]
    fn reset_pending_reads_succeeds() {
        let (read_requests, hints) = create_pending_read_requests([2, 0, 1, 3]);
        validate_pending_read_requests(read_requests, data_writes, hints);
    }

    #[test]
    fn reset_pending_reads_repeated_values() {
        let (read_requests, hints) = create_pending_read_requests([1, 0, 0, 1]);
        validate_pending_read_requests(read_requests, data_writes, hints);
    }

    #[test]
    fn reset_pending_reads_skips_nada() {
        let read_requests = [PublicDataRead { leaf_slot: 88, value: 9999 }];
        let hints = [PendingReadHint::nada(1)];
        validate_pending_read_requests(read_requests, data_writes, hints);
    }

    #[test(should_fail_with="Hinted slot of data write does not match read request")]
    fn reset_pending_reads_wrong_slot_fails() {
        let mut (read_requests, hints) = create_pending_read_requests([1]);
        hints[0].pending_value_index = 0;
        validate_pending_read_requests(read_requests, data_writes, hints);
    }

    #[test(should_fail_with="Hinted value of data write does not match read request")]
    fn reset_pending_reads_wrong_value_fails() {
        let mut (read_requests, hints) = create_pending_read_requests([1]);
        read_requests[0].value += 1;
        validate_pending_read_requests(read_requests, data_writes, hints);
    }

    #[test]
    fn reset_leaf_data_reads_succeeds() {
        let (read_requests, hints) = create_leaf_data_read_requests([2, 1, 0]);
        validate_leaf_data_read_requests(read_requests, leaf_data_hints, hints);
    }

    #[test]
    fn reset_leaf_data_reads_repeated_values() {
        let (read_requests, hints) = create_leaf_data_read_requests([1, 0, 1, 0]);
        validate_leaf_data_read_requests(read_requests, leaf_data_hints, hints);
    }

    #[test]
    fn reset_leaf_data_reads_skips_nada() {
        let read_requests = [PublicDataRead { leaf_slot: 88, value: 9999 }];
        let hints = [LeafDataReadHint::nada(1)];
        validate_leaf_data_read_requests(read_requests, leaf_data_hints, hints);
    }

    #[test(should_fail_with=""Hinted slot does not match read request")]
    fn reset_leaf_reads_wrong_slot_fails() {
        let mut (read_requests, hints) = create_leaf_data_read_requests([1]);
        hints[0].data_hint_index = 0;
        validate_leaf_data_read_requests(read_requests, leaf_data_hints, hints);
    }

    #[test(should_fail_with=""Hinted value does not match read request")]
    fn reset_leaf_reads_wrong_value_fails() {
        let mut (read_requests, hints) = create_leaf_data_read_requests([1]);
        read_requests[0].value += 1;
        validate_leaf_data_read_requests(read_requests, leaf_data_hints, hints);
    }

    #[test]
    fn ensure_all_read_requests_are_verified_succeeds() {
        let mut (pending_read_requests, pending_read_hints) = create_pending_read_requests([1]);
        let mut (leaf_read_requests, leaf_data_read_hints) = create_leaf_data_read_requests([0, 1]);
        let read_requests = [leaf_read_requests[0], pending_read_requests[0], leaf_read_requests[1]];
        pending_read_hints[0].read_request_index = 1;
        leaf_data_read_hints[1].read_request_index = 2;

        let statuses = [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 }
        ];

        ensure_all_read_requests_are_verified(
            read_requests,
            statuses,
            pending_read_hints,
            leaf_data_read_hints
        );
    }

    #[test(should_fail_with="Hinted pending read request does not match status")]
    fn ensure_all_read_requests_are_verified_wrong_pending_hint_index_fails() {
        let (read_requests, hints) = create_pending_read_requests([0, 1]);
        let statuses = [
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 }
        ];
        ensure_all_read_requests_are_verified(read_requests, statuses, hints, []);
    }

    #[test(should_fail_with="Hinted settled read request does not match status")]
    fn ensure_all_read_requests_are_verified_wrong_leaf_hint_index_fails() {
        let (read_requests, hints) = create_leaf_data_read_requests([0, 1]);
        let statuses = [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 }
        ];
        ensure_all_read_requests_are_verified(read_requests, statuses, [], hints);
    }

    #[test(should_fail_with="Read request status must be PENDING or SETTLED")]
    fn ensure_all_read_requests_are_verified_wrong_status_fails() {
        let (read_requests, hints) = create_leaf_data_read_requests([0]);
        let statuses = [ReadRequestStatus { state: ReadRequestState.NADA, hint_index: 0 }];
        ensure_all_read_requests_are_verified(read_requests, statuses, [], hints);
    }

    #[test]
    fn reset_mutable_data_read_requests_succeeds() {
        let mut (pending_read_requests, pending_read_hints) = create_pending_read_requests([3, 1]);
        let mut (leaf_read_requests, leaf_data_read_hints) = create_leaf_data_read_requests([0, 1]);
        let read_requests = [
            leaf_read_requests[0], pending_read_requests[0], pending_read_requests[1], leaf_read_requests[1]
        ];
        pending_read_hints[0].read_request_index = 1;
        pending_read_hints[1].read_request_index = 2;
        leaf_data_read_hints[1].read_request_index = 3;

        let statuses = [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 }
        ];

        reset_mutable_data_read_requests(
            read_requests,
            statuses,
            data_writes,
            leaf_data_hints,
            pending_read_hints,
            leaf_data_read_hints
        );
    }
}
