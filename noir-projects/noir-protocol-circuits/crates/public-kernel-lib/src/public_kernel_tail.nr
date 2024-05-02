use crate::common;
use dep::reset_kernel_lib::{
    NullifierReadRequestHints, NullifierNonExistentReadRequestHints, PublicDataReadRequestHints,
    PublicValidationRequestProcessor, PublicDataHint
};
use dep::types::{
    abis::{
    accumulated_data::CombinedAccumulatedData, kernel_circuit_public_inputs::KernelCircuitPublicInputs,
    kernel_data::PublicKernelData
},
    constants::MAX_PUBLIC_DATA_HINTS,
    merkle_tree::{conditionally_assert_check_membership, MembershipWitness},
    partial_state_reference::PartialStateReference, utils::{arrays::array_length}
};

struct PublicKernelTailCircuitPrivateInputs {
    previous_kernel: PublicKernelData,
    nullifier_read_request_hints: NullifierReadRequestHints,
    nullifier_non_existent_read_request_hints: NullifierNonExistentReadRequestHints,
    public_data_hints: [PublicDataHint; MAX_PUBLIC_DATA_HINTS],
    public_data_read_request_hints: PublicDataReadRequestHints,
    start_state: PartialStateReference,
}

impl PublicKernelTailCircuitPrivateInputs {
    fn validate_inputs(self) {
        let previous_public_inputs = self.previous_kernel.public_inputs;
        assert(previous_public_inputs.needs_setup() == false, "Previous kernel needs setup");
        assert(previous_public_inputs.needs_app_logic() == false, "Previous kernel needs app logic");
        assert(previous_public_inputs.needs_teardown() == false, "Previous kernel needs teardown");
        assert_eq(
            array_length(previous_public_inputs.end.public_call_stack), 0, "Public call stack must be empty when executing the tail circuit"
        );
        assert_eq(
            array_length(previous_public_inputs.end_non_revertible.public_call_stack), 0, "Public call stack must be empty when executing the tail circuit"
        );
    }

    fn validate_public_data_hints(self) {
        let public_data_hints = self.public_data_hints;
        let public_data_tree_root = self.start_state.public_data_tree.root;
        for i in 0..public_data_hints.len() {
            let hint = public_data_hints[i];
            // We only need to check leaf_slot to decide if a (non-)membership check is required.
            // It will fail if a PublicDataHint with 0 leaf_slot is used to verify a non-empty public read or write.
            if hint.leaf_slot != 0 {
                let exists_in_tree = hint.leaf_slot == hint.leaf_preimage.slot;
                if exists_in_tree {
                    assert(
                        hint.value == hint.leaf_preimage.value, "Hinted public data value does not match the value in leaf preimage"
                    );
                } else {
                    assert(hint.value == 0, "Value must be 0 for non-existent public data");
                }

                conditionally_assert_check_membership(
                    hint.leaf_slot,
                    exists_in_tree,
                    hint.leaf_preimage,
                    MembershipWitness { leaf_index: hint.membership_witness.leaf_index, sibling_path: hint.membership_witness.sibling_path },
                    public_data_tree_root
                );
            }
        }
    }

    fn propagate_accumulated_data(self) -> CombinedAccumulatedData {
        let previous_public_inputs = self.previous_kernel.public_inputs;
        // TODO: Sort the combined data.
        CombinedAccumulatedData::combine(
            previous_public_inputs.end_non_revertible,
            previous_public_inputs.end
        )
    }

    pub fn public_kernel_tail(self) -> KernelCircuitPublicInputs {
        self.validate_inputs();

        self.validate_public_data_hints();

        let previous_public_inputs = self.previous_kernel.public_inputs;
        let request_processor = PublicValidationRequestProcessor::new(
            previous_public_inputs,
            self.nullifier_read_request_hints,
            self.nullifier_non_existent_read_request_hints,
            self.start_state.nullifier_tree.root,
            self.public_data_read_request_hints,
            self.public_data_hints
        );
        request_processor.validate();

        let end = self.propagate_accumulated_data();

        KernelCircuitPublicInputs {
            aggregation_object: previous_public_inputs.aggregation_object,
            rollup_validation_requests: previous_public_inputs.validation_requests.for_rollup,
            end,
            constants: previous_public_inputs.constants,
            start_state: self.start_state,
            revert_code: previous_public_inputs.revert_code
        }
    }
}

