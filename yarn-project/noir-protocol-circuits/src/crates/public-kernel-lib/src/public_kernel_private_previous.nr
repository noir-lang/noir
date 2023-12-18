use crate::common;
use dep::types::abis::{
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, KernelCircuitPublicInputsBuilder},
    previous_kernel_data::PreviousKernelData,
    public_call_data::PublicCallData,
};
use dep::std::unsafe;
// translated from cpp impl in 
// aztec-packages/circuits/cpp/src/aztec3/circuits/kernel/public/native_public_kernel_circuit_private_previous_kernel.cpp

struct PublicKernelPrivatePreviousInputs {
    previous_kernel: PreviousKernelData,
    public_call: PublicCallData,
}

impl PublicKernelPrivatePreviousInputs {
    fn validate_inputs(self) {
        let private_call_stack = self.previous_kernel.public_inputs.end.private_call_stack;
        for i in 0..private_call_stack.len() {
            let private_call = private_call_stack[i];
            assert(private_call.is_empty(),
                "Private call stack must be empty when executing in the public kernel");
        }

        let previous_call_is_private = self.previous_kernel.public_inputs.is_private;
        assert(previous_call_is_private == true,
            "Previous kernel must be private when in this public kernel version");
    }

    fn public_kernel_private_previous(self) -> KernelCircuitPublicInputs {
        // construct the circuit outputs
        let mut public_inputs: KernelCircuitPublicInputsBuilder = unsafe::zeroed();

        // initialise the end state with our provided previous kernel state
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        // validate the inputs common to all invocation circumstances
        common::validate_inputs(self.public_call);

        // validate the inputs unique to having a previous private kernel
        self.validate_inputs();

        // Pops the item from the call stack and validates it against the current execution.
        let call_request = public_inputs.end.public_call_stack.pop();
        common::validate_call_against_request(self.public_call, call_request);

        common::update_public_end_values(self.public_call,&mut public_inputs);

        common::accumulate_unencrypted_logs(self.public_call, self.previous_kernel,&mut public_inputs);

        public_inputs.finish()
    }
}

mod tests {
    use crate::{
        public_kernel_private_previous::PublicKernelPrivatePreviousInputs,
        utils::{
            assert_eq_call_requests,
            assert_eq_new_contracts,
            assert_eq_public_data_reads,
            assert_eq_public_data_update_requests,
            compute_public_data_reads,
            compute_public_data_update_requests,
        },
    };
    use dep::types::{
        abis::{
            call_request::CallRequest,
            function_selector::FunctionSelector,
            kernel_circuit_public_inputs::KernelCircuitPublicInputs,
            new_contract_data::NewContractData,
            public_data_read::PublicDataRead,
            public_data_update_request::PublicDataUpdateRequest,
        },
        address::{AztecAddress, EthAddress},
        contrakt::storage_read::StorageRead,
        hash::compute_logs_hash,
        tests::{
            previous_kernel_data_builder::PreviousKernelDataBuilder,
            public_call_data_builder::PublicCallDataBuilder,
        },
        utils::{
            arrays::{
                array_eq,
                array_length,
            },
            bounded_vec::BoundedVec,
        },
    };
    use dep::types::constants::{
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_DATA_READS_PER_CALL,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    };

    struct PublicKernelPrivatePreviousInputsBuilder {
        previous_kernel: PreviousKernelDataBuilder,
        public_call: PublicCallDataBuilder,
    }

    impl PublicKernelPrivatePreviousInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = PreviousKernelDataBuilder::new();
            let public_call = PublicCallDataBuilder::new();
            
