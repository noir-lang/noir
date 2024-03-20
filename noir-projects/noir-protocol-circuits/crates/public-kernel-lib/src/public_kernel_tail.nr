use crate::common;
use dep::reset_kernel_lib::{
    NullifierReadRequestHints, NullifierNonExistentReadRequestHints, reset_non_existent_read_requests,
    reset_read_requests
};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, PublicKernelCircuitPublicInputsBuilder},
    kernel_data::PublicKernelData, side_effect::SideEffectLinkedToNoteHash
},
    constants::MAX_NEW_NULLIFIERS_PER_TX,
    utils::{arrays::{array_length, array_merge, array_concat, array_to_bounded_vec, assert_sorted_array}},
    hash::silo_nullifier, traits::is_empty
};
use dep::std::unsafe;

struct PublicKernelTailCircuitPrivateInputs {
    previous_kernel: PublicKernelData,
    nullifier_read_request_hints: NullifierReadRequestHints,
    nullifier_non_existent_read_request_hints: NullifierNonExistentReadRequestHints,
}

impl PublicKernelTailCircuitPrivateInputs {
    fn propagate_revert_code(self, public_inputs: &mut PublicKernelCircuitPublicInputsBuilder) {
        public_inputs.end_non_revertible.revert_code = self.previous_kernel.public_inputs.end_non_revertible.revert_code;
    }

    fn validate_inputs(self) {
        let previous_public_inputs = self.previous_kernel.public_inputs;
        assert(previous_public_inputs.needs_setup == false, "Previous kernel needs setup");
        assert(previous_public_inputs.needs_app_logic == false, "Previous kernel needs app logic");
        assert(previous_public_inputs.needs_teardown == false, "Previous kernel needs teardown");
        assert_eq(
            array_length(previous_public_inputs.end.public_call_stack), 0, "Public call stack must be empty when executing the tail circuit"
        );
        assert_eq(
            array_length(previous_public_inputs.end_non_revertible.public_call_stack), 0, "Public call stack must be empty when executing the tail circuit"
        );
    }

    fn validate_nullifier_read_requests(self, public_inputs: &mut PublicKernelCircuitPublicInputsBuilder) {
        let end_non_revertible = self.previous_kernel.public_inputs.end_non_revertible;
        let end = self.previous_kernel.public_inputs.end;

        let requests = self.previous_kernel.public_inputs.validation_requests.nullifier_read_requests;

        let pending_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX] = array_concat(end_non_revertible.new_nullifiers, end.new_nullifiers);

        let hints = self.nullifier_read_request_hints;

        let nullifier_tree_root = public_inputs.constants.historical_header.state.partial.nullifier_tree.root;

        let unverified_nullifier_read_requests = reset_read_requests(
            requests,
            pending_nullifiers,
            hints.read_request_statuses,
            hints.pending_read_hints,
            hints.settled_read_hints,
            nullifier_tree_root
        );

