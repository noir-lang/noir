use crate::common;
use dep::reset_kernel_lib::{NullifierReadRequestResetHints, reset_read_requests};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, PublicKernelCircuitPublicInputsBuilder},
    kernel_data::PublicKernelData, side_effect::SideEffectLinkedToNoteHash
},
    constants::MAX_NEW_NULLIFIERS_PER_TX, utils::{arrays::{array_length, array_merge, array_concat}}
};
use dep::std::unsafe;

struct PublicKernelTailCircuitPrivateInputs {
    previous_kernel: PublicKernelData,
    nullifier_read_request_reset_hints: NullifierReadRequestResetHints,
}

impl PublicKernelTailCircuitPrivateInputs {
    fn propagate_reverted_flag(self, public_inputs: &mut PublicKernelCircuitPublicInputsBuilder) {
        public_inputs.reverted = self.previous_kernel.public_inputs.reverted;
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

        // Total number should not exceed MAX_NULLIFIER_READ_REQUESTS_PER_TX.
        let requests = array_merge(
            end_non_revertible.nullifier_read_requests,
            end.nullifier_read_requests
        );

        let pending_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX] = array_concat(end_non_revertible.new_nullifiers, end.new_nullifiers);

        let hints = self.nullifier_read_request_reset_hints;

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

    pub fn public_kernel_tail(self) -> PublicKernelCircuitPublicInputs {
        let mut public_inputs: PublicKernelCircuitPublicInputsBuilder = unsafe::zeroed();

        self.validate_inputs();

        self.propagate_reverted_flag(&mut public_inputs);

        common::initialize_emitted_end_values(self.previous_kernel, &mut public_inputs);

        self.validate_nullifier_read_requests(&mut public_inputs);

        public_inputs.to_inner()
    }
}

mod tests {
    use crate::{public_kernel_tail::PublicKernelTailCircuitPrivateInputs};
    use dep::reset_kernel_lib::{
        NullifierReadRequestResetHintsBuilder,
        read_request_reset::{PendingReadHint, ReadRequestState, ReadRequestStatus}
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, PublicKernelCircuitPublicInputsBuilder},
        kernel_data::PublicKernelData
    },
        constants::MAX_NULLIFIER_READ_REQUESTS_PER_TX,
        tests::{kernel_data_builder::PreviousKernelDataBuilder}
    };

    struct PublicKernelTailCircuitPrivateInputsBuilder {
        previous_kernel: PreviousKernelDataBuilder,
        nullifier_read_request_reset_hints_builder: NullifierReadRequestResetHintsBuilder,
    }

    impl PublicKernelTailCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = PreviousKernelDataBuilder::new(true);

            PublicKernelTailCircuitPrivateInputsBuilder {
                previous_kernel,
                nullifier_read_request_reset_hints_builder: NullifierReadRequestResetHintsBuilder::new(MAX_NULLIFIER_READ_REQUESTS_PER_TX)
            }
        }

        pub fn append_nullifiers(&mut self, num_nullifiers: u64) {
            self.previous_kernel.append_new_nullifiers_from_public(num_nullifiers);
        }

        pub fn append_nullifiers_non_revertible(&mut self, num_nullifiers: u64) {
            self.previous_kernel.append_new_nullifiers_non_revertible_from_public(num_nullifiers);
        }

        pub fn add_nullifier_pending_read(&mut self, nullifier_index: u64) {
            let read_request_index = self.previous_kernel.add_read_request_for_pending_nullifier(nullifier_index);
            let hint_index = self.nullifier_read_request_reset_hints_builder.pending_read_hints.len();
            let pending_value_index = nullifier_index + self.previous_kernel.end_non_revertible.new_nullifiers.len();
            let hint = PendingReadHint { read_request_index, pending_value_index };
            self.nullifier_read_request_reset_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_reset_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn add_nullifier_pending_read_non_revertible(&mut self, nullifier_index_offset_one: u64) {
            let nullifier_index = nullifier_index_offset_one + 1; // + 1 is for the first nullifier
            let read_request_index = self.previous_kernel.add_read_request_for_pending_nullifier_non_revertible(nullifier_index);
            let hint_index = self.nullifier_read_request_reset_hints_builder.pending_read_hints.len();
            let hint = PendingReadHint { read_request_index, pending_value_index: nullifier_index };
            self.nullifier_read_request_reset_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_reset_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            let previous_kernel = self.previous_kernel.to_public_kernel_data();

            let kernel = PublicKernelTailCircuitPrivateInputs {
                previous_kernel,
                nullifier_read_request_reset_hints: self.nullifier_read_request_reset_hints_builder.to_hints()
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
    fn public_kernel_circuit_tail_succeeds() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();
        builder.succeeded();
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
        builder.add_nullifier_pending_read_non_revertible(1);
        builder.succeeded();
    }

    #[test(should_fail_with="Hinted value does not match read request")]
    unconstrained fn pending_nullifier_read_request_wrong_hint_fails() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        let mut hint = builder.nullifier_read_request_reset_hints_builder.pending_read_hints.pop();
        hint.pending_value_index -= 1;
        builder.nullifier_read_request_reset_hints_builder.pending_read_hints.push(hint);

        builder.failed();
    }

    #[test(should_fail_with="Read request counter must be greater than counter of the value being read")]
    unconstrained fn pending_nullifier_read_request_reads_before_value_fails() {
        let mut builder = PublicKernelTailCircuitPrivateInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        let nullifier_being_read = builder.previous_kernel.end.new_nullifiers.get(1);
        let mut read_request = builder.previous_kernel.end.nullifier_read_requests.pop();
        read_request.counter = nullifier_being_read.counter - 1;
        builder.previous_kernel.end.nullifier_read_requests.push(read_request);

        builder.failed();
    }
}
