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

struct NullifierReadRequestHints<let NUM_PENDING_HINTS: u32, let NUM_SETTLED_HINTS: u32> {
    read_request_statuses: [ReadRequestStatus; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    pending_read_hints: [PendingReadHint; NUM_PENDING_HINTS],
    settled_read_hints: [NullifierSettledReadHint; NUM_SETTLED_HINTS],
}

mod tests {
    use crate::nullifier_read_request_reset::NullifierSettledReadHint;
    use crate::reset::read_request::{
        get_unverified_read_requests, PendingReadHint, ReadRequestState, ReadRequestStatus,
        verify_reset_read_requests
    };
    use dep::types::{
        address::AztecAddress,
        abis::{
        nullifier::{Nullifier, ScopedNullifier}, nullifier_leaf_preimage::NullifierLeafPreimage,
        read_request::{ReadRequest, ScopedReadRequest}
    },
        constants::NULLIFIER_TREE_HEIGHT, hash::compute_siloed_nullifier,
        merkle_tree::MembershipWitness,
        tests::{merkle_tree_utils::NonEmptyMerkleTree, utils::assert_array_eq}, traits::is_empty_array
    };

    struct TestBuilder<let READ_REQUEST_LEN: u32, let PENDING_VALUE_LEN: u32, let NUM_PENDING_READS: u32, let NUM_SETTLED_READS: u32> {
        read_requests: [ScopedReadRequest; READ_REQUEST_LEN],
        read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN],
        pending_values: [ScopedNullifier; PENDING_VALUE_LEN],
        pending_read_hints: [PendingReadHint; NUM_PENDING_READS],
        leaf_preimages: [NullifierLeafPreimage; NUM_SETTLED_READS],
    }

    impl<let READ_REQUEST_LEN: u32, let PENDING_VALUE_LEN: u32, let NUM_PENDING_READS: u32, let NUM_SETTLED_READS: u32> TestBuilder<READ_REQUEST_LEN, PENDING_VALUE_LEN, NUM_PENDING_READS, NUM_SETTLED_READS> {
        pub fn new() -> TestBuilder<5, 4, 3, 2> {
            let contract_address = AztecAddress::from_field(123);

            // Create 4 nullifiers. 10 and 11 are settled. 12 and 13 are pending.
            let inner_nullifiers = [10, 11, 12, 13];
            let nullifiers = inner_nullifiers.map(|n| compute_siloed_nullifier(contract_address, n));

            // Create 5 read requests. 0 and 3 are reading settled nullifiers. 1, 2 and 4 are reading pending nullifiers.
            let read_requests = [
                ReadRequest { value: inner_nullifiers[1], counter: 11 }.scope(contract_address),// settled
                ReadRequest { value: inner_nullifiers[3], counter: 13 }.scope(contract_address),// pending
                ReadRequest { value: inner_nullifiers[2], counter: 39 }.scope(contract_address),// pending
                ReadRequest { value: inner_nullifiers[0], counter: 46 }.scope(contract_address),// settled
                ReadRequest { value: inner_nullifiers[3], counter: 78 }.scope(contract_address)// pending
            ];

            let pending_values = [
                Nullifier { value: inner_nullifiers[2], counter: 2, note_hash: 0 }.scope(contract_address),
                Nullifier { value: inner_nullifiers[3], counter: 8, note_hash: 0 }.scope(contract_address),
                ScopedNullifier::empty(),
                ScopedNullifier::empty()
            ];

            let pending_read_hints = [
                PendingReadHint { read_request_index: 1, pending_value_index: 1 },
                PendingReadHint { read_request_index: 2, pending_value_index: 0 },
                PendingReadHint { read_request_index: 4, pending_value_index: 1 }
            ];

            let leaf_preimages = [
                NullifierLeafPreimage { nullifier: nullifiers[0], next_nullifier: nullifiers[1], next_index: 1 },
                NullifierLeafPreimage { nullifier: nullifiers[1], next_nullifier: 0, next_index: 0 }
            ];

            let read_request_statuses = [
                ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
                ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
                ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
                ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 },
                ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 2 }
            ];