        assert(
            unverified_nullifier_read_requests.len() == 0, "All nullifier read requests must be verified"
        );
    }

    fn validate_nullifier_non_existent_read_requests(self, public_inputs: &mut PublicKernelCircuitPublicInputsBuilder) {
        let end_non_revertible = self.previous_kernel.public_inputs.end_non_revertible;
        let end = self.previous_kernel.public_inputs.end;

        // The values of the read requests here need to be siloed.
        // Notice that it's not the case for regular read requests, which can be run between two kernel iterations, and will to be verified against unsiloed pending values.
        let mut read_requests = self.previous_kernel.public_inputs.validation_requests.nullifier_non_existent_read_requests;
        for i in 0..read_requests.len() {
            let read_request = read_requests[i];
            if !is_empty(read_request) {
                read_requests[i].value = silo_nullifier(read_request.contract_address, read_request.value);
            }
        }

        let nullifier_tree_root = public_inputs.constants.historical_header.state.partial.nullifier_tree.root;

        let hints = self.nullifier_non_existent_read_request_hints;

        let pending_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX] = array_concat(end_non_revertible.new_nullifiers, end.new_nullifiers);
        assert_sorted_array(
            pending_nullifiers,
            hints.sorted_pending_values,
            hints.sorted_pending_value_index_hints,
            |a: SideEffectLinkedToNoteHash, b: SideEffectLinkedToNoteHash| a.value.lt(b.value)
        );
        let sorted_pending_nullifiers = array_to_bounded_vec(hints.sorted_pending_values);

        reset_non_existent_read_requests(
            read_requests,
            hints.non_membership_hints,
            nullifier_tree_root,
            sorted_pending_nullifiers,
            hints.next_pending_value_indices
        );
    }

    pub fn public_kernel_tail(self) -> PublicKernelCircuitPublicInputs {
        let mut public_inputs: PublicKernelCircuitPublicInputsBuilder = unsafe::zeroed();

        self.validate_inputs();

        self.propagate_revert_code(&mut public_inputs);

        common::initialize_emitted_end_values(self.previous_kernel, &mut public_inputs);

        self.validate_nullifier_read_requests(&mut public_inputs);

        self.validate_nullifier_non_existent_read_requests(&mut public_inputs);

        public_inputs.to_inner()
    }
}

