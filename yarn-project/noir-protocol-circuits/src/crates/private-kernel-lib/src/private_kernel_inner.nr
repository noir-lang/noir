use crate::common;
use dep::std::unsafe;
use dep::types::{
    abis::{
        previous_kernel_data::PreviousKernelData,
        private_kernel::private_call_data::PrivateCallData,
        kernel_circuit_public_inputs::{KernelCircuitPublicInputs, KernelCircuitPublicInputsBuilder},
    },
    mocked::verify_previous_kernel_state,
    utils::arrays::array_length,
};

struct PrivateKernelInputsInner {
    previous_kernel: PreviousKernelData,
    private_call: PrivateCallData,
}

impl PrivateKernelInputsInner {
    // TODO: We can create a type alias for private call stack item
    fn pop_and_validate_this_private_call_hash(self, public_inputs : &mut KernelCircuitPublicInputsBuilder){
        let private_call = self.private_call;
        let popped_private_call_hash = public_inputs.end.private_call_stack.pop();

        let calculated_this_private_call_hash = private_call.call_stack_item.hash();
        assert(popped_private_call_hash == calculated_this_private_call_hash, "calculated private_call_hash does not match provided private_call_hash at the top of the callstack");
    }

    fn validate_contract_tree_root(self) {
        let purported_contract_tree_root = self.private_call.call_stack_item.public_inputs().historical_block_data.contract_tree_root();
        let previous_kernel_contract_tree_root = self.previous_kernel.public_inputs.constants.block_data.contract_tree_root();

        assert(purported_contract_tree_root == previous_kernel_contract_tree_root, "purported_contract_tree_root does not match previous_kernel_contract_tree_root");
    }

    fn validate_inputs(self) {
        let this_call_stack_item = self.private_call.call_stack_item;
        assert(this_call_stack_item.function_data().is_private, "Cannot execute a non-private function with the private kernel circuit");

        let start = self.previous_kernel.public_inputs.end;
        let start_private_call_stack_length = array_length(start.private_call_stack);

        assert(self.previous_kernel.public_inputs.is_private, "Cannot verify a non-private kernel snark in the private kernel circuit");
        assert(this_call_stack_item.function_data().is_constructor == false, "A constructor must be executed as the first tx in the recursion");
        assert(start_private_call_stack_length != 0, "Cannot execute private kernel circuit with an empty private call stack");
    }

    pub fn native_private_kernel_circuit_inner(self) -> KernelCircuitPublicInputs {
        let mut public_inputs : KernelCircuitPublicInputsBuilder = unsafe::zeroed();
        public_inputs.is_private = true;

        common::validate_previous_kernel_values(self.previous_kernel.public_inputs.end);

        // Do this before any functions can modify the inputs.
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        self.validate_inputs();

        common::validate_arrays(self.private_call.call_stack_item.public_inputs());

        self.pop_and_validate_this_private_call_hash(&mut public_inputs);

        common::validate_call_stack(self.private_call);

        common::validate_read_requests(
            public_inputs.constants.block_data.note_hash_tree_root(),
            self.private_call.call_stack_item.public_inputs().read_requests,  // read requests from private call
            self.private_call.read_request_membership_witnesses);

        //TODO(David): feels like update_end_values should happen later
        common::update_end_values(self.private_call, &mut public_inputs);

        // ensure that historic/purported contract tree root matches the one in previous kernel
        self.validate_contract_tree_root();

        let private_call_stack_item = self.private_call.call_stack_item;

        common::contract_logic(self.private_call,
                          &mut public_inputs,
                          private_call_stack_item.public_inputs().contract_deployment_data,
                          private_call_stack_item.function_data());

        let (is_previous_state_valid, updated_aggregation_object) = verify_previous_kernel_state(public_inputs.end.aggregation_object,self.private_call.proof);
        assert(is_previous_state_valid);

        public_inputs.end.aggregation_object = updated_aggregation_object;

        public_inputs.finish()
    }
}


mod tests {
    use crate::private_kernel_inner::PrivateKernelInputsInner;
    use dep::aztec::constants_gen::{
        EMPTY_NULLIFIED_COMMITMENT,
        MAX_READ_REQUESTS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_TX,
    };
    use dep::types::{
        abis::kernel_circuit_public_inputs::KernelCircuitPublicInputs,
        tests::{
            testing_harness::{
                PreviousKernelDataBuilder,
                PrivateCallDataBuilder,
            },
        },
        address::Address,
        hash::compute_logs_hash,
        utils::{
            arrays::array_length,
            bounded_vec::BoundedVec,
        },
    };

    struct PrivateKernelInnerInputsBuilder {
        previous_kernel: PreviousKernelDataBuilder,
        private_call: PrivateCallDataBuilder,
    }

    impl PrivateKernelInnerInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = PreviousKernelDataBuilder::new(false);
            let private_call = PrivateCallDataBuilder::new(false);

