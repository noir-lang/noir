use crate::common;
use dep::types::abis::{
    kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, PublicKernelCircuitPublicInputsBuilder},
    public_kernel_data::PublicKernelData, public_call_data::PublicCallData, gas_fees::GasFees
};
use dep::types::utils::arrays::array_to_bounded_vec;
use dep::types::constants::{
    PUBLIC_KERNEL_SETUP_INDEX, PUBLIC_KERNEL_APP_LOGIC_INDEX, PUBLIC_KERNEL_TEARDOWN_INDEX,
    PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX
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
    fn initialize_revert_code(self, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
        // See https://docs.aztec.network/protocol-specs/gas-and-fees/kernel-tracking#handling-reverts
        let previous_kernel_revert_code = self.previous_kernel.public_inputs.revert_code;
        let public_call_revert_code = self.public_call.call_stack_item.public_inputs.revert_code;
        if previous_kernel_revert_code.eq(0) & public_call_revert_code.eq(0) {
            circuit_outputs.revert_code = 0;
        } else if previous_kernel_revert_code.eq(1) & public_call_revert_code.eq(0) {
            // Case where we carry forward a revert code from app logic
            circuit_outputs.revert_code = 1;
        } else if previous_kernel_revert_code.eq(0) & !public_call_revert_code.eq(0) {
            // Case where there is a new error in teardown
            circuit_outputs.revert_code = 2;
        } else if previous_kernel_revert_code.eq(1) & !public_call_revert_code.eq(0) {
            // Case where there is an error in both app logic and teardown
            circuit_outputs.revert_code = 3;
        }
    }

    fn validate_inputs(self) {
        // Currently the nested calls will be pushed to the public call stack and need_setup will return true.
        // This should not be the case when nested calls are handled in avm.
        // But we should also consider merging this and the setup circuit and have one circuit that deals with non-revertibles.
        let needs_setup = self.previous_kernel.public_inputs.needs_setup();
        assert(needs_setup == false, "Cannot run teardown circuit before setup circuit");
        let needs_app_logic = self.previous_kernel.public_inputs.needs_app_logic();
        assert(needs_app_logic == false, "Cannot run teardown circuit before app logic circuit");
        let needs_teardown = self.previous_kernel.public_inputs.needs_teardown();
        assert(needs_teardown == true, "Cannot run unnecessary teardown circuit");
    }

    // Validates that the start gas injected into the app circuit matches the teardown gas limits set by the user
    fn validate_start_gas(self) {
        // If this is a nested call (not an execution request), the start gas is correct as long as the
        // call being processed by this kernel iteration matches the call at the top of the callstack
        // as per the previous kernel's outputs.
        // An execution request's start gas is the remaining gas left in the transaction after the previous kernel.
        // A nested call's start gas is the gas allocated to it by its caller and placed in the callstack.
        if (self.public_call.call_stack_item.is_execution_request) {
            let public_call_start_gas = self.public_call.call_stack_item.public_inputs.start_gas_left;
            let teardown_gas_limit = self.previous_kernel.public_inputs.constants.tx_context.gas_settings.teardown_gas_limits;
            assert(
                public_call_start_gas == teardown_gas_limit, "Start gas for teardown phase does not match teardown gas allocation"
            );
        }
    }

    // Validates the transaction fee injected into the app circuit is properly computed from gas_used and block gas_fees
    fn validate_transaction_fee(self, public_inputs: PublicKernelCircuitPublicInputsBuilder) {
        let transaction_fee = self.public_call.call_stack_item.public_inputs.transaction_fee;
        // Note that teardown_gas is already included in end.gas_used as it was injected by the private kernel
        let total_gas_used = self.previous_kernel.public_inputs.end.gas_used
            + self.previous_kernel.public_inputs.end_non_revertible.gas_used;
        let block_gas_fees = public_inputs.constants.global_variables.gas_fees;
        let inclusion_fee = self.previous_kernel.public_inputs.constants.tx_context.gas_settings.inclusion_fee;
        let computed_transaction_fee = total_gas_used.compute_fee(block_gas_fees) + inclusion_fee;

        // dep::types::debug_log::debug_log_format(
        //     "Validating tx fee: end.gas_used.da={0} end.gas_used.l2={1} non_revertible.gas_used.da={2} non_revertible.gas_used.l2={3} block_fee_per_gas.da={4}  block_fee_per_gas.l2={5} inclusion_fee={6} computed={7} actual={8}",
        //     [
        //     self.previous_kernel.public_inputs.end.gas_used.da_gas as Field,
        //     self.previous_kernel.public_inputs.end.gas_used.l2_gas as Field,
        //     self.previous_kernel.public_inputs.end_non_revertible.gas_used.da_gas as Field,
        //     self.previous_kernel.public_inputs.end_non_revertible.gas_used.l2_gas as Field,
        //     block_gas_fees.fee_per_da_gas as Field,
        //     block_gas_fees.fee_per_l2_gas as Field,
        //     inclusion_fee,
        //     computed_transaction_fee,
        //     transaction_fee
        // ]
        // );

        assert(
            transaction_fee == computed_transaction_fee, "Transaction fee on teardown phase does not match expected value"
        );
    }

    fn initialize_end_values(self, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
        common::initialize_end_values(self.previous_kernel, circuit_outputs);
        // unlike app logic, only pass forward the teardown call stack if there is no new error
        circuit_outputs.public_teardown_call_stack = array_to_bounded_vec(self.previous_kernel.public_inputs.public_teardown_call_stack);
    }

    fn public_kernel_teardown(self) -> PublicKernelCircuitPublicInputs {
        if !dep::std::runtime::is_unconstrained() {
            // verify the previous kernel proof
            self.previous_kernel.verify();
            // TODO(#7410) currently stubbed out until tube vk handled
            // self.previous_kernel.validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }

        // construct the circuit outputs
        let mut public_inputs = PublicKernelCircuitPublicInputsBuilder::empty();
        self.initialize_revert_code(&mut public_inputs);

        // initialise the end state with our provided previous kernel state
        common::initialize_non_revertible_values(self.previous_kernel, &mut public_inputs);

        // validate the inputs common to all invocation circumstances
        common::validate_inputs(self.public_call);

        // validate constants injected into the public call are correct or set them if this is the first public call
        common::initialize_from_or_validate_public_call_variables(self.previous_kernel, self.public_call, &mut public_inputs);

        // validate the inputs unique to having a previous private kernel
        self.validate_inputs();

        self.validate_start_gas();
        self.validate_transaction_fee(public_inputs);

        common::update_validation_requests(self.public_call, &mut public_inputs);

        // there was no revert, or a revert in app logic
        if public_inputs.revert_code.eq(0) | public_inputs.revert_code.eq(1) {
            self.initialize_end_values(&mut public_inputs);
            // Pops the item from the call stack and validates it against the current execution.
            let call_request = public_inputs.public_teardown_call_stack.pop();
            common::validate_call_against_request(self.public_call, call_request);
            common::update_teardown_call_stack(self.public_call, &mut public_inputs);
            common::update_public_end_values(self.public_call, &mut public_inputs);
        } else {
            let mut remaining_calls = array_to_bounded_vec(self.previous_kernel.public_inputs.public_teardown_call_stack);
            let reverted_call_request = remaining_calls.pop();

            assert(
                reverted_call_request.hash == self.public_call.call_stack_item.get_compressed().hash(), "calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the teardown call stack"
            );
        }

        public_inputs.finish()
    }
}

mod tests {
    use crate::{
        public_kernel_teardown::{ALLOWED_PREVIOUS_CIRCUITS, PublicKernelTeardownCircuitPrivateInputs},
        utils::{
        assert_eq_call_requests, assert_eq_public_data_reads, assert_eq_public_data_update_requests,
        compute_public_data_reads, compute_public_data_update_requests
    }
    };
    use dep::types::{
        abis::{
        call_request::CallRequest, function_selector::FunctionSelector, gas::Gas,
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest
    },
        address::{AztecAddress, EthAddress}, contract_class_id::ContractClassId,
        contrakt::storage_read::StorageRead, hash::compute_siloed_unencrypted_log_hash,
        tests::{fixture_builder::FixtureBuilder, public_call_data_builder::PublicCallDataBuilder},
        utils::{arrays::{array_eq, array_length}}, traits::is_empty
    };
    use dep::types::constants::{
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_READS_PER_CALL,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX, BASE_ROLLUP_INDEX
    };

    struct PublicKernelTeardownCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        previous_revertible: FixtureBuilder,
        public_call: PublicCallDataBuilder,
    }

    impl PublicKernelTeardownCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new().as_parent_contract().in_vk_tree(PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX);
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
            let hash = public_call.call_stack_item.get_compressed().hash();
            let is_delegate_call = public_call.call_stack_item.public_inputs.call_context.is_delegate_call;
            self.previous_kernel.push_public_teardown_call_request(hash, is_delegate_call);
            let mut previous_kernel = self.previous_kernel.to_public_kernel_data(true);

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

        let hash = public_call.call_stack_item.get_compressed().hash();
        // Tweak the call stack item hash.
        builder.previous_kernel.push_public_teardown_call_request(hash + 1, false);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);

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

        let hash = public_call.call_stack_item.get_compressed().hash();
        // Caller context is empty for regular calls.
        let is_delegate_call = false;
        builder.previous_kernel.push_public_teardown_call_request(hash, is_delegate_call);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);

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
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);

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
        builder.previous_kernel.add_encrypted_log_hash(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.add_unencrypted_log_hash(
            prev_unencrypted_logs_hash,
            prev_unencrypted_log_preimages_length
        );
        let prev_data = builder.previous_kernel.to_public_accumulated_data();

        let mut expected_unencrypted_logs = [
            prev_data.unencrypted_logs_hashes[0], builder.public_call.public_inputs.unencrypted_logs_hashes.storage[0]
        ];
        // silo the new log hash
        expected_unencrypted_logs[1].value = compute_siloed_unencrypted_log_hash(builder.public_call.contract_address, expected_unencrypted_logs[1].value);
        // we assume the encrypted log is already siloed from private kernels
        let expected_encrypted_logs = [prev_data.encrypted_logs_hashes[0]];

        let public_inputs = builder.execute();

        assert(
            array_eq(
                public_inputs.end.encrypted_logs_hashes,
                expected_encrypted_logs
            )
        );
        assert(
            array_eq(
                public_inputs.end.unencrypted_logs_hashes,
                expected_unencrypted_logs
            )
        );
    }

    #[test(should_fail_with="Start gas for teardown phase does not match teardown gas allocation")]
    fn validates_start_gas() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.is_execution_request = true; // don't need to check start gas for nested calls
        builder.public_call.public_inputs.start_gas_left = Gas::new(10, 30);

        builder.failed();
    }

    #[test(should_fail_with="Transaction fee on teardown phase does not match expected value")]
    fn validates_transaction_fee() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.transaction_fee = 1234;

        builder.failed();
    }

    #[test]
    fn propagates_global_variables_if_empty() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.public_call.public_inputs.global_variables.block_number = 11;

        let public_inputs = builder.execute();

        assert_eq(public_inputs.constants.global_variables.block_number, 11);
    }

    #[test(should_fail_with="Global variables injected into the public call do not match constants")]
    fn validates_global_variables() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.global_variables.block_number = 10;
        builder.public_call.public_inputs.global_variables.block_number = 11;

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
        builder.public_call.public_inputs.revert_code = 1;
        let public_call = builder.public_call.finish();
        let hash = public_call.call_stack_item.get_compressed().hash();
        builder.previous_kernel.push_public_teardown_call_request(hash, false);
        // push again to check that the stack is cleared
        builder.previous_kernel.push_public_teardown_call_request(hash, false);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);

        let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };

        let public_inputs = kernel.public_kernel_teardown();

        assert_eq(array_length(public_inputs.public_teardown_call_stack), 0);
    }

    #[test]
    fn retains_stack_if_fail_in_app_logic() {
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.previous_kernel.revert_code = 1;
        let public_call = builder.public_call.finish();
        let hash = public_call.call_stack_item.get_compressed().hash();
        builder.previous_kernel.push_public_teardown_call_request(hash, false);
        // push again to check that we keep one item after popping the current call
        builder.previous_kernel.push_public_teardown_call_request(hash, false);
        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);

        let kernel = PublicKernelTeardownCircuitPrivateInputs { previous_kernel, public_call };

        let public_inputs = kernel.public_kernel_teardown();

        assert_eq(array_length(public_inputs.public_teardown_call_stack), 1);
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
        builder.public_call.public_inputs.revert_code = 1;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.revert_code, 2);
    }

    #[test]
    unconstrained fn correctly_updates_revert_code_3() {
        // Case where there is an error in both app logic and teardown
        let mut builder = PublicKernelTeardownCircuitPrivateInputsBuilder::new();
        builder.previous_kernel.revert_code = 1;
        builder.public_call.public_inputs.revert_code = 1;
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
