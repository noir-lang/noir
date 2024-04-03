use crate::common;
use dep::types::abis::{
    kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, PublicKernelCircuitPublicInputsBuilder},
    kernel_data::PublicKernelData, public_call_data::PublicCallData
};
use dep::std::unsafe;

struct PublicKernelTeardownCircuitPrivateInputs {
    previous_kernel: PublicKernelData,
    public_call: PublicCallData,
}

impl PublicKernelTeardownCircuitPrivateInputs {
    fn validate_inputs(self) {
        // Currently the nested calls will be pushed to the public call stack and need_setup will return true.
        // This should not be the case when nested calls are handled in avm.
        // But we should also consider merging this and the setup circuit and have one circuit that deals with non-revertibles.
        // let needs_setup = self.previous_kernel.public_inputs.needs_setup();
        // assert(needs_setup == false, "Cannot run teardown circuit before setup circuit");
        let needs_app_logic = self.previous_kernel.public_inputs.needs_app_logic();
        assert(needs_app_logic == false, "Cannot run teardown circuit before app logic circuit");
        let needs_teardown = self.previous_kernel.public_inputs.needs_teardown();
        assert(needs_teardown == true, "Cannot run unnecessary teardown circuit");
    }

    fn public_kernel_teardown(self) -> PublicKernelCircuitPublicInputs {
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

        if public_inputs.revert_code == 0 {
            common::accumulate_unencrypted_logs(
                self.public_call,
                self.previous_kernel.public_inputs.end.unencrypted_logs_hash,
                self.previous_kernel.public_inputs.end.unencrypted_log_preimages_length,
                &mut public_inputs
            );
        }

        public_inputs.finish()
    }
}

mod tests {
    use crate::{
        public_kernel_teardown::PublicKernelTeardownCircuitPrivateInputs,
        utils::{
        assert_eq_call_requests, assert_eq_public_data_reads, assert_eq_public_data_update_requests,
        compute_public_data_reads, compute_public_data_update_requests
    }
    };
    use dep::types::{
        abis::{
        call_request::CallRequest, function_selector::FunctionSelector,
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest
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

    struct PublicKernelTeardownCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        previous_revertible: FixtureBuilder,
        public_call: PublicCallDataBuilder,
    }

    impl PublicKernelTeardownCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new();
            let previous_revertible = FixtureBuilder::new();
            let public_call = PublicCallDataBuilder::new();

            PublicKernelTeardownCircuitPrivateInputsBuilder { previous_kernel, previous_revertible, public_call }
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
            let public_call = self.public_call.finish();
            // Adjust the call stack item hash for the current call in the previous iteration.
            let hash = public_call.call_stack_item.hash();
            let is_delegate_call = public_call.call_stack_item.public_inputs.call_context.is_delegate_call;
            self.previous_kernel.push_public_call_request(hash, is_delegate_call);
            let mut previous_kernel = self.previous_kernel.to_public_kernel_data(false);
            previous_kernel.public_inputs.end = self.previous_revertible.to_public_accumulated_data();

            let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };

            kernel.public_kernel_teardown()
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
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.bytecode_hash = 0;

