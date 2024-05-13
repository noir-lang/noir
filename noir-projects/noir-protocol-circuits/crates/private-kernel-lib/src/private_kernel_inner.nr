use crate::{
    private_call_data_validator::PrivateCallDataValidator,
    private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer
};
use dep::types::{
    abis::{
    private_kernel_data::{PrivateKernelData, verify_previous_kernel_proof},
    private_kernel::private_call_data::{PrivateCallData, verify_private_call},
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs
},
    constants::MAX_NEW_NOTE_HASHES_PER_CALL, utils::arrays::array_length
};

struct PrivateKernelInnerHints {
    note_hash_nullifier_counters: [u32; MAX_NEW_NOTE_HASHES_PER_CALL],
}

struct PrivateKernelInnerCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
    private_call: PrivateCallData,
    hints: PrivateKernelInnerHints,
}

impl PrivateKernelInnerCircuitPrivateInputs {
    pub fn native_private_kernel_circuit_inner(self) -> PrivateKernelCircuitPublicInputs {
        // verify/aggregate the private call proof
        verify_private_call(self.private_call);

        // verify/aggregate the previous kernel
        verify_previous_kernel_proof(self.previous_kernel);

        let privateCallDataValidator = PrivateCallDataValidator::new(self.private_call);
        privateCallDataValidator.validate();

        let private_call_stack = self.previous_kernel.public_inputs.end.private_call_stack;
        // TODO: Should be a hint from private inputs.
        let private_call_stack_size = array_length(private_call_stack);
        let call_request = private_call_stack[private_call_stack_size - 1];
        privateCallDataValidator.validate_against_call_request(call_request);

        PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(self.previous_kernel.public_inputs).compose(
            self.private_call.call_stack_item.public_inputs,
            self.hints.note_hash_nullifier_counters,
            self.private_call.private_call_stack,
            self.private_call.public_call_stack,
            self.private_call.public_teardown_call_request
        ).finish()
    }
}