            PublicKernelPrivatePreviousInputsBuilder {
                previous_kernel,
                public_call,
            }
        }

        pub fn is_delegate_call(&mut self) -> Self {
            let _ = self.public_call.is_delegate_call();
            *self
        }

        pub fn is_internal_call(&mut self) -> Self {
            let _ = self.public_call.is_internal_call();
            self.previous_kernel.contract_address = self.public_call.contract_address;
            *self
        }

        pub fn get_current_public_data_reads(self) -> [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_CALL] {
            let read_requests = self.public_call.public_inputs.contract_storage_reads.storage;
            compute_public_data_reads(self.public_call.contract_address, read_requests)
        }

        pub fn get_current_public_data_update_requests(self) -> [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL] {
            compute_public_data_update_requests(
                self.public_call.contract_address,
                self.public_call.public_inputs.contract_storage_update_requests.storage,
            )
        }

        pub fn execute(&mut self) -> KernelCircuitPublicInputs {
            let public_call = self.public_call.finish();
            // Adjust the call stack item hash for the current call in the previous iteration.
            let hash = public_call.call_stack_item.hash();
            let is_delegate_call = public_call.call_stack_item.public_inputs.call_context.is_delegate_call;
            self.previous_kernel.push_public_call_request(hash, is_delegate_call);
            let previous_kernel = self.previous_kernel.finish();

            let kernel = PublicKernelPrivatePreviousInputs {
                previous_kernel,
                public_call,
            };

            kernel.public_kernel_private_previous()
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
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.append_read_requests(1);
        builder.public_call.append_empty_read_requests(1);
        builder.public_call.append_read_requests(1);
        let public_data_reads = builder.get_current_public_data_reads();

        let public_inputs = builder.execute();

        // Only the 2 valid reads should have been propagated.
        let expected = [public_data_reads[0], public_data_reads[2]];
        assert_eq_public_data_reads(public_inputs.end.public_data_reads, expected);
    }

    #[test]
    fn only_valid_update_requests_should_be_propagated() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.append_update_requests(1);
        builder.public_call.append_empty_update_requests(1);
        builder.public_call.append_update_requests(1);
        let public_data_update_requests = builder.get_current_public_data_update_requests();

        let public_inputs = builder.execute();

        // Only the 2 valid reads should have been propagated.
        let expected = [public_data_update_requests[0], public_data_update_requests[2]];
        assert_eq_public_data_update_requests(public_inputs.end.public_data_update_requests, expected);
    }

    #[test(should_fail_with="Constructors cannot be public functions")]
    fn constructor_should_fail() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.function_data.is_constructor = true;

        builder.failed();
    }

    #[test(should_fail_with="Contract deployment cannot be a public function")]
    fn constructor_should_fail_2() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_contract_deployment = true;

        builder.failed();
    }

    #[test(should_fail_with="Bytecode hash cannot be zero")]
    fn no_bytecode_hash_should_fail() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.bytecode_hash = 0;

        builder.failed();
    }

    #[test]
    fn internal_call_succeeds() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new().is_internal_call();
        builder.succeeded();
    }

    #[test(should_fail_with="msg_sender must be self for internal calls")]
    fn invalid_is_internal() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new().is_internal_call();

        // Tweak the (storage) contract_address to be different to msg_sender.
        let msg_sender = builder.public_call.public_inputs.call_context.msg_sender.to_field();
        builder.public_call.contract_address = AztecAddress::from_field(msg_sender + 1);
        builder.public_call.public_inputs.call_context.storage_contract_address = AztecAddress::from_field(msg_sender + 1);

        builder.failed();
    }

    #[test(should_fail_with="Contract address cannot be zero")]
    fn contract_address_must_be_valid() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.contract_address = AztecAddress::zero();

        builder.failed();
    }

    #[test(should_fail_with="Function signature cannot be zero")]
    fn function_selector_must_be_valid() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.function_data.selector = FunctionSelector::zero();

        builder.failed();
    }

    #[test(should_fail_with="Cannot execute a private function with the public kernel circuit")]
    fn private_call_should_fail() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.function_data.is_private = true;

        builder.failed();
    }

    #[test(should_fail_with="calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the call stack")]
    fn inconsistent_call_hash_should_fail() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        let public_call = builder.public_call.finish();

        let hash = public_call.call_stack_item.hash();
        // Tweak the call stack item hash.
        builder.previous_kernel.push_public_call_request(hash + 1, false);
        let previous_kernel = builder.previous_kernel.finish();

        let kernel = PublicKernelPrivatePreviousInputs {
            previous_kernel,
            public_call,
        };

        let _ = kernel.public_kernel_private_previous();
    }

    #[test(should_fail_with="call stack msg_sender does not match caller contract address")]
    fn incorrect_msg_sender_fails_for_regular_calls() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();

        // Set the msg_sender to a wrong value.
        builder.public_call.public_inputs.call_context.msg_sender.inner += 1;

        builder.failed();
    }

    #[test(should_fail_with="call stack storage address does not match expected contract address")]
    fn incorrect_storage_contract_address_fails_for_regular_calls() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();

        // Set the storage contract address to a wrong value.
        builder.public_call.public_inputs.call_context.storage_contract_address.inner += 1;

        builder.failed();
    }

    #[test]
    fn delegate_call_succeeds() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new().is_delegate_call();
        builder.succeeded();
    }

    #[test(should_fail_with="caller context cannot be empty for delegate calls")]
    fn empty_caller_context_for_delegate_calls_fails() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new().is_delegate_call();
        let public_call = builder.public_call.finish();

        let hash = public_call.call_stack_item.hash();
        // Caller context is empty for regular calls.
        let is_delegate_call = false;
        builder.previous_kernel.push_public_call_request(hash, is_delegate_call);
        let previous_kernel = builder.previous_kernel.finish();

        let kernel = PublicKernelPrivatePreviousInputs {
            previous_kernel,
            public_call,
        };

        let _ = kernel.public_kernel_private_previous();
    }

    #[test(should_fail_with="call stack msg_sender does not match expected msg_sender for delegate calls")]
    fn incorrect_msg_sender_for_delegate_calls_fails() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new().is_delegate_call();

        // Set the msg_sender to be the caller contract.
        builder.public_call.public_inputs.call_context.msg_sender = builder.previous_kernel.contract_address;

        builder.failed();
    }

    #[test(should_fail_with="call stack storage address does not match expected contract address for delegate calls")]
    fn public_kernel_circuit_fails_on_incorrect_storage_contract_in_delegate_call() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new().is_delegate_call();

        // Set the storage contract address to be the contract address.
        builder.public_call.public_inputs.call_context.storage_contract_address = builder.public_call.contract_address;

        builder.failed();
    }

    #[test(should_fail_with="curent contract address must not match storage contract address for delegate calls")]
    fn previous_private_kernel_fails_if_incorrect_storage_contract_on_delegate_call() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new().is_delegate_call();

        builder.public_call.contract_address = builder.public_call.public_inputs.call_context.storage_contract_address;

        builder.failed();
    }

    #[test(should_fail_with = "call requests length does not match the expected length")]
    fn incorrect_call_requests_length_fails() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();

        builder.public_call.append_public_call_requests(2, false);
        // Remove one call request.
        let _ = builder.public_call.public_call_stack.pop();

        builder.failed();
    }

    #[test(should_fail_with = "call stack hash does not match call request hash")]
    fn incorrect_call_request_hash_fails() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();

        builder.public_call.append_public_call_requests(2, false);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the hash to be a different value.
        call_request.hash += 1;
        builder.public_call.public_call_stack.push(call_request);

        builder.failed();
    }

    #[test(should_fail_with = "invalid caller")]
    fn incorrect_caller_address_for_call_request_fails() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();

        builder.public_call.append_public_call_requests(1, false);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the caller contract address to be a different value.
        call_request.caller_contract_address.inner += 1;
        builder.public_call.public_call_stack.push(call_request);

        builder.failed();
    }

    #[test(should_fail_with = "invalid caller")]
    fn incorrect_caller_context_for_delegate_call_request_fails() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();

        builder.public_call.append_public_call_requests(1, true);
        let mut call_request = builder.public_call.public_call_stack.pop();
        // Change the storage contract to be a different value.
        call_request.caller_context.storage_contract_address.inner += 1;
        builder.public_call.public_call_stack.push(call_request);

        builder.failed();
    }

    #[test]
    fn public_kernel_circuit_with_private_previous_kernel_should_succeed() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.succeeded();
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_private_kernel() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        let new_contracts = [
            NewContractData {
                contract_address: AztecAddress::from_field(123),
                portal_contract_address: EthAddress::from_field(456),
                function_tree_root: 78,
            },
        ];
        builder.previous_kernel.end.new_contracts.push_array(new_contracts);

        builder.public_call.append_public_call_requests_for_regular_calls(2);
        let storage = builder.public_call.public_call_stack.storage;
        let public_call_stack = [storage[0], storage[1]];

        builder.public_call.append_update_requests(2);
        let storage = builder.get_current_public_data_update_requests();
        let update_requests = [storage[0], storage[1]];

        builder.public_call.append_read_requests(3);
        let storage = builder.get_current_public_data_reads();
        let read_requests = [storage[0], storage[1], storage[2]];

        let public_inputs = builder.execute();

        assert_eq_new_contracts(public_inputs.end.new_contracts, new_contracts);
        assert_eq_call_requests(public_inputs.end.private_call_stack, []);
        assert_eq_call_requests(public_inputs.end.public_call_stack, public_call_stack);
        assert_eq_public_data_update_requests(public_inputs.end.public_data_update_requests, update_requests);
        assert_eq_public_data_reads(public_inputs.end.public_data_reads, read_requests);
    }

    #[test(should_fail_with="Private call stack must be empty when executing in the public kernel")]
    fn private_previous_kernel_non_empty_private_call_stack_should_fail() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.previous_kernel.push_private_call_request(1, false);

        builder.failed();
    }

    // TODO: Find another way to test this. Currently it will crash because we are popping from an empty array:
    // The application panicked (crashed). Message:  Expected array index to fit in u64
    // #[test(should_fail_with="Public call stack can not be empty")]
    // fn private_previous_kernel_empty_public_call_stack_should_fail() {
    //     let builder = PublicKernelPrivatePreviousInputsBuilder::new();
    //     let previous_kernel = builder.previous_kernel.finish();
    //     let kernel = PublicKernelPrivatePreviousInputs {
    //         previous_kernel,
    //         public_call: builder.public_call.finish(),
    //     };

    //     let _ = kernel.public_kernel_private_previous();
    // }

    #[test(should_fail_with="Previous kernel must be private when in this public kernel version")]
    fn private_previous_kernel_non_private_previous_kernel_should_fail() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.previous_kernel.is_private = false;

        builder.failed();
    }

    #[test(should_fail_with="No contract storage update requests are allowed for static calls")]
    fn previous_private_kernel_fails_if_contract_storage_update_requests_on_static_call() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.append_update_requests(1);

        builder.failed();
    }

    #[test]
    fn logs_are_handled_as_expected() {
        let mut builder = PublicKernelPrivatePreviousInputsBuilder::new();

        // Logs for the current call stack.
        let unencrypted_logs_hash = [26, 47];
        let unencrypted_log_preimages_length = 50;
        builder.public_call.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = [80, 429];
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = [956, 112];
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.set_encrypted_logs(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.set_unencrypted_logs(prev_unencrypted_logs_hash, prev_unencrypted_log_preimages_length);

        let public_inputs = builder.execute();

        assert_eq(public_inputs.end.encrypted_log_preimages_length, prev_encrypted_log_preimages_length);
        assert_eq(public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length);

        assert_eq(public_inputs.end.encrypted_logs_hash, prev_encrypted_logs_hash);

        let expected_unencrypted_logs_hash = compute_logs_hash(prev_unencrypted_logs_hash, unencrypted_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
    }
}