            PrivateKernelInnerInputsBuilder { previous_kernel, private_call }
        }

        pub fn execute(&mut self) -> KernelCircuitPublicInputs {
            let mut previous_kernel = self.previous_kernel.finish();
            let private_call = self.private_call.finish();
            // We need to update the previous_kernel's private_call_stack in case the current call_stack_item has changed.
            previous_kernel.public_inputs.end.private_call_stack[0] = private_call.call_stack_item.hash();

            let kernel = PrivateKernelInputsInner {
                previous_kernel,
                private_call,
            };

            kernel.native_private_kernel_circuit_inner()
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }
    }

    #[test(should_fail_with = "contract address cannot be 0")]
    fn private_function_zero_storage_contract_address_fails() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Set storage_contract_address to 0
        builder.private_call.public_inputs.call_context.storage_contract_address = Address::ZERO();

        builder.failed();
    }

    #[test(should_fail_with = "call is internal but msg_sender is not self")]
    fn private_function_incorrect_is_internal() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Make the call internal but msg_sender != storage_contract_address.
        builder.private_call.function_data.is_internal = true;
        builder.private_call.public_inputs.call_context.msg_sender = Address::from_field(1);
        builder.private_call.public_inputs.call_context.storage_contract_address = Address::from_field(2);

        builder.failed();
    }

    #[test(should_fail_with = "purported_contract_tree_root does not match previous_kernel_contract_tree_root")]
    fn private_function_incorrect_contract_tree_root_fails() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Set historic_tree_root to a wrong value (the correct value + 1).
        let contract_tree_root = builder.previous_kernel.block_data.block.contract_tree_root;
        builder.previous_kernel.block_data.block.contract_tree_root = contract_tree_root + 1;

        builder.failed();
    }

    #[test(should_fail_with = "computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_contract_leaf_index_fails() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Set the leaf index of the contract leaf to a wrong value (the correct value + 1).
        let leaf_index = builder.private_call.contract_leaf_membership_witness.leaf_index;
        builder.private_call.contract_leaf_membership_witness.leaf_index = leaf_index + 1;

        builder.failed();
    }

    #[test(should_fail_with = "computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_contract_leaf_sibling_path_fails() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Set the first value of the sibling path to a wrong value (the correct value + 1).
        let sibling_path_0 = builder.private_call.contract_leaf_membership_witness.sibling_path[0];
        builder.private_call.contract_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

        builder.failed();
    }

    #[test(should_fail_with = "computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_function_leaf_index_fails() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Set the leaf index of the function leaf to a wrong value (the correct value + 1).
        let leaf_index = builder.private_call.function_leaf_membership_witness.leaf_index;
        builder.private_call.function_leaf_membership_witness.leaf_index = leaf_index + 1;

        builder.failed();
    }

    #[test(should_fail_with = "computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_function_leaf_sibling_path_fails() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Set the first value of the sibling path to a wrong value (the correct value + 1).
        let sibling_path_0 = builder.private_call.function_leaf_membership_witness.sibling_path[0];
        builder.private_call.function_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

        builder.failed();
    }

    #[test(should_fail_with = "calculated private_call_hash does not match provided private_call_hash at the top of the callstack")]
    fn private_function_incorrect_call_stack_item_hash_fails() {
        let builder = PrivateKernelInnerInputsBuilder::new();

        let private_call = builder.private_call.finish();
        let hash = private_call.call_stack_item.hash();

        let mut previous_kernel = builder.previous_kernel.finish();
        // Set the first call stack hash to a wrong value (the correct value + 1).
        previous_kernel.public_inputs.end.private_call_stack[0] = hash + 1;

        let kernel = PrivateKernelInputsInner {
            previous_kernel,
            private_call,
        };

        let _ = kernel.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_return_values() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.public_inputs.return_values.push_array([0, 553]);

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_read_requests() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.public_inputs.read_requests.push_array([0, 9123]);

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_commitments() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.public_inputs.new_commitments.push_array([0, 9123]);

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_nullifiers() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.public_inputs.new_nullifiers.push_array([0, 12]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_nullified_commitments() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.public_inputs.nullified_commitments.push_array([0, EMPTY_NULLIFIED_COMMITMENT]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_private_call_stack() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.public_inputs.private_call_stack.push_array([0, 888]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_public_call_stack() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.public_inputs.public_call_stack.push_array([0, 888]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_new_l2_to_l1_msgs() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.public_inputs.new_l2_to_l1_msgs.push_array([0, 888]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_read_requests() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.end.read_requests.push_array([0, 9123]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_commitments() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.end.new_commitments.push_array([0, 9123]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_nullifiers() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.end.new_nullifiers.push_array([0, 9123]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_nullified_commitments() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.end.nullified_commitments.push_array([0, EMPTY_NULLIFIED_COMMITMENT]);
        
        builder.failed();
    }
    
    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_private_call_stack() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.end.private_call_stack.push_array([0, 888]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_public_call_stack() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.end.public_call_stack.push_array([0, 888]);
        
        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_l2_to_l1_msgs() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.end.new_l2_to_l1_msgs.push_array([0, 1]);
        
        builder.failed();
    }

    #[test(should_fail_with = "push_vec out of bounds")]
    fn private_kernel_should_fail_if_aggregating_too_many_commitments() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // The current call stack has 1 commitment;
        builder.private_call.public_inputs.new_commitments.push(4321);

        // Mock the previous new commitments to be full, therefore no more commitments can be added.
        let mut full_new_commitments = [0; MAX_NEW_COMMITMENTS_PER_TX];
        for i in 0..MAX_NEW_COMMITMENTS_PER_TX {
            full_new_commitments[i] = i + 1;
        }
        builder.previous_kernel.end.new_commitments.push_array(full_new_commitments);
        
        builder.failed();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_request() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_read_requests(2);

        // tweak read_request so it gives wrong root when paired with its sibling path
        let read_request = builder.private_call.public_inputs.read_requests.pop();
        builder.private_call.public_inputs.read_requests.push(read_request + 1);

        builder.failed();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_leaf_index() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_read_requests(2);

        // Tweak leaf index so it gives wrong root when paired with its request and sibling path.
        let mut read_request_membership_witness = builder.private_call.read_request_membership_witnesses.pop();
        read_request_membership_witness.leaf_index += 1;
        builder.private_call.read_request_membership_witnesses.push(read_request_membership_witness);

        builder.failed();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_sibling_path() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_read_requests(2);

        // Tweak the sibling path of the second read so it gives wrong root when paired with its request.
        let mut read_request_membership_witness = builder.private_call.read_request_membership_witnesses.pop();
        read_request_membership_witness.sibling_path[1] += 1;
        builder.private_call.read_request_membership_witnesses.push(read_request_membership_witness);

        builder.failed();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_root_mismatch() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_read_requests(1);

        // Set the root to be a different root so the above read request is not under this root.
        let old_root = builder.previous_kernel.block_data.block.note_hash_tree_root;
        builder.previous_kernel.block_data.block.note_hash_tree_root = old_root + 1;

        builder.failed();
    }

    #[test]
    fn native_no_read_requests_works() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        assert_eq(builder.private_call.public_inputs.read_requests.len(), 0);

        let public_inputs = builder.execute();

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_one_read_requests_works() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_read_requests(1);

        let public_inputs = builder.execute();

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_two_read_requests_works() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_read_requests(2);

        let public_inputs = builder.execute();

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_max_read_requests_works() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_read_requests(MAX_READ_REQUESTS_PER_CALL);

        let public_inputs = builder.execute();

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_one_transient_read_requests_works() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_transient_read_requests(1);

        let public_inputs = builder.execute();

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 1);
    }

    #[test]
    fn native_max_read_requests_one_transient_works() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_read_requests(1);
        builder.private_call.append_transient_read_requests(1);
        builder.private_call.append_read_requests(MAX_READ_REQUESTS_PER_CALL - 2);

        let public_inputs = builder.execute();

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 1);
    }

    #[test]
    fn native_max_read_requests_all_transient_works() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.private_call.append_transient_read_requests(MAX_READ_REQUESTS_PER_CALL);

        let public_inputs = builder.execute();

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), MAX_READ_REQUESTS_PER_CALL);
    }

    #[test]
    fn native_logs_are_hashed_as_expected() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        // Logs for the current call stack.
        let encrypted_logs_hash = [16, 69];
        let encrypted_log_preimages_length = 100;
        let unencrypted_logs_hash = [26, 47];
        let unencrypted_log_preimages_length = 50;
        builder.private_call.set_encrypted_logs(encrypted_logs_hash, encrypted_log_preimages_length);
        builder.private_call.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = [80, 429];
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = [956, 112];
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.set_encrypted_logs(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.set_unencrypted_logs(prev_unencrypted_logs_hash, prev_unencrypted_log_preimages_length);

        let public_inputs = builder.execute();

        assert_eq(public_inputs.end.encrypted_log_preimages_length, encrypted_log_preimages_length + prev_encrypted_log_preimages_length);
        assert_eq(public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length);

        let expected_encrypted_logs_hash = compute_logs_hash(prev_encrypted_logs_hash, encrypted_logs_hash);
        assert_eq(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);

        let expected_unencrypted_logs_hash = compute_logs_hash(prev_unencrypted_logs_hash, unencrypted_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
    }

    #[test(should_fail_with="The 0th nullifier in the accumulated nullifier array is zero")]
    fn zero_0th_nullifier_fails() {
        let mut builder = PrivateKernelInnerInputsBuilder::new();

        builder.previous_kernel.end.new_nullifiers = BoundedVec::new(0);

        builder.failed();
    }
}