mod tests {
    use crate::{public_kernel_tail::PublicKernelTailCircuitPrivateInputs};
    use dep::reset_kernel_lib::{
        tests::{
        nullifier_non_existent_read_request_hints_builder::NullifierNonExistentReadRequestHintsBuilder,
        nullifier_read_request_hints_builder::NullifierReadRequestHintsBuilder
    },
        read_request_reset::{PendingReadHint, ReadRequestState, ReadRequestStatus}
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, PublicKernelCircuitPublicInputsBuilder},
        kernel_data::PublicKernelData, nullifier_leaf_preimage::NullifierLeafPreimage
    },
        constants::{
        MAX_NEW_NULLIFIERS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT,
        NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT
    },
        hash::silo_nullifier,
        tests::{kernel_data_builder::PreviousKernelDataBuilder, merkle_tree_utils::NonEmptyMerkleTree},
        utils::arrays::array_concat
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

    struct PublicKernelTailCircuitPrivateInputsBuilder {
        previous_kernel: PreviousKernelDataBuilder,
        nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder,
        nullifier_non_existent_read_request_hints_builder: NullifierNonExistentReadRequestHintsBuilder,
    }

    impl PublicKernelTailCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = PreviousKernelDataBuilder::new(true);
            let mut nullifier_non_existent_read_request_hints_builder = NullifierNonExistentReadRequestHintsBuilder::new();

            let mut builder = PublicKernelTailCircuitPrivateInputsBuilder {
                previous_kernel,
                nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder::new(MAX_NULLIFIER_READ_REQUESTS_PER_TX),
                nullifier_non_existent_read_request_hints_builder
            };
            builder.set_nullifiers_for_non_existent_read_request_hints();
            builder
        }

        pub fn with_nullifier_tree(&mut self) -> Self {
            let nullifier_tree = build_nullifier_tree();
            self.previous_kernel.historical_header.state.partial.nullifier_tree.root = nullifier_tree.get_root();
            self.nullifier_non_existent_read_request_hints_builder.set_nullifier_tree(nullifier_tree);
            *self
        }

        pub fn add_nullifier(&mut self, unsiloed_nullifier: Field) {
            self.previous_kernel.add_nullifier(unsiloed_nullifier);
            self.set_nullifiers_for_non_existent_read_request_hints();
        }

        pub fn append_nullifiers(&mut self, num_nullifiers: u64) {
            self.previous_kernel.append_new_nullifiers_from_public(num_nullifiers);
            self.set_nullifiers_for_non_existent_read_request_hints();
        }

        pub fn append_nullifiers_non_revertible(&mut self, num_nullifiers: u64) {
            self.previous_kernel.append_new_nullifiers_non_revertible_from_public(num_nullifiers);
            self.set_nullifiers_for_non_existent_read_request_hints();
        }

        fn set_nullifiers_for_non_existent_read_request_hints(&mut self) {
            let previous_kernel_public_inputs = self.previous_kernel.to_public_kernel_data().public_inputs;
            let nullifiers = array_concat(
                previous_kernel_public_inputs.end_non_revertible.new_nullifiers,
                previous_kernel_public_inputs.end.new_nullifiers
            );
            self.nullifier_non_existent_read_request_hints_builder.set_nullifiers(nullifiers);
        }

        pub fn add_nullifier_pending_read(&mut self, nullifier_index: u64) {
            let read_request_index = self.previous_kernel.add_read_request_for_pending_nullifier(nullifier_index);
            let hint_index = self.nullifier_read_request_hints_builder.pending_read_hints.len();
            let pending_value_index = nullifier_index + self.previous_kernel.end_non_revertible.new_nullifiers.len();
            let hint = PendingReadHint { read_request_index, pending_value_index };
            self.nullifier_read_request_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn add_non_revertible_nullifier_pending_read(&mut self, nullifier_index_offset_one: u64) {
            let nullifier_index = nullifier_index_offset_one + 1; // + 1 is for the first nullifier
            let read_request_index = self.previous_kernel.add_read_request_for_pending_non_revertible_nullifier(nullifier_index);
            let hint_index = self.nullifier_read_request_hints_builder.pending_read_hints.len();
            let hint = PendingReadHint { read_request_index, pending_value_index: nullifier_index };
            self.nullifier_read_request_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn read_non_existent_nullifier(&mut self, unsiloed_nullifier: Field) {
            self.previous_kernel.add_non_existent_read_request_for_nullifier(unsiloed_nullifier);
            let siloed_nullifier = silo_nullifier(
                self.previous_kernel.storage_contract_address,
                unsiloed_nullifier
            );
            self.nullifier_non_existent_read_request_hints_builder.add_value_read(siloed_nullifier);
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            let previous_kernel = self.previous_kernel.to_public_kernel_data();

            let kernel = PublicKernelTailCircuitPrivateInputs {
                previous_kernel,
                nullifier_read_request_hints: self.nullifier_read_request_hints_builder.to_hints(),
                nullifier_non_existent_read_request_hints: self.nullifier_non_existent_read_request_hints_builder.to_hints()
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
    unconstrained fn one_pending_nullifier_read_request() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        builder.succeeded();
    }

    #[test]
    unconstrained fn two_pending_nullifier_read_requests() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        builder.add_nullifier_pending_read(0);

        builder.succeeded();
    }

    #[test]
    unconstrained fn one_pending_nullifier_read_request_non_revertible() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers_non_revertible(3);
        builder.add_non_revertible_nullifier_pending_read(1);
        builder.succeeded();
    }

    #[test(should_fail_with="Hinted value does not match read request")]
    unconstrained fn pending_nullifier_read_request_wrong_hint_fails() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        let mut hint = builder.nullifier_read_request_hints_builder.pending_read_hints.pop();
        hint.pending_value_index -= 1;
        builder.nullifier_read_request_hints_builder.pending_read_hints.push(hint);

        builder.failed();
    }

    #[test(should_fail_with="Read request counter must be greater than counter of the value being read")]
    unconstrained fn pending_nullifier_read_request_reads_before_value_fails() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        let nullifier_being_read = builder.previous_kernel.end.new_nullifiers.get(1);
        let mut read_request = builder.previous_kernel.validation_requests.nullifier_read_requests.pop();
        read_request.counter = nullifier_being_read.counter - 1;
        builder.previous_kernel.validation_requests.nullifier_read_requests.push(read_request);

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

        builder.succeeded();
    }
}
