// This will be moved to a separate Read Request Reset Circuit.
use dep::types::{
    abis::{read_request::ReadRequestContext, side_effect::OrderedValue}, hash::{silo_nullifier},
    merkle_tree::{assert_check_membership, LeafPreimage, MembershipWitness}, traits::{Empty, is_empty}
};

struct ReadRequestStateEnum {
    NADA: u8,
    PENDING: u8,
    SETTLED: u8,
}

global ReadRequestState = ReadRequestStateEnum {
    NADA: 0,
    PENDING: 1,
    SETTLED: 2,
};

struct ReadRequestStatus {
    state: u8,
    hint_index: u64,
}

impl Empty for ReadRequestStatus {
    fn empty() -> Self {
        ReadRequestStatus { state: ReadRequestState.NADA, hint_index: 0 }
    }
}

trait ReadValueHint {
    fn read_request_index(self) -> u64;
}

struct PendingReadHint {
    read_request_index: u64,
    pending_value_index: u64,
}

impl ReadValueHint for PendingReadHint {
    fn read_request_index(self) -> u64 {
        self.read_request_index
    }
}

impl PendingReadHint {
    pub fn nada(read_request_len: u64) -> Self {
        PendingReadHint { read_request_index: read_request_len, pending_value_index: 0 }
    }
}

trait SettledReadHint<TREE_HEIGHT, LEAF_PREIMAGE> where LEAF_PREIMAGE: LeafPreimage {
    fn membership_witness(self) -> MembershipWitness<TREE_HEIGHT>;
    fn leaf_preimage(self) -> LEAF_PREIMAGE;
    fn nada(read_request_len: u64) -> Self;
}

// Validate the values being read were emitted in the same transaction before the read request were made.
// More info here:
// - https://discourse.aztec.network/t/to-read-or-not-to-read/178
// - https://discourse.aztec.network/t/spending-notes-which-havent-yet-been-inserted/180
fn validate_pending_read_requests<READ_REQUEST_LEN, T, PENDING_VALUE_LEN, NUM_PENDING_READS>(
    read_requests: [ReadRequestContext; READ_REQUEST_LEN],
    pending_values: [T; PENDING_VALUE_LEN],
    hints: [PendingReadHint; NUM_PENDING_READS]
) where T: OrderedValue<Field> { // TODO: Should be ContractScopedOrderedValue.
    for i in 0..NUM_PENDING_READS {
        let read_request_index = hints[i].read_request_index;
        if read_request_index != READ_REQUEST_LEN {
            let read_request = read_requests[read_request_index];
            let siloed_value = silo_nullifier(read_request.contract_address, read_request.value); // TODO: Should be comparing the contract address with ContractScopedOrderedValue.
            let pending_value = pending_values[hints[i].pending_value_index];
            assert(siloed_value.eq(pending_value.value()), "Hinted value does not match read request");
            assert(
                read_request.counter > pending_value.counter(), "Read request counter must be greater than counter of the value being read"
            );
            // TODO: for transient note hash, the counter of the read request must be less than the counter of the nullifier.
        }
    }
}

// Validate read requests against the historical tree root, for reading settled notes.
// Use their membership witnesses to do so.
fn validate_settled_read_requests<READ_REQUEST_LEN, NUM_SETTLED_READS, H, TREE_HEIGHT, LEAF_PREIMAGE>(
    read_requests: [ReadRequestContext; READ_REQUEST_LEN],
    hints: [H; NUM_SETTLED_READS],
    tree_root: Field
) where 
    H: SettledReadHint<TREE_HEIGHT, LEAF_PREIMAGE> + ReadValueHint, 
    LEAF_PREIMAGE: LeafPreimage {
    for i in 0..NUM_SETTLED_READS {
        let read_request_index = hints[i].read_request_index();
        if read_request_index != READ_REQUEST_LEN {
            let read_request = read_requests[read_request_index];
            let siloed_value = silo_nullifier(read_request.contract_address, read_request.value); // TODO: Should silo the value differently based on the type of the read request.
            let leaf_preimage = hints[i].leaf_preimage();
            assert(
                leaf_preimage.get_key() == siloed_value, "Provided leaf preimage is not for target value"
            );
            let leaf = leaf_preimage.as_leaf();
            let witness = hints[i].membership_witness();
            assert_check_membership(leaf, witness.leaf_index, witness.sibling_path, tree_root);
        }
    }
}

fn propagate_unverified_read_requests<READ_REQUEST_LEN, T, NUM_PENDING_READS, S, NUM_SETTLED_READS>(
    read_requests: [ReadRequestContext; READ_REQUEST_LEN],
    read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN],
    pending_read_hints: [T; NUM_PENDING_READS],
    settled_read_hints: [S; NUM_SETTLED_READS]
) -> BoundedVec<ReadRequestContext, READ_REQUEST_LEN> where T: ReadValueHint, S: ReadValueHint {
    let mut propagated_read_requests = BoundedVec::new();
    for i in 0..READ_REQUEST_LEN {
        let read_request = read_requests[i];
        if !is_empty(read_request) {
            let status = read_request_statuses[i];
            if status.state == ReadRequestState.NADA {
                propagated_read_requests.push(read_request);
            } else if status.state == ReadRequestState.PENDING {
                assert(
                    pending_read_hints[status.hint_index].read_request_index() == i, "Hinted pending read request does not match status"
                );
            } else if status.state == ReadRequestState.SETTLED {
                assert(
                    settled_read_hints[status.hint_index].read_request_index() == i, "Hinted settled read request does not match status"
                );
            }
        }
    }
    propagated_read_requests
}

