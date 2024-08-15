// This will be moved to a separate Read Request Reset Circuit.
use dep::types::{
    abis::{read_request::ScopedReadRequest, side_effect::Readable},
    merkle_tree::{assert_check_membership, LeafPreimage, MembershipWitness}, traits::Empty
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
    hint_index: u32,
}

impl Empty for ReadRequestStatus {
    fn empty() -> Self {
        ReadRequestStatus { state: ReadRequestState.NADA, hint_index: 0 }
    }
}

trait ReadValueHint {
    fn read_request_index(self) -> u32;
}

struct PendingReadHint {
    read_request_index: u32,
    pending_value_index: u32,
}

impl ReadValueHint for PendingReadHint {
    fn read_request_index(self) -> u32 {
        self.read_request_index
    }
}

impl PendingReadHint {
    pub fn nada(read_request_len: u32) -> Self {
        PendingReadHint { read_request_index: read_request_len, pending_value_index: 0 }
    }
}

trait SettledReadHint<let TREE_HEIGHT: u32, LEAF_PREIMAGE> where LEAF_PREIMAGE: LeafPreimage {
    fn membership_witness(self) -> MembershipWitness<TREE_HEIGHT>;
    fn leaf_preimage(self) -> LEAF_PREIMAGE;
    fn nada(read_request_len: u32) -> Self;
}

// Validate the values being read were emitted in the same transaction before the read request were made.
// More info here:
// - https://discourse.aztec.network/t/to-read-or-not-to-read/178
// - https://discourse.aztec.network/t/spending-notes-which-havent-yet-been-inserted/180
fn validate_pending_read_requests<let READ_REQUEST_LEN: u32, T, let PENDING_VALUE_LEN: u32, let NUM_PENDING_READS: u32>(
    read_requests: [ScopedReadRequest; READ_REQUEST_LEN],
    pending_values: [T; PENDING_VALUE_LEN],
    hints: [PendingReadHint; NUM_PENDING_READS]
) where T: Readable {
    for i in 0..NUM_PENDING_READS {
        let hint = hints[i];
        let read_request_index = hint.read_request_index;
        if read_request_index != READ_REQUEST_LEN {
            let read_request = read_requests[read_request_index];
            let pending_value = pending_values[hint.pending_value_index];
            pending_value.assert_match_read_request(read_request);
        }
    }
}

// Validate read requests against the historical tree root, for reading settled notes.
// Use their membership witnesses to do so.
fn validate_settled_read_requests<let READ_REQUEST_LEN: u32, let NUM_SETTLED_READS: u32, H, TREE_HEIGHT, LEAF_PREIMAGE>(
    read_requests: [ScopedReadRequest; READ_REQUEST_LEN],
    hints: [H; NUM_SETTLED_READS],
    tree_root: Field
) where 
    H: SettledReadHint<TREE_HEIGHT, LEAF_PREIMAGE> + ReadValueHint, 
    LEAF_PREIMAGE: LeafPreimage + Readable {
    for i in 0..NUM_SETTLED_READS {
        let hint = hints[i];
        let read_request_index = hint.read_request_index();
        if read_request_index != READ_REQUEST_LEN {
            let read_request = read_requests[read_request_index];
            let leaf_preimage = hint.leaf_preimage();
            leaf_preimage.assert_match_read_request(read_request);

            let leaf = leaf_preimage.as_leaf();
            let witness = hint.membership_witness();
            assert_check_membership(leaf, witness.leaf_index, witness.sibling_path, tree_root);
        }
    }
}

fn verify_propagated_read_requests<let READ_REQUEST_LEN: u32, T, let NUM_PENDING_READS: u32, S, let NUM_SETTLED_READS: u32>(
    read_requests: [ScopedReadRequest; READ_REQUEST_LEN],
    read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN],
    pending_read_hints: [T; NUM_PENDING_READS],
    settled_read_hints: [S; NUM_SETTLED_READS],
    propagated_read_requests: [ScopedReadRequest; READ_REQUEST_LEN]
) where T: ReadValueHint, S: ReadValueHint {
    let mut num_propagated = 0;
    for i in 0..READ_REQUEST_LEN {
        let read_request = read_requests[i];
        let status = read_request_statuses[i];
        if status.state == ReadRequestState.PENDING {
            assert_eq(
                pending_read_hints[status.hint_index].read_request_index(), i, "Hinted pending read request does not match status"
            );
        } else if status.state == ReadRequestState.SETTLED {
            assert_eq(
                settled_read_hints[status.hint_index].read_request_index(), i, "Hinted settled read request does not match status"
            );
        } else {
            assert_eq(
                propagated_read_requests[num_propagated], read_request, "mismatch propagated read request"
            );
            num_propagated += 1;
        }
    }
}