mod tests {
    use crate::private_kernel_inner::{PrivateKernelInnerCircuitPrivateInputs, PrivateKernelInnerHints};
    use dep::types::constants::MAX_NEW_NOTE_HASHES_PER_TX;
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
        max_block_number::MaxBlockNumber, note_hash::NoteHash, nullifier::Nullifier,
        read_request::ReadRequest, side_effect::SideEffect
    },
        address::{AztecAddress, EthAddress}, constants::MAX_NEW_NOTE_HASHES_PER_CALL,
        messaging::l2_to_l1_message::L2ToL1Message, utils::{arrays::array_length},
        tests::{private_call_data_builder::PrivateCallDataBuilder, fixture_builder::FixtureBuilder}
    };

    struct PrivateKernelInnerInputsBuilder {
        previous_kernel: FixtureBuilder,
        private_call: PrivateCallDataBuilder,
        hints: PrivateKernelInnerHints,
    }

    impl PrivateKernelInnerInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new();
            let private_call = PrivateCallDataBuilder::new();
            let hints = PrivateKernelInnerHints { note_hash_nullifier_counters: [0; MAX_NEW_NOTE_HASHES_PER_CALL] };

            // 0th nullifier must be non-zero.
            previous_kernel.append_new_nullifiers(1);

            PrivateKernelInnerInputsBuilder { previous_kernel, private_call, hints }
        }

        pub fn is_delegate_call(&mut self) -> Self {
            let _ = self.private_call.is_delegate_call();
            *self
        }

        pub fn is_static_call(&mut self) -> Self {
            let _ = self.private_call.is_static_call();
            *self
        }

        pub fn execute(&mut self) -> PrivateKernelCircuitPublicInputs {
            let private_call = self.private_call.finish();
            // Update the previous_kernel's private_call_stack with the current call_stack_item.
            let hash = private_call.call_stack_item.hash();
            let is_delegate_call = private_call.call_stack_item.public_inputs.call_context.is_delegate_call;
            self.previous_kernel.push_private_call_request(hash, is_delegate_call);
            let previous_kernel = self.previous_kernel.to_private_kernel_data();

            let kernel = PrivateKernelInnerCircuitPrivateInputs { previous_kernel, private_call, hints: self.hints };

            kernel.native_private_kernel_circuit_inner()
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }
    }

    #[test(should_fail_with = "push out of bounds")]
    fn private_kernel_should_fail_if_aggregating_too_many_note_hashes() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // The current call stack has 1 note_hash;
        builder.private_call.public_inputs.new_note_hashes.push(NoteHash { value: 4321, counter: 0 });

        // Mock the previous new note hashes to be full, therefore no more note_hashes can be added.
        builder.previous_kernel.append_new_note_hashes(MAX_NEW_NOTE_HASHES_PER_TX);

        builder.failed();
    }

    #[test]
    fn propagate_note_hashes_with_nullifier_counters() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        let note_hashes = [
            NoteHash { value: 12, counter: 3 },
            NoteHash { value: 45, counter: 6 },
            NoteHash { value: 78, counter: 9 }
        ];
        builder.private_call.public_inputs.new_note_hashes.extend_from_array(note_hashes);
        builder.hints.note_hash_nullifier_counters[0] = 10;
        builder.hints.note_hash_nullifier_counters[2] = 20;

        let public_inputs = builder.execute();

        assert_eq(public_inputs.end.new_note_hashes[0].nullifier_counter, 10);
        assert_eq(public_inputs.end.new_note_hashes[1].nullifier_counter, 0);
        assert_eq(public_inputs.end.new_note_hashes[2].nullifier_counter, 20);
    }

    #[test(should_fail_with="Invalid nullifier counter")]
    fn propagate_note_hashes_with_incorrect_nullifier_counters_fails() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        builder.private_call.public_inputs.new_note_hashes.push(NoteHash { value: 12, counter: 3 });
        builder.hints.note_hash_nullifier_counters[0] = 2; // Less than the note hash's counter 3.

        builder.failed();
    }

    #[test]
    fn propagate_previous_kernel_max_block_number() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        builder.previous_kernel.max_block_number = MaxBlockNumber::new(13);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.validation_requests.for_rollup.max_block_number.unwrap(), 13);
    }

    #[test]
    fn propagate_max_block_number_request() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        builder.private_call.set_tx_max_block_number(42);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.validation_requests.for_rollup.max_block_number.unwrap(), 42);
    }

    #[test]
    fn ignore_larger_max_block_number() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        builder.previous_kernel.max_block_number = MaxBlockNumber::new(13);
        // A private call requesting a larger max_block_number should not change the current one as that constraint is
        // already satisfied.
        builder.private_call.set_tx_max_block_number(42);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.validation_requests.for_rollup.max_block_number.unwrap(), 13);
    }

    #[test]
    fn propagate_note_hash_read_requests() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.append_note_hash_read_requests(2);
        let prev_requests = builder.previous_kernel.note_hash_read_requests;

        let cur_requests = [ReadRequest { value: 123, counter: 4567 }, ReadRequest { value: 777888, counter: 90 }];
        builder.private_call.public_inputs.note_hash_read_requests.extend_from_array(cur_requests);
        let cur_storage_contract_address = builder.private_call.public_inputs.call_context.storage_contract_address;

        let public_inputs = builder.execute();

        let end_note_hash_read_requests = public_inputs.validation_requests.note_hash_read_requests;
        assert_eq(array_length(end_note_hash_read_requests), 4);

        assert_eq(end_note_hash_read_requests[0], prev_requests.storage[0]);
        assert_eq(end_note_hash_read_requests[1], prev_requests.storage[1]);

        let request = end_note_hash_read_requests[2];
        assert_eq(request.read_request, cur_requests[0]);
        assert_eq(request.contract_address, cur_storage_contract_address);

        let request = end_note_hash_read_requests[3];
        assert_eq(request.read_request, cur_requests[1]);
        assert_eq(request.contract_address, cur_storage_contract_address);
    }

    #[test]
    fn native_logs_are_set_as_expected() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Logs for the current call stack.
        let encrypted_logs_hash = 16;
        let encrypted_log_preimages_length = 100;
        let unencrypted_logs_hash = 26;
        let unencrypted_log_preimages_length = 50;
        builder.private_call.set_encrypted_logs(encrypted_logs_hash, encrypted_log_preimages_length);
        builder.private_call.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = 80;
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = 956;
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.set_encrypted_logs(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.set_unencrypted_logs(
            prev_unencrypted_logs_hash,
            prev_unencrypted_log_preimages_length
        );

        let public_inputs = builder.execute();

        assert_eq(
            public_inputs.end.encrypted_log_preimages_length, encrypted_log_preimages_length + prev_encrypted_log_preimages_length
        );
        assert_eq(
            public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length
        );

        assert_eq(public_inputs.end.encrypted_logs_hashes[0].value, prev_encrypted_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hashes[0].value, prev_unencrypted_logs_hash);
        assert_eq(public_inputs.end.encrypted_logs_hashes[1].value, encrypted_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hashes[1].value, unencrypted_logs_hash);
    }

    #[test]
    unconstrained fn propagate_fee_payer() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        let fee_payer = builder.private_call.public_inputs.call_context.storage_contract_address;
        builder.private_call.public_inputs.is_fee_payer = true;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer is not set if is_fee_payer is false
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        assert_eq(builder.private_call.public_inputs.is_fee_payer, false);
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());

        // Check that we carry forward if the fee payer is already set
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        let fee_payer = AztecAddress::from_field(123);
        builder.previous_kernel.fee_payer = fee_payer;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);
    }

    #[test(should_fail_with="Cannot overwrite non-empty fee_payer")]
    unconstrained fn does_not_overwrite_fee_payer() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();
        let original_fee_payer = AztecAddress::from_field(123);

        builder.private_call.public_inputs.is_fee_payer = true;
        builder.previous_kernel.fee_payer = original_fee_payer;

        builder.failed();
    }
}