pub fn reset_read_requests<READ_REQUEST_LEN, P, PENDING_VALUE_LEN, NUM_PENDING_READS, NUM_SETTLED_READS, H, TREE_HEIGHT, LEAF_PREIMAGE>(
    read_requests: [ReadRequestContext; READ_REQUEST_LEN],
    pending_values: [P; PENDING_VALUE_LEN],
    read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN],
    pending_read_hints: [PendingReadHint; NUM_PENDING_READS],
    settled_read_hints: [H; NUM_SETTLED_READS],
    tree_root: Field
) -> BoundedVec<ReadRequestContext, READ_REQUEST_LEN> where 
    P: OrderedValue<Field>, 
    H: SettledReadHint<TREE_HEIGHT, LEAF_PREIMAGE> + ReadValueHint,
    LEAF_PREIMAGE: LeafPreimage {
    validate_pending_read_requests(read_requests, pending_values, pending_read_hints);

    validate_settled_read_requests(read_requests, settled_read_hints, tree_root);

    propagate_unverified_read_requests(
        read_requests,
        read_request_statuses,
        pending_read_hints,
        settled_read_hints
    )
}

mod tests {
    use crate::read_request_reset::{
        PendingReadHint, ReadRequestState, ReadRequestStatus, ReadValueHint, SettledReadHint,
        propagate_unverified_read_requests, reset_read_requests, validate_pending_read_requests,
        validate_settled_read_requests
    };
    use dep::std::{hash::pedersen_hash, unsafe};
    use dep::types::{
        address::AztecAddress, abis::{read_request::ReadRequestContext, side_effect::SideEffect},
        merkle_tree::{LeafPreimage, MembershipWitness}, hash::silo_nullifier,
        tests::merkle_tree_utils::NonEmptyMerkleTree
    };

    struct TestLeafPreimage {
        value: Field,
    }

    impl LeafPreimage for TestLeafPreimage {
        fn get_key(self) -> Field {
            self.value
        }

        fn as_leaf(self) -> Field {
            pedersen_hash([self.value])
        }
    }

    struct TestSettledReadHint {
        read_request_index: u64,
        membership_witness: MembershipWitness<3>,
        leaf_preimage: TestLeafPreimage,
    }

    impl ReadValueHint for TestSettledReadHint {
        fn read_request_index(self) -> u64 {
            self.read_request_index
        }
    }

    impl SettledReadHint<3, TestLeafPreimage> for TestSettledReadHint {
        fn membership_witness(self) -> MembershipWitness<3> {
            self.membership_witness
        }

        fn leaf_preimage(self) -> TestLeafPreimage {
            self.leaf_preimage
        }

        fn nada(read_request_len: u64) -> Self {
            TestSettledReadHint {
                read_request_index: read_request_len,
                membership_witness: unsafe::zeroed(),
                leaf_preimage: unsafe::zeroed()
            }
        }
    }

    global contract_address = AztecAddress::from_field(123);

    // Create 4 values. 10 and 11 are settled. 12 and 13 are pending.
    global values = [10, 11, 12, 13];
    global siloed_values = values.map(|n| silo_nullifier(contract_address, n));

    // Create 4 read requests. 0 and 3 are reading settled values. 1 and 2 are reading pending values.
    global read_requests = [
        ReadRequestContext { value: values[1], counter: 11, contract_address }, // settled
        ReadRequestContext { value: values[3], counter: 13, contract_address }, // pending
        ReadRequestContext { value: values[2], counter: 39, contract_address }, // pending
        ReadRequestContext { value: values[0], counter: 46, contract_address }, // settled
    ];

    global pending_values = [
        SideEffect { value: siloed_values[2], counter: 2 },
        SideEffect { value: siloed_values[3], counter: 8 },
    ];
    global pending_read_hints = [
        PendingReadHint { read_request_index: 1, pending_value_index: 1 }, 
        PendingReadHint { read_request_index: 2, pending_value_index: 0 },
    ];

    global leaf_preimages = [
        TestLeafPreimage { value: siloed_values[0] },
        TestLeafPreimage { value: siloed_values[1] },
    ];

    fn build_tree<N>() -> NonEmptyMerkleTree<2, 3, 2, 1> {
        NonEmptyMerkleTree::new(
            [leaf_preimages[0].as_leaf(), leaf_preimages[1].as_leaf()],
            [0; 3],
            [0; 2],
            [0; 1]
        )
    }