mod tests {
    use crate::public_kernel_tail::PublicKernelTailCircuitPrivateInputs;
    use dep::reset_kernel_lib::{
        tests::{
        nullifier_non_existent_read_request_hints_builder::NullifierNonExistentReadRequestHintsBuilder,
        nullifier_read_request_hints_builder::NullifierReadRequestHintsBuilder,
        public_data_read_request_hints_builder::PublicDataReadRequestHintsBuilder
    },
        PublicDataHint, reset::read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus}
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::KernelCircuitPublicInputs, kernel_data::PublicKernelData,
        nullifier_leaf_preimage::NullifierLeafPreimage, membership_witness::PublicDataMembershipWitness
    },
        constants::{
        MAX_NEW_NULLIFIERS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_PUBLIC_DATA_HINTS,
        MAX_PUBLIC_DATA_READS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT,
        NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT, PUBLIC_DATA_SUBTREE_HEIGHT,
        PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, PUBLIC_DATA_TREE_HEIGHT, MAX_ENCRYPTED_LOGS_PER_TX,
        MAX_UNENCRYPTED_LOGS_PER_TX,
    },
        hash::{silo_nullifier, sha256_to_field},
        public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage,
        tests::{fixture_builder::FixtureBuilder, merkle_tree_utils::NonEmptyMerkleTree},
        partial_state_reference::PartialStateReference, utils::arrays::array_merge
    };

    fn build_nullifier_tree<N>() -> NonEmptyMerkleTree<MAX_NEW_NULLIFIERS_PER_TX, NULLIFIER_TREE_HEIGHT, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT> {
        let mut pre_existing_nullifiers = [NullifierLeafPreimage::empty(); MAX_NEW_NULLIFIERS_PER_TX];
        pre_existing_nullifiers[0] = NullifierLeafPreimage { nullifier: 0, next_nullifier: 100, next_index: 1 };
        pre_existing_nullifiers[1] = NullifierLeafPreimage { nullifier: 100, next_nullifier: 0, next_index: 0 };
        NonEmptyMerkleTree::new(
            pre_existing_nullifiers.map(|preimage: NullifierLeafPreimage| preimage.hash()),
            [0; NULLIFIER_TREE_HEIGHT],
            [0; NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT],
            [0; NULLIFIER_SUBTREE_HEIGHT]
        )
    }

    fn get_settled_public_data_leaves() -> [PublicDataTreeLeafPreimage; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] {
        let mut settled_public_data_leaves = [PublicDataTreeLeafPreimage::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX];
        settled_public_data_leaves[0] = PublicDataTreeLeafPreimage { slot: 22, value: 200, next_slot: 33, next_index: 1 };
        settled_public_data_leaves[1] = PublicDataTreeLeafPreimage { slot: 33, value: 300, next_slot: 0, next_index: 0 };
        settled_public_data_leaves[2] = PublicDataTreeLeafPreimage { slot: 11, value: 100, next_slot: 22, next_index: 0 };
        settled_public_data_leaves
    }

    fn build_public_data_tree<N>() -> NonEmptyMerkleTree<MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PUBLIC_DATA_TREE_HEIGHT, PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, PUBLIC_DATA_SUBTREE_HEIGHT> {
        let settled_public_data_leaves = get_settled_public_data_leaves();
        NonEmptyMerkleTree::new(
            settled_public_data_leaves.map(|preimage: PublicDataTreeLeafPreimage| preimage.hash()),
            [0; PUBLIC_DATA_TREE_HEIGHT],
            [0; PUBLIC_DATA_TREE_HEIGHT - PUBLIC_DATA_SUBTREE_HEIGHT],
            [0; PUBLIC_DATA_SUBTREE_HEIGHT]
        )
    }

    struct PublicKernelTailCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        previous_revertible: FixtureBuilder,
        nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder,
        nullifier_non_existent_read_request_hints_builder: NullifierNonExistentReadRequestHintsBuilder,
        public_data_read_request_hints_builder: PublicDataReadRequestHintsBuilder,
        public_data_hints: BoundedVec<PublicDataHint, MAX_PUBLIC_DATA_HINTS>,
        public_data_tree: NonEmptyMerkleTree<MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PUBLIC_DATA_TREE_HEIGHT, PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, PUBLIC_DATA_SUBTREE_HEIGHT>,
        start_state: PartialStateReference,
    }

    impl PublicKernelTailCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new();
            let previous_revertible = FixtureBuilder::new();
            let nullifier_non_existent_read_request_hints_builder = NullifierNonExistentReadRequestHintsBuilder::new();

            let mut builder = PublicKernelTailCircuitPrivateInputsBuilder {
                previous_kernel,
                previous_revertible,
                nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder::new(MAX_NULLIFIER_READ_REQUESTS_PER_TX),
                nullifier_non_existent_read_request_hints_builder,
                public_data_read_request_hints_builder: PublicDataReadRequestHintsBuilder::new(MAX_PUBLIC_DATA_READS_PER_TX),
                public_data_hints: BoundedVec::new(),
                public_data_tree: NonEmptyMerkleTree::empty(),
                start_state: PartialStateReference::empty()
            };
            builder.set_nullifiers_for_non_existent_read_request_hints();
            builder
        }

        pub fn with_nullifier_tree(&mut self) -> Self {
            let nullifier_tree = build_nullifier_tree();
            self.nullifier_non_existent_read_request_hints_builder.set_nullifier_tree(nullifier_tree);
            let tree_root = nullifier_tree.get_root();
            self.start_state.nullifier_tree.root = tree_root;
            self.previous_kernel.historical_header.state.partial.nullifier_tree.root = tree_root;
            *self
        }

        pub fn with_public_data_tree(&mut self) -> Self {
            let public_data_tree = build_public_data_tree();
            self.public_data_tree = public_data_tree;
            self.start_state.public_data_tree.root = public_data_tree.get_root();
            *self
        }

        pub fn add_nullifier(&mut self, unsiloed_nullifier: Field) {
            self.previous_kernel.add_nullifier(unsiloed_nullifier);
            self.sync_counters();
            self.set_nullifiers_for_non_existent_read_request_hints();
        }

        pub fn append_nullifiers_revertible(&mut self, num_nullifiers: u64) {
            self.previous_revertible.append_new_nullifiers(num_nullifiers);
            self.sync_counters();
            self.set_nullifiers_for_non_existent_read_request_hints();
        }

        pub fn append_nullifiers_non_revertible(&mut self, num_nullifiers: u64) {
            self.previous_kernel.append_new_nullifiers(num_nullifiers);
            self.sync_counters();
            self.set_nullifiers_for_non_existent_read_request_hints();
        }

        fn set_nullifiers_for_non_existent_read_request_hints(&mut self) {
            let nullifiers = array_merge(
                self.previous_kernel.new_nullifiers.storage,
                self.previous_revertible.new_nullifiers.storage
            );
            self.nullifier_non_existent_read_request_hints_builder.set_nullifiers(nullifiers);
        }

        pub fn add_pending_revertible_nullifier_read_request(&mut self, nullifier_index: u64) {
            let read_request_index = self.previous_kernel.add_read_request_for_pending_nullifier(nullifier_index);
            self.sync_counters();
            let hint_index = self.nullifier_read_request_hints_builder.pending_read_hints.len();
            let pending_value_index = nullifier_index + self.previous_kernel.new_nullifiers.len();
            let hint = PendingReadHint { read_request_index, pending_value_index };
            self.nullifier_read_request_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn add_pending_non_revertible_nullifier_read_request(&mut self, nullifier_index: u64) {
            let read_request_index = self.previous_kernel.add_read_request_for_pending_nullifier(nullifier_index);
            self.sync_counters();
            let hint_index = self.nullifier_read_request_hints_builder.pending_read_hints.len();
            let hint = PendingReadHint { read_request_index, pending_value_index: nullifier_index };
            self.nullifier_read_request_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn read_non_existent_nullifier(&mut self, unsiloed_nullifier: Field) {
            self.previous_kernel.add_non_existent_read_request_for_nullifier(unsiloed_nullifier);
            self.sync_counters();
            let siloed_nullifier = silo_nullifier(
                self.previous_kernel.storage_contract_address,
                unsiloed_nullifier
            );
            self.nullifier_non_existent_read_request_hints_builder.add_value_read(siloed_nullifier);
        }

        pub fn add_public_data_hint_for_settled_public_data(&mut self, leaf_index: u64) {
            let leaf_preimage = get_settled_public_data_leaves()[leaf_index];
            let membership_witness = PublicDataMembershipWitness { leaf_index: leaf_index as Field, sibling_path: self.public_data_tree.get_sibling_path(leaf_index) };
            let hint = PublicDataHint {
                leaf_slot: leaf_preimage.slot,
                value: leaf_preimage.value,
                override_counter: 0,
                membership_witness,
                leaf_preimage
            };
            self.public_data_hints.push(hint);
        }

        pub fn add_public_data_hint_for_non_existent_public_data(&mut self, leaf_slot: Field, low_leaf_index: u64) {
            let leaf_preimage = get_settled_public_data_leaves()[low_leaf_index];
            let membership_witness = PublicDataMembershipWitness {
                leaf_index: low_leaf_index as Field,
                sibling_path: self.public_data_tree.get_sibling_path(low_leaf_index)
            };
            let hint = PublicDataHint { leaf_slot, value: 0, override_counter: 0, membership_witness, leaf_preimage };
            self.public_data_hints.push(hint);
        }

        pub fn add_pending_public_data_read_request(&mut self, public_date_update_request_index: u64) {
            let read_request_index = self.previous_kernel.add_read_request_for_pending_public_data(public_date_update_request_index);
            let hint_index = self.public_data_read_request_hints_builder.pending_read_hints.len();
            let hint = PendingReadHint { read_request_index, pending_value_index: public_date_update_request_index };
            self.public_data_read_request_hints_builder.pending_read_hints.push(hint);
            self.public_data_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        fn sync_counters(&mut self) {
            let counter_non_revertible = self.previous_kernel.counter;
            let counter_revertible = self.previous_revertible.counter;
            if counter_non_revertible > counter_revertible {
                self.previous_revertible.counter = counter_non_revertible;
            } else {
                self.previous_kernel.counter = counter_revertible;
            }
        }

        pub fn execute(&mut self) -> KernelCircuitPublicInputs {
            let mut previous_kernel = self.previous_kernel.to_public_kernel_data(false);
            previous_kernel.public_inputs.end = self.previous_revertible.to_public_accumulated_data();

            let kernel = PublicKernelTailCircuitPrivateInputs {
                previous_kernel,
                nullifier_read_request_hints: self.nullifier_read_request_hints_builder.to_hints(),
                nullifier_non_existent_read_request_hints: self.nullifier_non_existent_read_request_hints_builder.to_hints(),
                public_data_hints: self.public_data_hints.storage,
                public_data_read_request_hints: self.public_data_read_request_hints_builder.to_hints(),
                start_state: self.start_state
            };

            kernel.public_kernel_tail()
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }
    }

    #[test]
    unconstrained fn public_kernel_circuit_tail_succeeds() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();
        builder.succeeded();
        // TODO: Check the values in public inputs.
    }

    #[test]
    fn logs_are_handled_as_expected() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();
        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = 80;
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = 956;
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_revertible.set_encrypted_logs(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_revertible.set_unencrypted_logs(
            prev_unencrypted_logs_hash,
            prev_unencrypted_log_preimages_length
        );
        // Logs for the current call stack.
        let unencrypted_logs_hash = 26;
        let unencrypted_log_preimages_length = 50;
        builder.previous_revertible.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

        let public_inputs = builder.execute();

        assert_eq(public_inputs.end.encrypted_log_preimages_length, prev_encrypted_log_preimages_length);
        assert_eq(
            public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length
        );

        let hash_bytes: [u8; MAX_ENCRYPTED_LOGS_PER_TX * 32] = prev_encrypted_logs_hash
            .to_be_bytes(32)
            .append(&[0; MAX_ENCRYPTED_LOGS_PER_TX * 32 - 32])
            .as_array();
        let expected_encrypted_logs_hash = sha256_to_field(hash_bytes);
        assert_eq(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);

        let hash_bytes: [u8; MAX_UNENCRYPTED_LOGS_PER_TX * 32] = prev_unencrypted_logs_hash
            .to_be_bytes(32)
            .append(unencrypted_logs_hash.to_be_bytes(32))
            .append(&[0; MAX_UNENCRYPTED_LOGS_PER_TX * 32 - 64])
            .as_array();
        let expected_unencrypted_logs_hash = sha256_to_field(hash_bytes);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
    }

    #[test]
    unconstrained fn one_pending_nullifier_read_request() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers_revertible(3);
        builder.add_pending_revertible_nullifier_read_request(1);
        builder.succeeded();
    }

    #[test]
    unconstrained fn two_pending_nullifier_read_requests() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers_revertible(3);
        builder.add_pending_revertible_nullifier_read_request(1);
        builder.add_pending_revertible_nullifier_read_request(0);

        builder.succeeded();
    }

    #[test]
    unconstrained fn one_pending_nullifier_read_request_non_revertible() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers_non_revertible(3);
        builder.add_pending_non_revertible_nullifier_read_request(1);
        builder.succeeded();
    }

    #[test(should_fail_with="Value of the nullifier does not match read request")]
    unconstrained fn pending_nullifier_read_request_wrong_hint_fails() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers_revertible(3);
        builder.add_pending_revertible_nullifier_read_request(1);
        let mut hint = builder.nullifier_read_request_hints_builder.pending_read_hints.pop();
        hint.pending_value_index -= 1;
        builder.nullifier_read_request_hints_builder.pending_read_hints.push(hint);

        builder.failed();
    }

    #[test(should_fail_with="Read request counter must be greater than the counter of the nullifier")]
    unconstrained fn pending_nullifier_read_request_reads_before_value_fails() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers_revertible(3);
        builder.add_pending_revertible_nullifier_read_request(1);
        let nullifier_being_read = builder.previous_revertible.new_nullifiers.get(1);
        let mut read_request = builder.previous_kernel.nullifier_read_requests.pop();
        read_request.counter = nullifier_being_read.counter - 1;
        builder.previous_kernel.nullifier_read_requests.push(read_request);

        builder.failed();
    }

    // TODO: Add tests for reading (non-existent) settled values.

    #[test]
    unconstrained fn nullifier_non_existent_read_request() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new().with_nullifier_tree();

        builder.add_nullifier(3);
        builder.add_nullifier(1);
        builder.add_nullifier(9);

        builder.read_non_existent_nullifier(8);

        builder.succeeded();
    }

    #[test(should_fail_with="Value exists in pending set")]
    unconstrained fn nullifier_non_existent_read_request_failed_read_exist() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new().with_nullifier_tree();

        builder.add_nullifier(3);
        builder.add_nullifier(1);
        builder.add_nullifier(9);

        builder.read_non_existent_nullifier(1);

        builder.failed();
    }

    #[test]
    unconstrained fn validate_public_data_hints() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new().with_public_data_tree();

        builder.add_public_data_hint_for_settled_public_data(1);
        builder.add_public_data_hint_for_settled_public_data(0);
        builder.add_public_data_hint_for_settled_public_data(2);

        builder.succeeded();
    }

    #[test(should_fail_with="Hinted public data value does not match the value in leaf preimage")]
    unconstrained fn validate_public_data_hints_failed_mismatch_value() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new().with_public_data_tree();

        builder.add_public_data_hint_for_settled_public_data(1);

        let mut hint = builder.public_data_hints.pop();
        hint.value += 1;
        builder.public_data_hints.push(hint);

        builder.failed();
    }

    #[test]
    unconstrained fn validate_public_data_hints_uninitialized_value() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new().with_public_data_tree();

        builder.add_public_data_hint_for_non_existent_public_data(25, 0);

        builder.succeeded();
    }

    #[test(should_fail_with="Value must be 0 for non-existent public data")]
    unconstrained fn validate_public_data_hints_failed_non_zero_uninitialized_value() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new().with_public_data_tree();

        builder.add_public_data_hint_for_non_existent_public_data(25, 0);

        let mut hint = builder.public_data_hints.pop();
        hint.value = 1;
        builder.public_data_hints.push(hint);

        builder.failed();
    }

    #[test]
    unconstrained fn pending_public_data_read_requests() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.append_public_data_update_requests(3);

        builder.add_pending_public_data_read_request(1);
        builder.add_pending_public_data_read_request(0);
        builder.add_pending_public_data_read_request(2);

        builder.succeeded();
    }

    #[test(should_fail_with="Hinted slot of data write does not match read request")]
    unconstrained fn pending_public_data_read_requests_failed_wrong_write_index() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.append_public_data_update_requests(2);

        builder.add_pending_public_data_read_request(1);

        let mut hint = builder.public_data_read_request_hints_builder.pending_read_hints.pop();
        hint.pending_value_index += 1;
        builder.public_data_read_request_hints_builder.pending_read_hints.push(hint);

        builder.failed();
    }

    #[test(should_fail_with="Hinted value of data write does not match read request")]
    unconstrained fn pending_public_data_read_requests_failed_wrong_write_value() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.append_public_data_update_requests(1);

        builder.add_pending_public_data_read_request(0);

        let mut public_data_write = builder.previous_kernel.public_data_update_requests.pop();
        public_data_write.new_value += 1;
        builder.previous_kernel.public_data_update_requests.push(public_data_write);

        builder.failed();
    }
}
