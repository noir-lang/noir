use crate::common;
use dep::types::abis::{
    kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, PublicKernelCircuitPublicInputsBuilder},
    kernel_data::PublicKernelData, public_call_data::PublicCallData
};
use dep::std::unsafe;

struct PublicKernelSetupCircuitPrivateInputs {
    // Note: One might think that our previous_kernel ought to be
    // a PrivateKernelTailData. However, we instead supply a PublicKernelData.
    // This is because PrivateKernelTailData is a subset of PublicKernelData.
    // And we just initialize the missing values to zero in TS before passing it to the circuit.
    // This is a bit of a hack, but it allows us to reuse the setup circuit until
    // the setup phase of the public kernel is complete. Maybe in a perfect world we would 
    // have a SetupInit, SetupInner, etc, but this will change anyway once the public VM is able to 
    // consume entire enqueued public calls.
    previous_kernel: PublicKernelData,
    public_call: PublicCallData,
}

impl PublicKernelSetupCircuitPrivateInputs {
    fn validate_inputs(self) {
        let needs_setup = self.previous_kernel.public_inputs.needs_setup();
        assert(needs_setup == true, "Cannot run unnecessary setup circuit");
    }

    fn public_kernel_setup(self) -> PublicKernelCircuitPublicInputs {
        // construct the circuit outputs
        let mut public_inputs: PublicKernelCircuitPublicInputsBuilder = unsafe::zeroed();
        // since this phase is non-revertible, we must assert the public call did not revert
        common::validate_public_call_non_revert(self.public_call);
        common::initialize_revert_code(self.previous_kernel, self.public_call, &mut public_inputs);

        // initialise the end state with our provided previous kernel state
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        // validate the inputs common to all invocation circumstances
        common::validate_inputs(self.public_call);

        // validate the inputs unique to having a previous private kernel
        self.validate_inputs();

        // Pops the item from the call stack and validates it against the current execution.
        let call_request = public_inputs.end_non_revertible.public_call_stack.pop();
        common::validate_call_against_request(self.public_call, call_request);

        common::update_validation_requests(self.public_call, &mut public_inputs);

        common::update_public_end_non_revertible_values(self.public_call, &mut public_inputs);

        common::accumulate_unencrypted_logs(
            self.public_call,
            self.previous_kernel.public_inputs.end.unencrypted_logs_hash,
            self.previous_kernel.public_inputs.end.unencrypted_log_preimages_length,
            &mut public_inputs
        );

        public_inputs.finish()
    }
}

mod tests {
    use crate::{
        public_kernel_setup::PublicKernelSetupCircuitPrivateInputs,
        utils::{
        assert_eq_call_requests, assert_eq_public_data_reads, assert_eq_public_data_update_requests,
        compute_public_data_reads, compute_public_data_update_requests
    }
    };
    use dep::types::{
        abis::{
        call_request::CallRequest, function_selector::FunctionSelector,
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, max_block_number::MaxBlockNumber,
        public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest,
        public_call_data::PublicCallData, read_request::ReadRequest
    },
        address::{AztecAddress, EthAddress}, contract_class_id::ContractClassId,
        contrakt::storage_read::StorageRead, hash::compute_logs_hash,
        tests::{fixture_builder::FixtureBuilder, public_call_data_builder::PublicCallDataBuilder},
        utils::{arrays::{array_eq, array_length}}
    };
    use dep::types::constants::{
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_READS_PER_CALL,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL
    };