pub fn verify_reset_read_requests<
    let READ_REQUEST_LEN: u32,
    P,
    let PENDING_VALUE_LEN: u32,
    let NUM_PENDING_READS: u32,
    let NUM_SETTLED_READS: u32,
    H,
    TREE_HEIGHT,
    LEAF_PREIMAGE
>(
    read_requests: [ScopedReadRequest; READ_REQUEST_LEN],
    pending_values: [P; PENDING_VALUE_LEN],
    read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN],
    pending_read_hints: [PendingReadHint; NUM_PENDING_READS],
    settled_read_hints: [H; NUM_SETTLED_READS],
    tree_root: Field,
    propagated_read_requests: [ScopedReadRequest; READ_REQUEST_LEN]
) where 
    P: Readable, 
    H: SettledReadHint<TREE_HEIGHT, LEAF_PREIMAGE> + ReadValueHint,
    LEAF_PREIMAGE: LeafPreimage + Readable {
    validate_pending_read_requests(read_requests, pending_values, pending_read_hints);

    validate_settled_read_requests(read_requests, settled_read_hints, tree_root);

    verify_propagated_read_requests(
        read_requests,
        read_request_statuses,
        pending_read_hints,
        settled_read_hints,
        propagated_read_requests
    );
}

unconstrained pub fn get_unverified_read_requests<let READ_REQUEST_LEN: u32>(
    read_requests: [ScopedReadRequest; READ_REQUEST_LEN],
    read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN]
) -> [ScopedReadRequest; READ_REQUEST_LEN] {
    let mut propagated_read_requests = BoundedVec::new();
    for i in 0..READ_REQUEST_LEN {
        let read_request = read_requests[i];
        if read_request_statuses[i].state == ReadRequestState.NADA {
            propagated_read_requests.push(read_request);
        }
    }
    propagated_read_requests.storage
}

mod tests {
    use crate::reset::read_request::{
        PendingReadHint, ReadRequestState, ReadRequestStatus, ReadValueHint, SettledReadHint,
        get_unverified_read_requests, validate_pending_read_requests, validate_settled_read_requests,
        verify_reset_read_requests
    };
    use dep::types::{
        address::AztecAddress,
        abis::{read_request::{ReadRequest, ScopedReadRequest}, side_effect::Readable},
        merkle_tree::{LeafPreimage, MembershipWitness},
        tests::{merkle_tree_utils::NonEmptyMerkleTree, utils::assert_array_eq},
        traits::{Empty, is_empty_array}
    };

    fn silo_test_value(value: Field) -> Field {
        value + 9999
    }

    struct TestValue {
        value: Field,
        counter: u32,
    }

    impl Readable for TestValue {
        fn assert_match_read_request(self, read_request: ScopedReadRequest) {
            let siloed_value = silo_test_value(read_request.value());
            assert_eq(self.value, siloed_value, "Hinted test value does not match");
        }
    }

    struct TestLeafPreimage {
        value: Field,
    }

    impl LeafPreimage for TestLeafPreimage {
        fn get_key(self) -> Field {
            self.value
        }

        fn as_leaf(self) -> Field {
            silo_test_value(self.value)
        }
    }

    impl Empty for TestLeafPreimage {
        fn empty() -> Self {
            TestLeafPreimage {
                value: 0,
            }
        }
    }

    impl Readable for TestLeafPreimage {
        fn assert_match_read_request(self, read_request: ScopedReadRequest) {
            let siloed_value = silo_test_value(read_request.value());
            assert_eq(siloed_value, self.value, "Provided leaf preimage is not for target value");
        }
    }

    struct TestSettledReadHint {
        read_request_index: u32,
        membership_witness: MembershipWitness<3>,
        leaf_preimage: TestLeafPreimage,
    }

    impl ReadValueHint for TestSettledReadHint {
        fn read_request_index(self) -> u32 {
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

        fn nada(read_request_len: u32) -> Self {
            TestSettledReadHint {
                read_request_index: read_request_len,
                membership_witness: MembershipWitness::empty(),
                leaf_preimage: TestLeafPreimage::empty()
            }
        }
    }

    struct TestBuilder<let READ_REQUEST_LEN: u32, let PENDING_VALUE_LEN: u32, let NUM_PENDING_READS: u32, let NUM_SETTLED_READS: u32> {
        read_requests: [ScopedReadRequest; READ_REQUEST_LEN],
        read_request_statuses: [ReadRequestStatus; READ_REQUEST_LEN],
        pending_values: [TestValue; PENDING_VALUE_LEN],
        pending_read_hints: [PendingReadHint; NUM_PENDING_READS],
        leaf_preimages: [TestLeafPreimage; NUM_SETTLED_READS],
    }

