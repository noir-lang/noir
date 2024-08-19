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
    constants::{
    PUBLIC_KERNEL_SETUP_INDEX, PUBLIC_KERNEL_APP_LOGIC_INDEX, PUBLIC_KERNEL_TEARDOWN_INDEX,
    PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX
}
};

struct PublicKernelTeardownCircuitPrivateInputs {
    previous_kernel: PublicKernelData,
    public_call: PublicCallData,
}

global ALLOWED_PREVIOUS_CIRCUITS = [
    PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX,
    PUBLIC_KERNEL_SETUP_INDEX,
    PUBLIC_KERNEL_APP_LOGIC_INDEX,
    PUBLIC_KERNEL_TEARDOWN_INDEX,
];

impl PublicKernelTeardownCircuitPrivateInputs {
    fn execute(self) -> PublicKernelCircuitPublicInputs {
        let phase = PublicKernelPhase.TEARDOWN;

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
    use crate::{public_kernel_teardown::{ALLOWED_PREVIOUS_CIRCUITS, PublicKernelTeardownCircuitPrivateInputs}};
    use dep::types::{
        abis::{
        function_selector::FunctionSelector, gas::Gas,
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs,
        public_call_stack_item_compressed::PublicCallStackItemCompressed
    },
        address::AztecAddress, constants::{PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX, BASE_ROLLUP_INDEX},
        tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq}, traits::is_empty
    };