    fn get_settled_read_hints() -> ([TestSettledReadHint; 2], Field) {
        let tree = build_tree();
        let hints = [
            TestSettledReadHint {
                read_request_index: 0,
                membership_witness: MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) },
                leaf_preimage: leaf_preimages[1]
            },
            TestSettledReadHint {
                read_request_index: 3,
                membership_witness: MembershipWitness { leaf_index: 0, sibling_path: tree.get_sibling_path(0) },
                leaf_preimage: leaf_preimages[0]
            }
        ];
        let tree_root = tree.get_root();
        (hints, tree_root)
    }

    #[test]
    fn test_validate_pending_read_requests() {
        validate_pending_read_requests(read_requests, pending_values, pending_read_hints);
    }

    #[test]
    fn test_partial_validate_pending_read_requests() {
        let hints = [pending_read_hints[1]];
        validate_pending_read_requests(read_requests, pending_values, hints);
    }

    #[test(should_fail_with="Hinted value does not match read request")]
    fn test_validate_pending_read_requests_wrong_hint_fails() {
        let mut hint = pending_read_hints[1];
        hint.pending_value_index = 1;
        let hints = [hint];
        validate_pending_read_requests(read_requests, pending_values, hints);
    }

    #[test(should_fail_with="Read request counter must be greater than counter of the value being read")]
    fn test_validate_pending_read_requests_reads_later_value_fails() {
        let read_requests = [
            ReadRequestContext { value: values[3], counter: pending_values[1].counter - 1, contract_address }
        ];
        let hints = [PendingReadHint { read_request_index: 0, pending_value_index: 1 }];
        validate_pending_read_requests(read_requests, pending_values, hints);
    }

    #[test]
    fn test_validate_settled_read_requests() {
        let (hints, tree_root) = get_settled_read_hints();
        validate_settled_read_requests(read_requests, hints, tree_root);
    }

    #[test]
    fn test_partial_validate_settled_read_requests() {
        let (settled_hints, tree_root) = get_settled_read_hints();
        let hints = [settled_hints[0]];
        validate_settled_read_requests(read_requests, hints, tree_root);
    }

    #[test(should_fail_with="membership check failed")]
    fn test_validate_settled_read_requests_wrong_witness_fails() {
        let (settled_hints, tree_root) = get_settled_read_hints();
        let mut hint = settled_hints[0];
        hint.membership_witness.leaf_index += 1;
        let hints = [hint];
        validate_settled_read_requests(read_requests, hints, tree_root);
    }

    #[test(should_fail_with="Provided leaf preimage is not for target value")]
    fn test_validate_settled_read_requests_wrong_preimage_value_fails() {
        let (settled_hints, tree_root) = get_settled_read_hints();
        let mut hint = settled_hints[0];
        hint.leaf_preimage.value = siloed_values[0];
        let hints = [hint];
        validate_settled_read_requests(read_requests, hints, tree_root);
    }

    #[test]
    fn test_propagate_unverified_read_requests() {
        let read_request_statuses = [
            ReadRequestStatus::empty(),
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus::empty(),
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 }
        ];
        let pending_read_hints = [pending_read_hints[0]];
        let (settled_read_hints, _) = get_settled_read_hints();
        let settled_read_hints = [settled_read_hints[1]];
        let unverified_read_requests = propagate_unverified_read_requests(
            read_requests,
            read_request_statuses,
            pending_read_hints,
            settled_read_hints
        );
        assert(unverified_read_requests.len() == 2);
        assert(unverified_read_requests.get(0) == read_requests[0]);
        assert(unverified_read_requests.get(1) == read_requests[2]);
    }

    #[test]
    fn test_propagate_unverified_read_requests_clears_all() {
        let read_request_statuses = [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 }
        ];
        let (settled_read_hints, _) = get_settled_read_hints();
        let unverified_read_requests = propagate_unverified_read_requests(
            read_requests,
            read_request_statuses,
            pending_read_hints,
            settled_read_hints
        );
        assert(unverified_read_requests.len() == 0);
    }

    #[test(should_fail_with="Hinted pending read request does not match status")]
    fn test_propagate_unverified_read_requests_wrong_pending_read_status_fails() {
        let read_request_statuses = [
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 }
        ];
        let (settled_read_hints, _) = get_settled_read_hints();
        let _ = propagate_unverified_read_requests(
            read_requests,
            read_request_statuses,
            pending_read_hints,
            settled_read_hints
        );
    }

    #[test(should_fail_with="Hinted settled read request does not match status")]
    fn test_propagate_unverified_read_requests_wrong_settled_read_status_fails() {
        let read_request_statuses = [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 }
        ];
        let (settled_read_hints, _) = get_settled_read_hints();
        let _ = propagate_unverified_read_requests(
            read_requests,
            read_request_statuses,
            pending_read_hints,
            settled_read_hints
        );
    }

    #[test]
    fn test_reset_read_requests_all() {
        let read_request_statuses = [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 }
        ];
        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let unverified_read_requests = reset_read_requests(
            read_requests,
            pending_values,
            read_request_statuses,
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
        assert(unverified_read_requests.len() == 0);
    }
}

