use crate::{
    components::{
    previous_kernel_validator::PreviousKernelValidator,
    public_call_data_validator::PublicCallDataValidator,
    public_kernel_output_composer::PublicKernelOutputComposer
},
    public_kernel_phase::PublicKernelPhase
};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, public_kernel_data::PublicKernelData,
    public_call_data::PublicCallData
},
    constants::{PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX, PUBLIC_KERNEL_SETUP_INDEX}
};

global ALLOWED_PREVIOUS_CIRCUITS = [
    PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX,
    PUBLIC_KERNEL_SETUP_INDEX,
];

struct PublicKernelSetupCircuitPrivateInputs {
    // Note: One might think that our previous_kernel ought to be
    // a PrivateKernelTailData. However, we instead supply a PublicKernelData.
    // This is because PrivateKernelTailData is a subset of PublicKernelData.
    // And we just initialize the missing values to zero in TS before passing it to the circuit,
    // except for the constants.global_variables which we populate with the current block values.
    // This is a bit of a hack, but it allows us to reuse the setup circuit until
    // the setup phase of the public kernel is complete. Maybe in a perfect world we would
    // have a SetupInit, SetupInner, etc, but this will change anyway once the public VM is able to
    // consume entire enqueued public calls.
    previous_kernel: PublicKernelData,
    public_call: PublicCallData,
}

impl PublicKernelSetupCircuitPrivateInputs {
    fn execute(self) -> PublicKernelCircuitPublicInputs {
        let phase = PublicKernelPhase.SETUP;

        let previous_kernel_validator = PreviousKernelValidator::new(self.previous_kernel);
        previous_kernel_validator.validate_phase(phase);
        previous_kernel_validator.validate_proof(ALLOWED_PREVIOUS_CIRCUITS);

        let call_data_validator = PublicCallDataValidator::new(self.public_call, phase);
        call_data_validator.validate();
        call_data_validator.validate_against_previous_kernel(self.previous_kernel.public_inputs);

        // noir-fmt:ignore
        PublicKernelOutputComposer::new_from_previous_kernel(self.previous_kernel.public_inputs)
            .remove_top_call_request(phase)
            .propagate_from_public_call(self.public_call.call_stack_item.public_inputs, phase)
            .finish()
    }
}