    struct PublicKernelTeardownCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        public_call: FixtureBuilder,
    }

    impl PublicKernelTeardownCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new().as_parent_contract().in_vk_tree(PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX);
            let mut public_call = FixtureBuilder::new().is_public_function();
            public_call.value_offset = 9999; // Add an offset so that the mock data won't be the same as the values in previous_kernel.

            PublicKernelTeardownCircuitPrivateInputsBuilder { previous_kernel, public_call }
        }

        pub fn is_delegate_call(&mut self) -> Self {
            let _ = self.public_call.is_delegate_call();
            *self
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            let public_call = self.public_call.to_public_call_data();
            // Adjust the call stack item hash for the current call in the previous iteration.
            let compressed = public_call.call_stack_item.get_compressed();
            self.previous_kernel.push_public_teardown_call_request(compressed);
            let mut previous_kernel = self.previous_kernel.to_public_kernel_data(true);

            let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };

            kernel.execute()
        }

        pub fn get_call_request_item(self) -> PublicCallStackItemCompressed {
            self.public_call.to_public_call_stack_item().get_compressed()
        }

        pub fn failed_with_call_request_item(&mut self, item: PublicCallStackItemCompressed) {
            let public_call = self.public_call.to_public_call_data();
            self.previous_kernel.push_public_teardown_call_request(item);
            let previous_kernel = self.previous_kernel.to_public_kernel_data(true);
            let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };
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
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.bytecode_hash = 0;

        builder.failed();
    }

    #[test(should_fail_with="Contract address cannot be zero")]
    fn contract_address_must_be_valid() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.append_public_teardown_call_requests(1);
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

    #[test(should_fail_with="call stack item does not match item at the top of the call stack")]
    fn inconsistent_call_request_item_should_fail() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        let mut item = builder.get_call_request_item();
        // Tweak the call stack item.
        item.args_hash += 1;

        builder.failed_with_call_request_item(item);
    }

    #[test(should_fail_with="call stack item does not match item at the top of the call stack")]
    fn incorrect_msg_sender_fails_for_regular_calls() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        let mut item = builder.get_call_request_item();
        // Set the msg_sender to a wrong value.
        item.call_context.msg_sender.inner += 1;

        builder.failed_with_call_request_item(item);
    }

    #[test(should_fail_with="call stack storage address does not match expected contract address")]
    fn incorrect_storage_contract_address_fails_for_regular_calls() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        // Set the storage contract address to a wrong value.
        builder.public_call.storage_contract_address.inner += 1;

        builder.failed();
    }

    #[test]
    fn delegate_call_succeeds() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();
        builder.succeeded();
    }

    #[test(should_fail_with="call stack item does not match item at the top of the call stack")]
    fn incorrect_msg_sender_for_delegate_calls_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();

        let mut item = builder.get_call_request_item();
        // Set the msg_sender to be the caller contract.
        item.call_context.msg_sender = builder.previous_kernel.contract_address;

        builder.failed_with_call_request_item(item);
    }

    #[test(should_fail_with="call stack item does not match item at the top of the call stack")]
    fn public_kernel_circuit_fails_on_incorrect_storage_contract_in_delegate_call() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();

        let mut item = builder.get_call_request_item();
        // Set the storage contract address to be the contract address.
        item.call_context.storage_contract_address = builder.public_call.contract_address;

        builder.failed_with_call_request_item(item);
    }

    #[test(should_fail_with="curent contract address must not match storage contract address for delegate calls")]
    fn previous_private_kernel_fails_if_incorrect_storage_contract_on_delegate_call() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new().is_delegate_call();

        builder.public_call.contract_address = builder.public_call.storage_contract_address;

        builder.failed();
    }

    #[test(should_fail_with="incorrect storage_contract_address for call request")]
    fn incorrect_storage_contract_address_for_call_request_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(1);
        let mut call_request = builder.public_call.public_call_requests.pop();
        // Change the caller contract address to be a different value.
        call_request.item.call_context.storage_contract_address.inner += 1;
        builder.public_call.public_call_requests.push(call_request);

        builder.failed();
    }

    #[test(should_fail_with="incorrect storage_contract_address for delegate call request")]
    fn incorrect_call_context_for_delegate_call_request_fails() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests_delegate(1);
        let mut call_request = builder.public_call.public_call_requests.pop();
        // Change the storage contract address to be the target contract address.
        call_request.item.call_context.storage_contract_address = call_request.item.contract_address;
        builder.public_call.public_call_requests.push(call_request);

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

        let public_call = builder.public_call.to_public_call_data();
        // Don't push a call for teardown
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);

        let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };

        let _ = kernel.execute();
    }

    #[test(should_fail_with="No contract storage update requests are allowed for static calls")]
    fn previous_private_kernel_fails_if_contract_storage_update_requests_on_static_call() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.is_static_call = true;
        builder.public_call.append_contract_storage_update_requests(1);

        builder.failed();
    }

    #[test]
    fn logs_are_handled_as_expected() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        // Logs for the current call stack.
        builder.public_call.append_unencrypted_log_hashes(1);
        let curr_data = builder.public_call.to_public_accumulated_data();

        // Logs for the previous call stack.
        builder.previous_kernel.append_encrypted_log_hashes(1);
        builder.previous_kernel.append_unencrypted_log_hashes(1);
        let prev_data = builder.previous_kernel.to_public_accumulated_data();

        let expected_unencrypted_logs = [prev_data.unencrypted_logs_hashes[0], curr_data.unencrypted_logs_hashes[0]];
        // we assume the encrypted log is already siloed from private kernels
        let expected_encrypted_logs = [prev_data.encrypted_logs_hashes[0]];

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.end.encrypted_logs_hashes,
            expected_encrypted_logs
        );
        assert_array_eq(
            public_inputs.end.unencrypted_logs_hashes,
            expected_unencrypted_logs
        );
    }

    #[test(should_fail_with="Start gas for teardown phase does not match teardown gas allocation")]
    fn validates_start_gas() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.is_execution_request = true; // don't need to check start gas for nested calls
        builder.public_call.start_gas_left = Gas::new(10, 30);

        builder.failed();
    }

    #[test(should_fail_with="Transaction fee on teardown phase does not match expected value")]
    fn validates_transaction_fee() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.transaction_fee = 1234;

        builder.failed();
    }

    #[test]
    fn propagates_global_variables_if_empty() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.global_variables.block_number = 11;

        let public_inputs = builder.execute();

        assert_eq(public_inputs.constants.global_variables.block_number, 11);
    }

    #[test(should_fail_with="Global variables injected into the public call do not match constants")]
    fn validates_global_variables() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.global_variables.block_number = 10;
        builder.public_call.global_variables.block_number = 11;

        builder.failed();
    }

    #[test]
    unconstrained fn propagate_fee_payer() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        let fee_payer = AztecAddress::from_field(123);
        builder.previous_kernel.set_fee_payer(fee_payer);
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer is not set if is_fee_payer is false
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        assert_eq(is_empty(builder.previous_kernel.fee_payer), true);
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());
    }

    #[test]
    fn clears_stack_if_current_call_revert_code_is_set() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.revert_code = 1;

        builder.previous_kernel.append_public_teardown_call_requests(2);

        let public_inputs = builder.execute();

        assert_array_eq(public_inputs.public_teardown_call_stack, []);
    }

    #[test]
    fn retains_stack_if_fail_in_app_logic() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.previous_kernel.revert_code = 1;

        builder.previous_kernel.append_public_teardown_call_requests(2);
        let teardown_calls = builder.previous_kernel.public_teardown_call_stack.storage;

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.public_teardown_call_stack,
            [teardown_calls[0], teardown_calls[1]]
        );
    }

    #[test]
    unconstrained fn correctly_updates_revert_code_0() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        let public_inputs = builder.execute();
        assert_eq(public_inputs.revert_code, 0);
    }

    #[test]
    unconstrained fn correctly_updates_revert_code_1() {
        // Case where we carry forward a revert code from app logic
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.previous_kernel.revert_code = 1;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.revert_code, 1);
    }

    #[test]
    unconstrained fn correctly_updates_revert_code_2() {
        // Case where there is a new error in teardown
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.revert_code = 1;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.revert_code, 2);
    }

    #[test]
    unconstrained fn correctly_updates_revert_code_3() {
        // Case where there is an error in both app logic and teardown
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.previous_kernel.revert_code = 1;
        builder.public_call.revert_code = 1;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.revert_code, 3);
    }

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
            builder.previous_kernel = builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_kernel() {
    //     let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
    //     builder.previous_kernel = builder.previous_kernel.in_vk_tree(BASE_ROLLUP_INDEX);
    //     let _res = builder.execute();
    // }
}