            TestBuilder { read_requests, read_request_statuses, pending_values, pending_read_hints, leaf_preimages }
        }

        fn build_tree<N>(self) -> NonEmptyMerkleTree<2, NULLIFIER_TREE_HEIGHT, NULLIFIER_TREE_HEIGHT - 1, 1> {
            NonEmptyMerkleTree::new(
                [self.leaf_preimages[0].hash(), self.leaf_preimages[1].hash()],
                [0; NULLIFIER_TREE_HEIGHT],
                [0; NULLIFIER_TREE_HEIGHT - 1],
                [0; 1]
            )
        }

        fn get_settled_read_hints(self) -> ([NullifierSettledReadHint; 2], Field) {
            let tree = self.build_tree();
            let hints = [
                NullifierSettledReadHint {
                    read_request_index: 0,
                    membership_witness: MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) },
                    leaf_preimage: self.leaf_preimages[1]
                },
                NullifierSettledReadHint {
                    read_request_index: 3,
                    membership_witness: MembershipWitness { leaf_index: 0, sibling_path: tree.get_sibling_path(0) },
                    leaf_preimage: self.leaf_preimages[0]
                }
            ];
            let tree_root = tree.get_root();
            (hints, tree_root)
        }

        pub fn get_unverified_read_requests(self) -> [ScopedReadRequest; READ_REQUEST_LEN] {
            get_unverified_read_requests(self.read_requests, self.read_request_statuses)
        }

        pub fn verify(self) {
            let (settled_hints, tree_root) = self.get_settled_read_hints();
            let unverified_read_requests = self.get_unverified_read_requests();
            verify_reset_read_requests(
                self.read_requests,
                self.pending_values,
                self.read_request_statuses,
                self.pending_read_hints,
                settled_hints,
                tree_root,
                unverified_read_requests
            );
        }
    }

    #[test]
    fn verify_reset_nullifier_read_requests_clears_all_succeeds() {
        let builder = TestBuilder::new();

        let unverified_read_requests = builder.get_unverified_read_requests();
        assert(is_empty_array(unverified_read_requests));

        builder.verify();
    }

    #[test]
    fn verify_reset_nullifier_read_requests_clears_partial_succeeds() {
        let mut builder = TestBuilder::new();

        builder.read_request_statuses[2] = ReadRequestStatus::empty();
        builder.read_request_statuses[4] = ReadRequestStatus::empty();

        let read_requests = builder.read_requests;
        let unverified_read_requests = builder.get_unverified_read_requests();
        assert_array_eq(
            unverified_read_requests,
            [read_requests[2], read_requests[4]]
        );

        builder.verify();
    }

    #[test(should_fail_with="Value of the nullifier does not match read request")]
    fn verify_reset_nullifier_read_requests_wrong_hinted_value() {
        let mut builder = TestBuilder::new();

        // Tweak the value to be something different.
        builder.pending_values[0].nullifier.value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Contract address of the nullifier does not match read request")]
    fn verify_reset_nullifier_read_requests_different_contract_addresses_fails() {
        let mut builder = TestBuilder::new();

        builder.pending_values[0].contract_address.inner += 1;

        builder.verify();
    }

    #[test(should_fail_with="Read request counter must be greater than the counter of the nullifier")]
    fn verify_reset_nullifier_read_requests_invalid_counter() {
        let mut builder = TestBuilder::new();

        let hint = builder.pending_read_hints[0];
        let pending_read = builder.read_requests[hint.read_request_index];
        // Tweak the counter of the value to be greater than the read request.
        builder.pending_values[hint.pending_value_index].nullifier.counter = pending_read.counter() + 1;

        builder.verify();
    }

    #[test(should_fail_with="Value of the nullifier leaf does not match read request")]
    fn verify_reset_nullifier_read_requests_invalid_leaf() {
        let mut builder = TestBuilder::new();

        let (settled_read_hints, _) = builder.get_settled_read_hints();
        let hint = settled_read_hints[0];
        // Tweak the value of the first settled read to be something different.
        builder.read_requests[hint.read_request_index].read_request.value += 1;

        builder.verify();
    }
}