    impl<let READ_REQUEST_LEN: u32, let PENDING_VALUE_LEN: u32, let NUM_PENDING_READS: u32, let NUM_SETTLED_READS: u32> TestBuilder<READ_REQUEST_LEN, PENDING_VALUE_LEN, NUM_PENDING_READS, NUM_SETTLED_READS> {
        pub fn new() -> TestBuilder<4, 2, 2, 2> {
            let contract_address = AztecAddress::from_field(123);

            // Create 4 values. 10 and 11 are settled. 12 and 13 are pending.
            let values = [10, 11, 12, 13];
            let siloed_values = values.map(|n| silo_test_value(n));

            // Create 4 read requests. 0 and 3 are reading settled values. 1 and 2 are reading pending values.
            let read_requests = [
                ReadRequest { value: values[1], counter: 11 }.scope(contract_address),// settled
                ReadRequest { value: values[3], counter: 13 }.scope(contract_address),// pending
                ReadRequest { value: values[2], counter: 39 }.scope(contract_address),// pending
                ReadRequest { value: values[0], counter: 46 }.scope(contract_address)// settled
            ];

            let pending_values = [
                TestValue { value: siloed_values[2], counter: 2 },
                TestValue { value: siloed_values[3], counter: 8 }
            ];
            let pending_read_hints = [
                PendingReadHint { read_request_index: 1, pending_value_index: 1 },
                PendingReadHint { read_request_index: 2, pending_value_index: 0 }
            ];

            let leaf_preimages = [
                TestLeafPreimage { value: siloed_values[0] },
                TestLeafPreimage { value: siloed_values[1] }
            ];

            let read_request_statuses = [
                ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
                ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
                ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
                ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 }
            ];

            TestBuilder { read_requests, read_request_statuses, pending_values, pending_read_hints, leaf_preimages }
        }

        fn build_tree<N>(self) -> NonEmptyMerkleTree<2, 3, 2, 1> {
            NonEmptyMerkleTree::new(
                [self.leaf_preimages[0].as_leaf(), self.leaf_preimages[1].as_leaf()],
                [0; 3],
                [0; 2],
                [0; 1]
            )
        }