    struct PublicKernelSetupCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        previous_revertible: FixtureBuilder,
        public_call: PublicCallDataBuilder,
    }

    impl PublicKernelSetupCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new();
            let previous_revertible = FixtureBuilder::new();
            let public_call = PublicCallDataBuilder::new();

            PublicKernelSetupCircuitPrivateInputsBuilder { previous_kernel, previous_revertible, public_call }
        }

        pub fn stub_teardown_call(&mut self) {
            let teardown_call = PublicCallDataBuilder::new();
            let teardown_call = teardown_call.finish();
            let teardown_call_hash = teardown_call.call_stack_item.hash();
            let teardown_is_delegate_call = teardown_call.call_stack_item.public_inputs.call_context.is_delegate_call;
            self.previous_kernel.push_public_call_request(teardown_call_hash, teardown_is_delegate_call);
        }

        pub fn push_public_call(&mut self, public_call: PublicCallData) {
            let public_call_hash = public_call.call_stack_item.hash();
            let setup_is_delegate_call = public_call.call_stack_item.public_inputs.call_context.is_delegate_call;
            self.previous_kernel.push_public_call_request(public_call_hash, setup_is_delegate_call);
        }

        pub fn is_delegate_call(&mut self) -> Self {
            let _ = self.public_call.is_delegate_call();
            *self
        }

        pub fn get_current_public_data_reads(self) -> [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_CALL] {
            let read_requests = self.public_call.public_inputs.contract_storage_reads.storage;
            compute_public_data_reads(self.public_call.contract_address, read_requests)
        }

        pub fn get_current_public_data_update_requests(self) -> [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL] {
            compute_public_data_update_requests(
                self.public_call.contract_address,
                self.public_call.public_inputs.contract_storage_update_requests.storage
            )
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            // In order to run the setup circuit, we must have an enqueued public call for setup.
            // In order to have an enqueued public call for setup, we must have an enqueued public call for teardown.

            // The teardown call is at the bottom of the call stack.
            self.stub_teardown_call();

            // Push the public call on top of the teardown call.
            let setup_call = self.public_call.finish();
            self.push_public_call(setup_call);
            let mut previous_kernel = self.previous_kernel.to_public_kernel_data(false);
            previous_kernel.public_inputs.end = self.previous_revertible.to_public_accumulated_data();

            // Run the kernel on the setup call
            let kernel = PublicKernelSetupCircuitPrivateInputs { previous_kernel, public_call: setup_call };

            kernel.public_kernel_setup()
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }
    }

    #[test]
    fn only_valid_public_data_reads_should_be_propagated() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.append_public_data_read_requests(1);
        builder.public_call.append_empty_public_data_read_requests(1);
        builder.public_call.append_public_data_read_requests(1);
        let _ = builder.get_current_public_data_reads();

        let _ = builder.execute();
        // TODO(fees) we don't yet handle non-revertible reads
        // Only the 2 valid reads should have been propagated.
        // let expected = [public_data_reads[0], public_data_reads[2]];
        // assert_eq_public_data_reads(public_inputs.end.public_data_reads, expected);
    }

    // TODO(fees) we don't yet handle non-revertible update requests
    // #[test]
    // fn only_valid_update_requests_should_be_propagated() {
    //     let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
    //     builder.public_call.append_update_requests(1);
    //     builder.public_call.append_empty_update_requests(1);
    //     builder.public_call.append_update_requests(1);
    //     let public_data_update_requests = builder.get_current_public_data_update_requests();

    //     let public_inputs = builder.execute();

    //     // Only the 2 valid reads should have been propagated.
    //     let expected = [public_data_update_requests[0], public_data_update_requests[2]];
    //     assert_eq_public_data_update_requests(public_inputs.end.public_data_update_requests, expected);
    // }

    #[test(should_fail_with="Bytecode hash cannot be zero")]
    fn no_bytecode_hash_should_fail() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.bytecode_hash = 0;

        builder.failed();
    }

    #[test(should_fail_with="Contract address cannot be zero")]
    fn contract_address_must_be_valid() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
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

    #[test(should_fail_with="calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the call stack")]
    fn inconsistent_call_hash_should_fail() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.stub_teardown_call();
        let public_call = builder.public_call.finish();

        let hash = public_call.call_stack_item.hash();
        // Tweak the call stack item hash.
        builder.previous_kernel.push_public_call_request(hash + 1, false);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(false);

        let kernel = PublicKernelSetupCircuitPrivateInputs { previous_kernel, public_call };

        let _ = kernel.public_kernel_setup();
    }

    #[test(should_fail_with="call stack msg_sender does not match caller contract address")]
    fn incorrect_msg_sender_fails_for_regular_calls() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        // Set the msg_sender to a wrong value.
        builder.public_call.public_inputs.call_context.msg_sender.inner += 1;

        builder.failed();
    }

    #[test(should_fail_with="call stack storage address does not match expected contract address")]
    fn incorrect_storage_contract_address_fails_for_regular_calls() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        // Set the storage contract address to a wrong value.
        builder.public_call.public_inputs.call_context.storage_contract_address.inner += 1;

        builder.failed();
    }

    #[test]
    fn delegate_call_succeeds() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new().is_delegate_call();
        builder.succeeded();
    }

    #[test(should_fail_with="caller context cannot be empty for delegate calls")]
    fn empty_caller_context_for_delegate_calls_fails() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new().is_delegate_call();
        builder.stub_teardown_call();

        let public_call = builder.public_call.finish();

        let hash = public_call.call_stack_item.hash();
        // Caller context is empty for regular calls.
        let is_delegate_call = false;
        builder.previous_kernel.push_public_call_request(hash, is_delegate_call);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(false);

        let kernel = PublicKernelSetupCircuitPrivateInputs { previous_kernel, public_call };

        let _ = kernel.public_kernel_setup();
    }

    #[test(should_fail_with="call stack msg_sender does not match expected msg_sender for delegate calls")]
    fn incorrect_msg_sender_for_delegate_calls_fails() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new().is_delegate_call();

        // Set the msg_sender to be the caller contract.
        builder.public_call.public_inputs.call_context.msg_sender = builder.previous_kernel.contract_address;

        builder.failed();
    }

    #[test(should_fail_with="curent contract address must not match storage contract address for delegate calls")]
    fn previous_private_kernel_fails_if_incorrect_storage_contract_on_delegate_call() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new().is_delegate_call();

        builder.public_call.contract_address = builder.public_call.public_inputs.call_context.storage_contract_address;

        builder.failed();
    }

    #[test(should_fail_with = "call stack hash does not match call request hash")]
    fn incorrect_call_request_hash_fails() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(2, false);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the hash to be a different value.
        call_request.hash += 1;
        builder.public_call.public_call_stack.push(call_request);

        builder.failed();
    }

    #[test(should_fail_with = "invalid caller")]
    fn incorrect_caller_address_for_call_request_fails() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(1, false);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the caller contract address to be a different value.
        call_request.caller_contract_address.inner += 1;
        builder.public_call.public_call_stack.push(call_request);

        builder.failed();
    }

    #[test(should_fail_with = "invalid caller")]
    fn incorrect_caller_context_for_delegate_call_request_fails() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(1, true);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the storage contract to be a different value.
        call_request.caller_context.storage_contract_address.inner += 1;
        builder.public_call.public_call_stack.push(call_request);

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

        builder.public_call.append_public_call_requests_for_regular_calls(2);
        let storage = builder.public_call.public_call_stack.storage;

        builder.stub_teardown_call();
        let non_revertible_call_stack = [
            builder.previous_kernel.public_call_stack.get(0),
            storage[0], storage[1]
        ];

        builder.public_call.append_update_requests(2);
        let storage = builder.get_current_public_data_update_requests();
        let _update_requests = [storage[0], storage[1]];

        builder.public_call.append_public_data_read_requests(3);
        let storage = builder.get_current_public_data_reads();
        let _read_requests = [storage[0], storage[1], storage[2]];

        // Push the public call on top of the teardown call.
        let setup_call = builder.public_call.finish();
        builder.push_public_call(setup_call);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(false);

        // Run the kernel on the setup call
        let kernel = PublicKernelSetupCircuitPrivateInputs { previous_kernel, public_call: setup_call };

        let public_inputs = kernel.public_kernel_setup();

        assert_eq(public_inputs.validation_requests.for_rollup.max_block_number.unwrap(), 13);
        assert_eq_call_requests(
            public_inputs.end_non_revertible.public_call_stack,
            non_revertible_call_stack
        );
        // TODO(fees) we don't yet handle non-revertible update requests
        // assert_eq_public_data_update_requests(
        //     public_inputs.end.public_data_update_requests,
        //     update_requests
        // );
        // assert_eq_public_data_reads(public_inputs.end.public_data_reads, read_requests);
    }

    // TODO: Find another way to test this. Currently it will crash because we are popping from an empty array:
    // The application panicked (crashed). Message:  Expected array index to fit in u64
    // #[test(should_fail_with="Public call stack can not be empty")]
    // fn private_previous_kernel_empty_public_call_stack_should_fail() {
    //     let builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
    //     let previous_kernel = builder.previous_kernel.finish();
    //     let kernel = PublicKernelSetupCircuitPrivateInputs {
    //         previous_kernel,
    //         public_call: builder.public_call.finish(),
    //     };

    //     let _ = kernel.public_kernel_setup();
    // }

    #[test(should_fail_with="Cannot run unnecessary setup circuit")]
    fn unnecessary_public_kernel_setup_with_teardown_should_fail() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        // in this case, we only push a single call, which is interpreted as the teardown call
        let teardown_call = builder.public_call.finish();
        let teardown_call_hash = teardown_call.call_stack_item.hash();
        let teardown_is_delegate_call = teardown_call.call_stack_item.public_inputs.call_context.is_delegate_call;
        builder.previous_kernel.push_public_call_request(teardown_call_hash, teardown_is_delegate_call);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(false);

        // Run the kernel on the setup call
        let kernel = PublicKernelSetupCircuitPrivateInputs { previous_kernel, public_call: teardown_call };

        let _ = kernel.public_kernel_setup();
    }

    #[test(should_fail_with="No contract storage update requests are allowed for static calls")]
    fn previous_private_kernel_fails_if_contract_storage_update_requests_on_static_call() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.append_update_requests(1);

        builder.failed();
    }

    #[test]
    fn logs_are_handled_as_expected() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();

        // Logs for the current call stack.
        let unencrypted_logs_hash = 26;
        let unencrypted_log_preimages_length = 50;
        builder.public_call.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

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

        let public_inputs = builder.execute();

        assert_eq(public_inputs.end.encrypted_log_preimages_length, prev_encrypted_log_preimages_length);
        assert_eq(
            public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length
        );

        assert_eq(public_inputs.end.encrypted_logs_hash, prev_encrypted_logs_hash);

        let expected_unencrypted_logs_hash = compute_logs_hash(prev_unencrypted_logs_hash, unencrypted_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
    }

    #[test]
    fn propagate_nullifier_non_existent_read_requests() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        let storage_contract_address = builder.public_call.public_inputs.call_context.storage_contract_address;

        let request_0 = ReadRequest { value: 123, counter: 4567 };
        builder.public_call.public_inputs.nullifier_non_existent_read_requests.push(request_0);
        let request_1 = ReadRequest { value: 777888, counter: 90 };
        builder.public_call.public_inputs.nullifier_non_existent_read_requests.push(request_1);

        let public_inputs = builder.execute();

        let end_requests = public_inputs.validation_requests.nullifier_non_existent_read_requests;
        assert_eq(array_length(end_requests), 2);

        let request_context = end_requests[0];
        assert_eq(request_context.value, request_0.value);
        assert_eq(request_context.counter, request_0.counter);
        assert_eq(request_context.contract_address, storage_contract_address);

        let request_context = end_requests[1];
        assert_eq(request_context.value, request_1.value);
        assert_eq(request_context.counter, request_1.counter);
        assert_eq(request_context.contract_address, storage_contract_address);
    }

    #[test(should_fail_with="Public call cannot be reverted")]
    fn fails_if_public_call_reverted() {
        let mut builder = PublicKernelSetupCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.revert_code = 1;

        builder.failed();
    }
}