mod tests {
    use crate::{public_kernel_setup::{PublicKernelSetupCircuitPrivateInputs, ALLOWED_PREVIOUS_CIRCUITS}};
    use dep::types::{
        abis::{
        function_selector::FunctionSelector, gas::Gas,
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, max_block_number::MaxBlockNumber,
        note_hash::NoteHash, public_call_stack_item_compressed::PublicCallStackItemCompressed,
        public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest,
        public_call_data::PublicCallData, public_kernel_data::PublicKernelData,
        read_request::ReadRequest
    },
        address::AztecAddress, constants::{PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX, BASE_ROLLUP_INDEX},
        contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
        tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq}, traits::is_empty
    };

    struct PublicKernelSetupCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        previous_revertible: FixtureBuilder,
        public_call: FixtureBuilder,
    }

    impl PublicKernelSetupCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new().as_parent_contract().in_vk_tree(PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX);
            let previous_revertible = FixtureBuilder::new();
            let mut public_call = FixtureBuilder::new().is_public_function();
            public_call.value_offset = 9999; // Add an offset so that the mock data won't be the same as the values in previous_kernel.

            PublicKernelSetupCircuitPrivateInputsBuilder { previous_kernel, previous_revertible, public_call }
        }

        pub fn is_delegate_call(&mut self) -> Self {
            let _ = self.public_call.is_delegate_call();
            *self
        }

        pub fn to_public_data_reads<let N: u32>(self, contract_storage_reads: [StorageRead; N]) -> [PublicDataRead; N] {
            let storage_contract_address = self.public_call.storage_contract_address;
            contract_storage_reads.map(|r: StorageRead| PublicDataRead::from_contract_storage_read(storage_contract_address, r))
        }

        pub fn to_public_data_update_requests<let N: u32>(
            self,
            storage_update_requests: [StorageUpdateRequest; N]
        ) -> [PublicDataUpdateRequest; N] {
            let storage_contract_address = self.public_call.storage_contract_address;
            storage_update_requests.map(
                |r: StorageUpdateRequest|
                PublicDataUpdateRequest::from_contract_storage_update_request(storage_contract_address, r)
            )
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            let public_call = self.public_call.to_public_call_data();

            // In order to run the setup circuit, we must have an enqueued public call for setup.
            let compressed_item = public_call.call_stack_item.get_compressed();
            self.previous_kernel.push_public_call_request(compressed_item);

            let mut previous_kernel = self.previous_kernel.to_public_kernel_data(false);

            previous_kernel.public_inputs.end = self.previous_revertible.to_public_accumulated_data();

            // Run the kernel on the setup call
            let kernel = PublicKernelSetupCircuitPrivateInputs { previous_kernel, public_call };
            kernel.execute()
        }

        pub fn get_call_request_item(self) -> PublicCallStackItemCompressed {
            self.public_call.to_public_call_stack_item().get_compressed()
        }

        pub fn failed_with_call_request_item(&mut self, item: PublicCallStackItemCompressed) {
            let public_call = self.public_call.to_public_call_data();
            self.previous_kernel.push_public_call_request(item);
            let previous_kernel = self.previous_kernel.to_public_kernel_data(false);

            let kernel = PublicKernelSetupCircuitPrivateInputs { previous_kernel, public_call };
            let _ = kernel.execute();
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }
    }

    #[test(should_fail_with="Bytecode hash cannot be zero")]
    fn no_bytecode_hash_should_fail() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.bytecode_hash = 0;

        builder.failed();
    }

    #[test(should_fail_with="Contract address cannot be zero")]
    fn contract_address_must_be_valid() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.append_public_call_requests(1);
        builder.public_call.contract_address = AztecAddress::zero();

        builder.failed();
    }

    #[test(should_fail_with="Function signature cannot be zero")]
    fn function_selector_must_be_valid() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.function_data.selector = FunctionSelector::zero();

        builder.failed();
    }

    #[test(should_fail_with="Cannot execute a private function with the public kernel circuit")]
    fn private_call_should_fail() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.function_data.is_private = true;

        builder.failed();
    }

    #[test(should_fail_with="call stack item does not match item at the top of the call stack")]
    fn inconsistent_call_request_item_should_fail() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        let mut item = builder.get_call_request_item();
        // Tweak the call stack item.
        item.args_hash += 1;

        builder.failed_with_call_request_item(item);
    }

    #[test(should_fail_with="call stack item does not match item at the top of the call stack")]
    fn incorrect_msg_sender_fails_for_regular_calls() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        let mut item = builder.get_call_request_item();
        // Set the msg_sender to a wrong value.
        item.call_context.msg_sender.inner += 1;

        builder.failed_with_call_request_item(item);
    }

    #[test(should_fail_with="call stack storage address does not match expected contract address")]
    fn incorrect_storage_contract_address_fails_for_regular_calls() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        // Set the storage contract address to a wrong value.
        builder.public_call.storage_contract_address.inner += 1;

        builder.failed();
    }

    #[test]
    fn delegate_call_succeeds() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new().is_delegate_call();
        builder.succeeded();
    }

    #[test(should_fail_with="call stack item does not match item at the top of the call stack")]
    fn incorrect_msg_sender_for_delegate_calls_fails() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new().is_delegate_call();

        let mut item = builder.get_call_request_item();
        // Set the msg_sender to be the caller contract.
        item.call_context.msg_sender = builder.previous_kernel.contract_address;

        builder.failed_with_call_request_item(item);
    }

    #[test(should_fail_with="curent contract address must not match storage contract address for delegate calls")]
    fn previous_private_kernel_fails_if_incorrect_storage_contract_on_delegate_call() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new().is_delegate_call();

        builder.public_call.contract_address = builder.public_call.storage_contract_address;

        builder.failed();
    }

    #[test(should_fail_with = "incorrect storage_contract_address for call request")]
    fn incorrect_storage_contract_address_for_call_request_fails() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(1);
        let mut call_request = builder.public_call.public_call_requests.pop();
        // Change the caller contract address to be a different value.
        call_request.item.call_context.storage_contract_address.inner += 1;
        builder.public_call.public_call_requests.push(call_request);

        builder.failed();
    }

    #[test(should_fail_with="incorrect storage_contract_address for delegate call request")]
    fn incorrect_call_context_for_delegate_call_request_fails() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests_delegate(1);
        let mut call_request = builder.public_call.public_call_requests.pop();
        // Change the storage contract address to be the contract address.
        call_request.item.call_context.storage_contract_address = call_request.item.contract_address;
        builder.public_call.public_call_requests.push(call_request);

        builder.failed();
    }

    #[test]
    fn public_kernel_circuit_with_private_previous_kernel_should_succeed() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.succeeded();
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_private_kernel() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.max_block_number = MaxBlockNumber::new(13);

        builder.public_call.append_public_call_requests(2);
        let curr_calls = builder.public_call.public_call_requests.storage;

        builder.previous_kernel.append_public_call_requests(1);
        let prev_calls = builder.previous_kernel.public_call_requests.storage;

        builder.public_call.append_contract_storage_update_requests(2);
        let update_requests = builder.public_call.contract_storage_update_requests.storage;

        builder.public_call.append_contract_storage_read_requests(3);
        let read_requests = builder.public_call.contract_storage_reads.storage;

        let public_inputs = builder.execute();

        assert_eq(public_inputs.validation_requests.for_rollup.max_block_number.unwrap(), 13);
        assert_array_eq(
            public_inputs.end_non_revertible.public_call_stack,
            [prev_calls[0], curr_calls[0], curr_calls[1]]
        );
        assert_array_eq(
            public_inputs.end_non_revertible.public_data_update_requests,
            builder.to_public_data_update_requests([update_requests[0], update_requests[1]])
        );
        assert_array_eq(
            public_inputs.validation_requests.public_data_reads,
            builder.to_public_data_reads([read_requests[0], read_requests[1], read_requests[2]])
        );
    }

    #[test(should_fail_with="Cannot run unnecessary setup circuit")]
    fn previous_kernel_empty_public_call_stack_should_fail() {
        let builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(false);
        let public_call = builder.public_call.to_public_call_data();
        let kernel = PublicKernelSetupCircuitPrivateInputs { previous_kernel, public_call };
        let _ = kernel.execute();
    }

    #[test(should_fail_with="No contract storage update requests are allowed for static calls")]
    fn previous_private_kernel_fails_if_contract_storage_update_requests_on_static_call() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.is_static_call = true;
        builder.public_call.append_contract_storage_update_requests(1);

        builder.failed();
    }

    #[test]
    fn logs_are_handled_as_expected() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        // Logs for the current call stack.
        let unencrypted_logs_hash = 26;
        let unencrypted_log_preimages_length = 50;
        builder.public_call.add_unencrypted_log_hash(unencrypted_logs_hash, unencrypted_log_preimages_length);

        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = 80;
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = 956;
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.add_encrypted_log_hash(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.add_unencrypted_log_hash(
            prev_unencrypted_logs_hash,
            prev_unencrypted_log_preimages_length
        );
        let prev_data = builder.previous_kernel.to_public_accumulated_data();

        let expected_unencrypted_logs = [prev_data.unencrypted_logs_hashes[0], builder.public_call.unencrypted_logs_hashes.storage[0]];
        // we assume the encrypted log is already siloed from private kernels
        let expected_encrypted_logs = [prev_data.encrypted_logs_hashes[0]];

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.end_non_revertible.encrypted_logs_hashes,
            expected_encrypted_logs
        );
        assert_array_eq(
            public_inputs.end_non_revertible.unencrypted_logs_hashes,
            expected_unencrypted_logs
        );
    }

    #[test]
    fn propagate_nullifier_non_existent_read_requests() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        let storage_contract_address = builder.public_call.storage_contract_address;

        let request_0 = ReadRequest { value: 123, counter: 4567 }.scope(storage_contract_address);
        builder.public_call.nullifier_non_existent_read_requests.push(request_0);
        let request_1 = ReadRequest { value: 777888, counter: 90 }.scope(storage_contract_address);
        builder.public_call.nullifier_non_existent_read_requests.push(request_1);

        let public_inputs = builder.execute();

        let end_requests = public_inputs.validation_requests.nullifier_non_existent_read_requests;
        assert_array_eq(end_requests, [request_0, request_1]);
    }

    #[test(should_fail_with="Public call cannot be reverted")]
    fn fails_if_public_call_reverted() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.revert_code = 1;

        builder.failed();
    }

    #[test]
    fn updates_non_revertible_gas_used() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        // Transaction gas limit is 1k
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1000, 1000);

        // Revertible has already used 100
        builder.previous_revertible.gas_used = Gas::new(100, 100);

        // And non-revertible has used another 200
        builder.previous_kernel.gas_used = Gas::new(200, 200);

        // So this call starts with 700 gas left
        builder.public_call.start_gas_left = Gas::new(700, 700);

        // And uses 300, ending with 400 left
        builder.public_call.end_gas_left = Gas::new(400, 400);

        // So the updated gas used by non-revertible must go up by 300, and revertible must stay the same
        let output = builder.execute();
        assert_eq(output.end_non_revertible.gas_used, Gas::new(500, 500));
        assert_eq(output.end.gas_used, Gas::new(100, 100));
    }

    #[test(should_fail_with="Start gas for public phase does not match transaction gas left")]
    fn validates_start_gas() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.public_call.is_execution_request = true; // don't need to check start gas for nested calls
        builder.public_call.start_gas_left = Gas::new(100, 100);

        builder.failed();
    }

    #[test(should_fail_with="Transaction fee must be zero on setup and app phases")]
    fn validates_transaction_fee() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.public_call.transaction_fee = 10;

        builder.failed();
    }

    #[test]
    fn propagates_global_variables_if_empty() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.public_call.global_variables.block_number = 11;

        let public_inputs = builder.execute();

        assert_eq(public_inputs.constants.global_variables.block_number, 11);
    }

    #[test(should_fail_with="Global variables injected into the public call do not match constants")]
    fn validates_global_variables() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.global_variables.block_number = 10;
        builder.public_call.global_variables.block_number = 11;

        builder.failed();
    }

    #[test]
    unconstrained fn propagate_fee_payer() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        let fee_payer = AztecAddress::from_field(123);
        builder.previous_kernel.set_fee_payer(fee_payer);
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer is not set if is_fee_payer is false
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        assert_eq(is_empty(builder.previous_kernel.fee_payer), true);
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());
    }

    #[test]
    fn propagate_call_stacks_on_success() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.append_public_call_requests(1);
        let prev_non_rev_calls = builder.previous_kernel.public_call_requests.storage;

        builder.previous_revertible.append_public_call_requests(2);
        let prev_rev_calls = builder.previous_revertible.public_call_requests.storage;

        builder.previous_kernel.append_public_teardown_call_requests(1);
        let prev_teardoen_calls = builder.previous_kernel.public_teardown_call_stack.storage;

        builder.public_call.append_public_call_requests(2);
        let curr_calls = builder.public_call.public_call_requests.storage;

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.end_non_revertible.public_call_stack,
            [prev_non_rev_calls[0], curr_calls[0], curr_calls[1]]
        );
        assert_array_eq(
            public_inputs.end.public_call_stack,
            [prev_rev_calls[0], prev_rev_calls[1]]
        );
        assert_array_eq(
            public_inputs.public_teardown_call_stack,
            [prev_teardoen_calls[0]]
        );
    }

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
            builder.previous_kernel = builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }
    // TODO(#7410) we need the tube vk to reinstate this
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_kernel() {
    //     let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
    //     builder.previous_kernel = builder.previous_kernel.in_vk_tree(BASE_ROLLUP_INDEX);
    //     let _res = builder.execute();
    // }
}