        pub fn get_settled_read_hints(self) -> ([TestSettledReadHint; 2], Field) {
            let tree = self.build_tree();
            let hints = [
                TestSettledReadHint {
                    read_request_index: 0,
                    membership_witness: MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) },
                    leaf_preimage: self.leaf_preimages[1]
                },
                TestSettledReadHint {
                    read_request_index: 3,
                    membership_witness: MembershipWitness { leaf_index: 0, sibling_path: tree.get_sibling_path(0) },
                    leaf_preimage: self.leaf_preimages[0]
                }
            ];
            let tree_root = tree.get_root();
            (hints, tree_root)
        }

        pub fn get_nada_pending_read_hint(self) -> PendingReadHint {
            PendingReadHint::nada(self.read_requests.len())
        }

        pub fn get_nada_settled_read_hint(self) -> TestSettledReadHint {
            TestSettledReadHint::nada(self.read_requests.len())
        }

        pub fn get_unverified_read_requests(self) -> [ScopedReadRequest; READ_REQUEST_LEN] {
            get_unverified_read_requests(self.read_requests, self.read_request_statuses)
        }

        pub fn validate_pending_read_requests(self) {
            validate_pending_read_requests(
                self.read_requests,
                self.pending_values,
                self.pending_read_hints
            );
        }

        pub fn validate_settled_read_requests(self) {
            let (settled_hints, tree_root) = self.get_settled_read_hints();
            validate_settled_read_requests(self.read_requests, settled_hints, tree_root);
        }

        pub fn validate_settled_read_requests_with_hints(
            self,
            settled_hints: [TestSettledReadHint; 2],
            tree_root: Field
        ) {
            validate_settled_read_requests(self.read_requests, settled_hints, tree_root);
        }

        pub fn verify_with_settled_hints(self, settled_hints: [TestSettledReadHint; 2]) {
            let tree = self.build_tree();
            verify_reset_read_requests(
                self.read_requests,
                self.pending_values,
                self.read_request_statuses,
                self.pending_read_hints,
                settled_hints,
                tree.get_root(),
                self.get_unverified_read_requests()
            );
        }

        pub fn verify(self) {
            let (settled_hints, tree_root) = self.get_settled_read_hints();
            verify_reset_read_requests(
                self.read_requests,
                self.pending_values,
                self.read_request_statuses,
                self.pending_read_hints,
                settled_hints,
                tree_root,
                self.get_unverified_read_requests()
            );
        }
    }

    /**
     * validate_pending_read_requests
     */

    #[test]
    fn validate_pending_read_requests_succeeds() {
        let builder = TestBuilder::new();
        builder.validate_pending_read_requests();
    }

    #[test]
    fn validate_pending_read_requests_partial_succeeds() {
        let mut builder = TestBuilder::new();

        builder.pending_read_hints[1] = builder.get_nada_pending_read_hint();

        builder.validate_pending_read_requests();
    }

    #[test(should_fail_with="Hinted test value does not match")]
    fn validate_pending_read_requests_wrong_hint_fails() {
        let mut builder = TestBuilder::new();

        builder.pending_read_hints[1].pending_value_index += 1;

        builder.validate_pending_read_requests();
    }

    #[test(should_fail_with="Hinted test value does not match")]
    fn validate_pending_read_requests_wrong_hint_after_nada_fails() {
        let mut builder = TestBuilder::new();

        builder.pending_read_hints[0] = builder.get_nada_pending_read_hint();
        builder.pending_read_hints[1].pending_value_index += 1;

        builder.validate_pending_read_requests();
    }

    /**
     * validate_settled_read_requests
     */

    #[test]
    fn validate_settled_read_requests_succeeds() {
        let builder = TestBuilder::new();
        builder.validate_settled_read_requests();
    }

    #[test]
    fn validate_settled_read_requests_partial_succeeds() {
        let mut builder = TestBuilder::new();

        let mut (settled_hints, tree_root) = builder.get_settled_read_hints();
        settled_hints[0] = builder.get_nada_settled_read_hint();

        builder.validate_settled_read_requests_with_hints(settled_hints, tree_root);
    }

    #[test(should_fail_with="membership check failed")]
    fn validate_settled_read_requests_wrong_leaf_index_fails() {
        let mut builder = TestBuilder::new();

        let mut (settled_hints, tree_root) = builder.get_settled_read_hints();
        settled_hints[0].membership_witness.leaf_index += 1;

        builder.validate_settled_read_requests_with_hints(settled_hints, tree_root);
    }

    #[test(should_fail_with="Provided leaf preimage is not for target value")]
    fn validate_settled_read_requests_wrong_preimage_value_fails() {
        let mut builder = TestBuilder::new();

        let mut (settled_hints, tree_root) = builder.get_settled_read_hints();
        settled_hints[0].leaf_preimage.value += 1;

        builder.validate_settled_read_requests_with_hints(settled_hints, tree_root);
    }

    #[test(should_fail_with="membership check failed")]
    fn validate_settled_read_requests_wrong_path_after_nada_fails() {
        let mut builder = TestBuilder::new();

        let mut (settled_hints, tree_root) = builder.get_settled_read_hints();
        settled_hints[0] = builder.get_nada_settled_read_hint();
        settled_hints[1].membership_witness.sibling_path[0] += 1;

        builder.validate_settled_read_requests_with_hints(settled_hints, tree_root);
    }

    /**
     * verify_reset_read_requests
     */

    #[test]
    fn verify_read_requests_clears_all_succeeds() {
        let builder = TestBuilder::new();

        let unverified_read_requests = builder.get_unverified_read_requests();
        assert(is_empty_array(unverified_read_requests));

        builder.verify();
    }

    #[test]
    fn verify_reset_read_requests_partial_succeeds() {
        let mut builder = TestBuilder::new();

        builder.read_request_statuses[0] = ReadRequestStatus::empty();
        builder.read_request_statuses[2] = ReadRequestStatus::empty();
        builder.pending_read_hints[1] = builder.get_nada_pending_read_hint();

        let mut (settled_hints, _) = builder.get_settled_read_hints();
        settled_hints[0] = builder.get_nada_settled_read_hint();

        let read_requests = builder.read_requests;
        let unverified_read_requests = builder.get_unverified_read_requests();
        assert_array_eq(
            unverified_read_requests,
            [read_requests[0], read_requests[2]]
        );

        builder.verify_with_settled_hints(settled_hints);
    }

    #[test(should_fail_with="Hinted pending read request does not match status")]
    fn verify_reset_read_requests_wrong_pending_read_status_fails() {
        let mut builder = TestBuilder::new();

        builder.read_request_statuses[0].state = ReadRequestState.PENDING;

        builder.verify();
    }

    #[test(should_fail_with="Hinted settled read request does not match status")]
    fn verify_reset_read_requests_wrong_settled_read_status_fails() {
        let mut builder = TestBuilder::new();

        builder.read_request_statuses[2].state = ReadRequestState.SETTLED;

        builder.verify();
    }
}

