// This will be moved to a separate Read Request Reset Circuit.
use crate::reset::read_request::{PendingReadHint, ReadRequestStatus, ReadValueHint, SettledReadHint};
use dep::types::{
    abis::{nullifier_leaf_preimage::NullifierLeafPreimage},
    constants::{MAX_NULLIFIER_READ_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT},
    merkle_tree::MembershipWitness
};

struct NullifierSettledReadHint {
    read_request_index: u64,
    membership_witness: MembershipWitness<NULLIFIER_TREE_HEIGHT>,
    leaf_preimage: NullifierLeafPreimage,
}

impl ReadValueHint for NullifierSettledReadHint {
    fn read_request_index(self) -> u64 {
        self.read_request_index
    }
}

impl SettledReadHint<NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage> for NullifierSettledReadHint {
    fn membership_witness(self) -> MembershipWitness<NULLIFIER_TREE_HEIGHT> {
        self.membership_witness
    }

    fn leaf_preimage(self) -> NullifierLeafPreimage {
        self.leaf_preimage
    }

    fn nada(read_request_len: u64) -> Self {
        NullifierSettledReadHint {
            read_request_index: read_request_len,
            membership_witness: MembershipWitness::empty(),
            leaf_preimage: NullifierLeafPreimage::empty()
        }
    }
}

struct NullifierReadRequestHints {
    read_request_statuses: [ReadRequestStatus; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    pending_read_hints: [PendingReadHint; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    settled_read_hints: [NullifierSettledReadHint; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
}

mod tests {
    use crate::nullifier_read_request_reset::NullifierSettledReadHint;
    use crate::reset::read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus, reset_read_requests};
    use dep::types::{
        address::AztecAddress,
        abis::{
        nullifier_leaf_preimage::NullifierLeafPreimage, read_request::ReadRequestContext,
        side_effect::SideEffect
    },
        constants::NULLIFIER_TREE_HEIGHT, hash::silo_nullifier, merkle_tree::MembershipWitness,
        tests::merkle_tree_utils::NonEmptyMerkleTree
    };

    global contract_address = AztecAddress::from_field(123);

    // Create 4 nullifiers. 10 and 11 are settled. 12 and 13 are pending.
    global inner_nullifiers = [10, 11, 12, 13];
    global nullifiers = inner_nullifiers.map(|n| silo_nullifier(contract_address, n));

    // Create 5 read requests. 0 and 3 are reading settled nullifiers. 1, 2 and 4 are reading pending nullifiers.
    global read_requests = [
        ReadRequestContext { value: inner_nullifiers[1], counter: 11, contract_address }, // settled
        ReadRequestContext { value: inner_nullifiers[3], counter: 13, contract_address }, // pending
        ReadRequestContext { value: inner_nullifiers[2], counter: 39, contract_address }, // pending
        ReadRequestContext { value: inner_nullifiers[0], counter: 46, contract_address }, // settled
        ReadRequestContext { value: inner_nullifiers[3], counter: 78, contract_address }, // pending
    ];

    global pending_values = [SideEffect { value: nullifiers[2], counter: 2 }, SideEffect { value: nullifiers[3], counter: 8 }];
    global pending_read_hints = [
        PendingReadHint { read_request_index: 1, pending_value_index: 1 },
        PendingReadHint { read_request_index: 2, pending_value_index: 0 },
        PendingReadHint { read_request_index: 4, pending_value_index: 1 },
    ];

    global leaf_preimages = [
        NullifierLeafPreimage { nullifier: nullifiers[0], next_nullifier: nullifiers[1], next_index: 1 },
        NullifierLeafPreimage { nullifier: nullifiers[1], next_nullifier: 0, next_index: 0 },
    ];

    fn build_tree<N>() -> NonEmptyMerkleTree<2, NULLIFIER_TREE_HEIGHT, NULLIFIER_TREE_HEIGHT - 1, 1> {
        NonEmptyMerkleTree::new(
            [leaf_preimages[0].hash(), leaf_preimages[1].hash()],
            [0; NULLIFIER_TREE_HEIGHT],
            [0; NULLIFIER_TREE_HEIGHT - 1],
            [0; 1]
        )
    }

    fn get_settled_read_hints() -> ([NullifierSettledReadHint; 2], Field) {
        let tree = build_tree();
        let hints = [
            NullifierSettledReadHint {
                read_request_index: 0,
                membership_witness: MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) },
                leaf_preimage: leaf_preimages[1]
            },
            NullifierSettledReadHint {
                read_request_index: 3,
                membership_witness: MembershipWitness { leaf_index: 0, sibling_path: tree.get_sibling_path(0) },
                leaf_preimage: leaf_preimages[0]
            }
        ];
        let tree_root = tree.get_root();
        (hints, tree_root)
    }

    #[test]
    fn test_reset_read_requests_all() {
        let read_request_statuses = [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 2 }
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

    #[test]
    fn test_reset_read_requests_partial() {
        let read_request_statuses = [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus::empty(),
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 },
            ReadRequestStatus::empty()
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
        assert(unverified_read_requests.len() == 2);
        assert(unverified_read_requests.get(0) == read_requests[2]);
        assert(unverified_read_requests.get(1) == read_requests[4]);
    }
}

