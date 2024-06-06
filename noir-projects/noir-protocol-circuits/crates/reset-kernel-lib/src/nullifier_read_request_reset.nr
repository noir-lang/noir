// This will be moved to a separate Read Request Reset Circuit.
use crate::reset::read_request::{PendingReadHint, ReadRequestStatus, ReadValueHint, SettledReadHint};
use dep::types::{
    abis::{nullifier_leaf_preimage::NullifierLeafPreimage},
    constants::{MAX_NULLIFIER_READ_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT},
    merkle_tree::MembershipWitness
};

struct NullifierSettledReadHint {
    read_request_index: u32,
    membership_witness: MembershipWitness<NULLIFIER_TREE_HEIGHT>,
    leaf_preimage: NullifierLeafPreimage,
}

impl ReadValueHint for NullifierSettledReadHint {
    fn read_request_index(self) -> u32 {
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

    fn nada(read_request_len: u32) -> Self {
        NullifierSettledReadHint {
            read_request_index: read_request_len,
            membership_witness: MembershipWitness::empty(),
            leaf_preimage: NullifierLeafPreimage::empty()
        }
    }
}

struct NullifierReadRequestHints<PENDING, SETTLED> {
    read_request_statuses: [ReadRequestStatus; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    pending_read_hints: [PendingReadHint; PENDING],
    settled_read_hints: [NullifierSettledReadHint; SETTLED],
}

mod tests {
    use crate::nullifier_read_request_reset::NullifierSettledReadHint;
    use crate::reset::read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus, reset_read_requests};
    use dep::types::{
        address::AztecAddress,
        abis::{nullifier::Nullifier, nullifier_leaf_preimage::NullifierLeafPreimage, read_request::ReadRequest},
        constants::NULLIFIER_TREE_HEIGHT, hash::compute_siloed_nullifier,
        merkle_tree::MembershipWitness, tests::merkle_tree_utils::NonEmptyMerkleTree
    };

    global contract_address = AztecAddress::from_field(123);

    // Create 4 nullifiers. 10 and 11 are settled. 12 and 13 are pending.
    global inner_nullifiers = [10, 11, 12, 13];
    global nullifiers = inner_nullifiers.map(|n| compute_siloed_nullifier(contract_address, n));

    // Create 5 read requests. 0 and 3 are reading settled nullifiers. 1, 2 and 4 are reading pending nullifiers.
    global read_requests = [
        ReadRequest { value: inner_nullifiers[1], counter: 11 }.scope(contract_address), // settled
        ReadRequest { value: inner_nullifiers[3], counter: 13 }.scope(contract_address), // pending
        ReadRequest { value: inner_nullifiers[2], counter: 39 }.scope(contract_address), // pending
        ReadRequest { value: inner_nullifiers[0], counter: 46 }.scope(contract_address), // settled
        ReadRequest { value: inner_nullifiers[3], counter: 78 }.scope(contract_address), // pending
    ];

    global pending_values = [
        Nullifier { value: inner_nullifiers[2], counter: 2, note_hash: 0 }.scope(contract_address),
        Nullifier { value: inner_nullifiers[3], counter: 8, note_hash: 0 }.scope(contract_address),
    ];
    global pending_read_hints = [
        PendingReadHint { read_request_index: 1, pending_value_index: 1 },
        PendingReadHint { read_request_index: 2, pending_value_index: 0 },
        PendingReadHint { read_request_index: 4, pending_value_index: 1 },
    ];

    global leaf_preimages = [
        NullifierLeafPreimage { nullifier: nullifiers[0], next_nullifier: nullifiers[1], next_index: 1 },
        NullifierLeafPreimage { nullifier: nullifiers[1], next_nullifier: 0, next_index: 0 },
    ];

    fn read_request_statuses() -> [ReadRequestStatus; 5] {
        // Failed when assigned to global: 
        // The application panicked (crashed). Message:  no entry found for key
        [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 2 }
        ]
    }

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
    fn test_reset_nullifier_read_requests_all() {
        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let unverified_read_requests = reset_read_requests(
            read_requests,
            pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
        assert(unverified_read_requests.len() == 0);
    }

    #[test]
    fn test_reset_nullifier_read_requests_partial() {
        let mut partial_read_request_statuses = read_request_statuses();
        partial_read_request_statuses[2] = ReadRequestStatus::empty();
        partial_read_request_statuses[4] = ReadRequestStatus::empty();

        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let unverified_read_requests = reset_read_requests(
            read_requests,
            pending_values,
            partial_read_request_statuses,
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
        assert(unverified_read_requests.len() == 2);
        assert(unverified_read_requests.get(0) == read_requests[2]);
        assert(unverified_read_requests.get(1) == read_requests[4]);
    }

    #[test(should_fail_with="Value of the nullifier does not match read request")]
    fn test_reset_nullifier_read_requests_wrong_hinted_value() {
        let mut tainted_pending_values = pending_values;
        // Tweak the value to be something different.
        tainted_pending_values[0].nullifier.value += 1;

        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let _ = reset_read_requests(
            read_requests,
            tainted_pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
    }

    #[test(should_fail_with="Read request counter must be greater than the counter of the nullifier")]
    fn test_reset_nullifier_read_requests_invalid_counter() {
        let hint = pending_read_hints[0];
        let pending_read = read_requests[hint.read_request_index];
        let mut tainted_pending_values = pending_values;
        // Tweak the counter of the value to be greater than the read request.
        tainted_pending_values[hint.pending_value_index].nullifier.counter = pending_read.counter() + 1;

        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let _ = reset_read_requests(
            read_requests,
            tainted_pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
    }

    #[test(should_fail_with="Value of the nullifier leaf does not match read request")]
    fn test_reset_nullifier_read_requests_invalid_leaf() {
        let (settled_read_hints, tree_root) = get_settled_read_hints();

        let mut tained_read_requests = read_requests;
        let hint = settled_read_hints[0];
        // Tweak the value of the first settled read to be something different.
        tained_read_requests[hint.read_request_index].read_request.value += 1;

        let _ = reset_read_requests(
            tained_read_requests,
            pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
    }
}