        builder.failed();
    }

    #[test(should_fail_with="Contract address cannot be zero")]
    fn contract_address_must_be_valid() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.contract_address = AztecAddress::zero();

        builder.failed();
    }

    #[test(should_fail_with="Function signature cannot be zero")]
    fn function_selector_must_be_valid() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.function_data.selector = FunctionSelector::zero();

        builder.failed();
    }

    #[test(should_fail_with="Cannot execute a private function with the public kernel circuit")]
    fn private_call_should_fail() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.function_data.is_private = true;

        builder.failed();
    }

    #[test(should_fail_with="calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the call stack")]
    fn inconsistent_call_hash_should_fail() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        let public_call = builder.public_call.finish();

        let hash = public_call.call_stack_item.hash();
        // Tweak the call stack item hash.
        builder.previous_kernel.push_public_call_request(hash + 1, false);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(false);

        let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };

        let _ = kernel.public_kernel_teardown();
    }

    #[test(should_fail_with="call stack msg_sender does not match caller contract address")]
    fn incorrect_msg_sender_fails_for_regular_calls() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        // Set the msg_sender to a wrong value.
        builder.public_call.public_inputs.call_context.msg_sender.inner += 1;

        builder.failed();
    }

    #[test(should_fail_with="call stack storage address does not match expected contract address")]
    fn incorrect_storage_contract_address_fails_for_regular_calls() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        // Set the storage contract address to a wrong value.
        builder.public_call.public_inputs.call_context.storage_contract_address.inner += 1;

        builder.failed();
    }

    #[test]
    fn delegate_call_succeeds() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();
        builder.succeeded();
    }

    #[test(should_fail_with="caller context cannot be empty for delegate calls")]
    fn empty_caller_context_for_delegate_calls_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();
        let public_call = builder.public_call.finish();

        let hash = public_call.call_stack_item.hash();
        // Caller context is empty for regular calls.
        let is_delegate_call = false;
        builder.previous_kernel.push_public_call_request(hash, is_delegate_call);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(false);

        let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };

        let _ = kernel.public_kernel_teardown();
    }

    #[test(should_fail_with="call stack msg_sender does not match expected msg_sender for delegate calls")]
    fn incorrect_msg_sender_for_delegate_calls_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();

        // Set the msg_sender to be the caller contract.
        builder.public_call.public_inputs.call_context.msg_sender = builder.previous_kernel.contract_address;

        builder.failed();
    }

    #[test(should_fail_with="call stack storage address does not match expected contract address for delegate calls")]
    fn public_kernel_circuit_fails_on_incorrect_storage_contract_in_delegate_call() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();

        // Set the storage contract address to be the contract address.
        builder.public_call.public_inputs.call_context.storage_contract_address = builder.public_call.contract_address;

        builder.failed();
    }

    #[test(should_fail_with="curent contract address must not match storage contract address for delegate calls")]
    fn previous_private_kernel_fails_if_incorrect_storage_contract_on_delegate_call() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();

        builder.public_call.contract_address = builder.public_call.public_inputs.call_context.storage_contract_address;

        builder.failed();
    }

    #[test(should_fail_with = "call requests length does not match the expected length")]
    fn incorrect_call_requests_length_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(2, false);
        // Remove one call request.
        let _ = builder.public_call.public_call_stack.pop();

        builder.failed();
    }

    #[test(should_fail_with = "call stack hash does not match call request hash")]
    fn incorrect_call_request_hash_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(2, false);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the hash to be a different value.
        call_request.hash += 1;
        builder.public_call.public_call_stack.push(call_request);

        builder.failed();
    }

    #[test(should_fail_with = "invalid caller")]
    fn incorrect_caller_address_for_call_request_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(1, false);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the caller contract address to be a different value.
        call_request.caller_contract_address.inner += 1;
        builder.public_call.public_call_stack.push(call_request);

        builder.failed();
    }

    #[test(should_fail_with = "invalid caller")]
    fn incorrect_caller_context_for_delegate_call_request_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(1, true);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the storage contract to be a different value.
        call_request.caller_context.storage_contract_address.inner += 1;
        builder.public_call.public_call_stack.push(call_request);

        builder.failed();
    }

    #[test]
    fn public_kernel_circuit_teardown_with_no_setup_and_no_app_logic_succeeds() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.succeeded();
    }

    #[test(should_fail_with="Cannot run unnecessary teardown circuit")]
    fn private_previous_kernel_non_empty_private_call_stack_should_fail() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        let public_call = builder.public_call.finish();
        // Don't push a call for teardown
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(false);

        let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };

        let _ = kernel.public_kernel_teardown();
    }

    #[test(should_fail_with="No contract storage update requests are allowed for static calls")]
    fn previous_private_kernel_fails_if_contract_storage_update_requests_on_static_call() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.append_update_requests(1);

        builder.failed();
    }

    #[test]
    fn logs_are_handled_as_expected() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

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

    #[test(should_fail_with="Public call cannot be reverted")]
    fn fails_if_public_call_reverted() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.revert_code = 1;

        builder.failed();
    }
}
