use dep::aztec::constants_gen::{MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX};
use crate::common;
use crate::mocked::{Proof, AggregationObject, verify_previous_kernel_state};
use crate::transaction::request::TxRequest;
use crate::abis::{
    previous_kernel_data::PreviousKernelData,
    private_kernel::private_call_data::PrivateCallData,
    new_contract_data::NewContractData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, KernelCircuitPublicInputsBuilder}
};
use crate::utils::{
    arrays,
    bounded_vec::BoundedVec,
};
use dep::std::unsafe;

struct PrivateKernelInputsInner {
    previous_kernel : PreviousKernelData,
    private_call : PrivateCallData,
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
        let start_private_call_stack_length = arrays::array_length(start.private_call_stack);

        assert(self.previous_kernel.public_inputs.is_private, "Cannot verify a non-private kernel snark in the private kernel circuit");
        assert(this_call_stack_item.function_data().is_constructor == false, "A constructor must be executed as the first tx in the recursion");
        assert(start_private_call_stack_length != 0, "Cannot execute private kernel circuit with an empty private call stack");
    }

    fn native_private_kernel_circuit_inner(self) -> KernelCircuitPublicInputs {
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
    use crate::abis::{
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        read_request_membership_witness::ReadRequestMembershipWitness,
    };
    use crate::tests::{
        testing_harness::{
            create_previous_kernel_data,
            create_private_call_data,
            generate_read_requests,
            PrivateAppInputs,
            non_zero_items,
        },
        apps::{
            constructor::constructor_app,
            deposit::deposit_app,
        },
    };
    use crate::address::Address;
    use crate::hash::compute_logs_hash;
    use dep::aztec::constants_gen::{
        MAX_READ_REQUESTS_PER_CALL,
        MAX_READ_REQUESTS_PER_TX,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_TX,
        MAX_NEW_NULLIFIERS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_TX,
        EMPTY_NULLIFIED_COMMITMENT,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        MAX_NEW_L2_TO_L1_MSGS_PER_TX,
        NOTE_HASH_TREE_HEIGHT,
    };

    fn build_inputs<T>(
        function: fn (PrivateAppInputs, T) -> PrivateCircuitPublicInputs,
        app_params: T,
    ) -> PrivateKernelInputsInner {
        let msg_sender = Address::from_field(27);
        let params = dep::std::unsafe::zeroed();
        let previous_kernel = create_previous_kernel_data(
            false,
            constructor_app,
            params,
            msg_sender,
        );
        
        let (private_call, _) = create_private_call_data(
            false,
            function,
            app_params,
            msg_sender,
        );

        PrivateKernelInputsInner {
            previous_kernel,
            private_call,
        }
    }

    #[test(should_fail_with = "contract address cannot be 0")]
    fn private_function_zero_storage_contract_address_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // Set storage_contract_address to 0
        private_inputs.private_call.call_stack_item.inner.public_inputs.call_context.storage_contract_address = Address::ZERO();

        // We need to update the previous_kernel's private_call_stack because the current call_stack_item has changed.
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "call is internal but msg_sender is not self")]
    fn private_function_incorrect_is_internal() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // Make the call internal but msg_sender != storage_contract_address.
        private_inputs.private_call.call_stack_item.inner.function_data.is_internal = true;
        private_inputs.private_call.call_stack_item.inner.public_inputs.call_context.msg_sender = Address::from_field(1);
        private_inputs.private_call.call_stack_item.inner.public_inputs.call_context.storage_contract_address = Address::from_field(2);

        // We need to update the previous_kernel's private_call_stack because the current call_stack_item has changed.
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "purported_contract_tree_root does not match previous_kernel_contract_tree_root")]
    fn private_function_incorrect_contract_tree_root_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // Set historic_tree_root to a wrong value (the correct value + 1).
        let contract_tree_root = private_inputs.previous_kernel.public_inputs.constants.block_data.block.contract_tree_root;
        private_inputs.previous_kernel.public_inputs.constants.block_data.block.contract_tree_root = contract_tree_root + 1;

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_contract_leaf_index_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // Set the leaf index of the contract leaf to a wrong value (the correct value + 1).
        let leaf_index = private_inputs.private_call.contract_leaf_membership_witness.leaf_index;
        private_inputs.private_call.contract_leaf_membership_witness.leaf_index = leaf_index + 1;

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_contract_leaf_sibling_path_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // Set the first value of the sibling path to a wrong value (the correct value + 1).
        let sibling_path_0 = private_inputs.private_call.contract_leaf_membership_witness.sibling_path[0];
        private_inputs.private_call.contract_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_function_leaf_index_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // Set the leaf index of the function leaf to a wrong value (the correct value + 1).
        let leaf_index = private_inputs.private_call.function_leaf_membership_witness.leaf_index;
        private_inputs.private_call.function_leaf_membership_witness.leaf_index = leaf_index + 1;

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_function_leaf_sibling_path_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // Set the first value of the sibling path to a wrong value (the correct value + 1).
        let sibling_path_0 = private_inputs.private_call.function_leaf_membership_witness.sibling_path[0];
        private_inputs.private_call.function_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "calculated private_call_hash does not match provided private_call_hash at the top of the callstack")]
    fn private_function_incorrect_call_stack_item_hash_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // Set the first call stack hash to a wrong value (the correct value + 1).
        let hash_0 = private_inputs.private_call.call_stack_item.inner.public_inputs.private_call_stack[0];
        private_inputs.private_call.call_stack_item.inner.public_inputs.private_call_stack[0] = hash_0 + 1;

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_return_values() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let malformed_return_values = [0,0,0,553];
        private_inputs.private_call.call_stack_item.inner.public_inputs.return_values = malformed_return_values;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_read_requests() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_read_requests = [0; MAX_READ_REQUESTS_PER_CALL];
        malformed_read_requests[1] = 9123;
        malformed_read_requests[3] = 12;
        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = malformed_read_requests;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_commitments = [0; MAX_NEW_COMMITMENTS_PER_CALL];
        malformed_commitments[1] = 9123;
        private_inputs.private_call.call_stack_item.inner.public_inputs.new_commitments = malformed_commitments;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_nullifiers() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_nullifiers = [0; MAX_NEW_NULLIFIERS_PER_CALL];
        malformed_nullifiers[MAX_NEW_NULLIFIERS_PER_CALL-1] = 12;
        private_inputs.private_call.call_stack_item.inner.public_inputs.new_nullifiers = malformed_nullifiers;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_nullified_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_nullified_commitments = [0; MAX_NEW_NULLIFIERS_PER_CALL];
        malformed_nullified_commitments[2] = EMPTY_NULLIFIED_COMMITMENT;
        private_inputs.private_call.call_stack_item.inner.public_inputs.nullified_commitments = malformed_nullified_commitments;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_private_call_stack() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_private_call_stack = [0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL];
        malformed_private_call_stack[1] = 888;
        private_inputs.private_call.call_stack_item.inner.public_inputs.private_call_stack = malformed_private_call_stack;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_public_call_stack() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_public_call_stack = [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL];
        malformed_public_call_stack[1] = 888;
        private_inputs.private_call.call_stack_item.inner.public_inputs.public_call_stack = malformed_public_call_stack;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_new_l2_to_l1_msgs() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_new_l2_to_l1_msgs = [0; MAX_NEW_L2_TO_L1_MSGS_PER_CALL];
        malformed_new_l2_to_l1_msgs[1] = 888;
        private_inputs.private_call.call_stack_item.inner.public_inputs.new_l2_to_l1_msgs = malformed_new_l2_to_l1_msgs;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_read_requests() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_read_requests = [0; MAX_READ_REQUESTS_PER_TX];
        malformed_read_requests[1] = 9123;
        malformed_read_requests[3] = 12;
        private_inputs.previous_kernel.public_inputs.end.read_requests = malformed_read_requests;
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_commitments = [0; MAX_NEW_COMMITMENTS_PER_TX];
        malformed_commitments[1] = 9123;
        private_inputs.previous_kernel.public_inputs.end.new_commitments = malformed_commitments;
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_nullifiers() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_nullifiers = [0; MAX_NEW_NULLIFIERS_PER_TX];
        malformed_nullifiers[MAX_NEW_NULLIFIERS_PER_TX - 1] = 12;
        private_inputs.previous_kernel.public_inputs.end.new_nullifiers = malformed_nullifiers;
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_nullified_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_nullified_commitments = [0; MAX_NEW_NULLIFIERS_PER_TX];
        malformed_nullified_commitments[2] = EMPTY_NULLIFIED_COMMITMENT;
        private_inputs.previous_kernel.public_inputs.end.nullified_commitments = malformed_nullified_commitments;
        
        private_inputs.native_private_kernel_circuit_inner();
    }
    
    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_private_call_stack() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_private_call_stack = [0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX];
        malformed_private_call_stack[1] = 888;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack = malformed_private_call_stack;
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_public_call_stack() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_public_call_stack = [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX];
        malformed_public_call_stack[1] = 888;
        private_inputs.previous_kernel.public_inputs.end.public_call_stack = malformed_public_call_stack;
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_end_arrays_l2_to_l1_msgs() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_l2_to_l1_msgs = [0; MAX_NEW_L2_TO_L1_MSGS_PER_TX];
        malformed_l2_to_l1_msgs[MAX_NEW_L2_TO_L1_MSGS_PER_TX - 1] = 1;
        private_inputs.previous_kernel.public_inputs.end.new_l2_to_l1_msgs = malformed_l2_to_l1_msgs;
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with = "push_vec out of bounds")]
    fn private_kernel_should_fail_if_aggregating_too_many_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        // The current call stack has 1 commitment;
        private_inputs.private_call.call_stack_item.inner.public_inputs.new_commitments[0] = 1;

        // We need to update the previous_kernel's private_call_stack because the current call_stack_item has changed.
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();
        
        // Mock the previous new commitments to be full, therefore no more commitments can be added.
        let mut full_new_commitments = [0; MAX_NEW_COMMITMENTS_PER_TX];
        for i in 0..MAX_NEW_COMMITMENTS_PER_TX {
            full_new_commitments[i] = i + 1;
        }
        private_inputs.previous_kernel.public_inputs.end.new_commitments = full_new_commitments;
        
        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_request() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(2);

        // tweak read_request so it gives wrong root when paired with its sibling path
        read_requests[1] += 1;

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        // We need to update the previous_kernel's private_call_stack because the current call_stack_item has changed.
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_leaf_index() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(2);

        // tweak leaf index so it gives wrong root when paired with its request and sibling path
        read_request_membership_witnesses[1].leaf_index += 1;
        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        // We need to update the previous_kernel's private_call_stack because the current call_stack_item has changed.
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_sibling_path() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(2);

        // tweak sibling path so it gives wrong root when paired with its request
        read_request_membership_witnesses[1].sibling_path[1] += 1;
        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        // We need to update the previous_kernel's private_call_stack because the current call_stack_item has changed.
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_root_mismatch() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(1);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        // Set the root to be a different root so the above read request is not under this root.
        let old_root = private_inputs.previous_kernel.public_inputs.constants.block_data.block.note_hash_tree_root;
        private_inputs.previous_kernel.public_inputs.constants.block_data.block.note_hash_tree_root = old_root + 1;

        private_inputs.native_private_kernel_circuit_inner();
    }

    #[test]
    fn native_no_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(0);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        // We need to update the previous_kernel's private_call_stack because the current call_stack_item has changed.
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        let public_inputs = private_inputs.native_private_kernel_circuit_inner();

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_one_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(1);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        let public_inputs = private_inputs.native_private_kernel_circuit_inner();

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_two_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(2);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        let public_inputs = private_inputs.native_private_kernel_circuit_inner();

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_max_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(MAX_READ_REQUESTS_PER_CALL as u64);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        let public_inputs = private_inputs.native_private_kernel_circuit_inner();

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_one_transient_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(1);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses.map(|mut witness: ReadRequestMembershipWitness<NOTE_HASH_TREE_HEIGHT>| {
            witness.is_transient = true;
            witness
        });
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        let public_inputs = private_inputs.native_private_kernel_circuit_inner();

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 1);
    }

    #[test]
    fn native_max_read_requests_one_transient_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(MAX_READ_REQUESTS_PER_CALL as u64);

        read_request_membership_witnesses[1].is_transient = true;
        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        let public_inputs = private_inputs.native_private_kernel_circuit_inner();

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 1);
    }

    #[test]
    fn native_max_read_requests_all_transient_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(MAX_READ_REQUESTS_PER_CALL as u64);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses.map(|mut witness: ReadRequestMembershipWitness<NOTE_HASH_TREE_HEIGHT>| {
            witness.is_transient = true;
            witness
        });
        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        let public_inputs = private_inputs.native_private_kernel_circuit_inner();

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), MAX_READ_REQUESTS_PER_CALL as u64);
    }

    #[test]
    fn native_logs_are_hashed_as_expected() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        // Logs for the current call stack.
        let encrypted_logs_hash = [16, 69];
        let encrypted_log_preimages_length = 100;
        let unencrypted_logs_hash = [26, 47];
        let unencrypted_log_preimages_length = 50;
        private_inputs.private_call.call_stack_item.inner.public_inputs.encrypted_logs_hash = encrypted_logs_hash;
        private_inputs.private_call.call_stack_item.inner.public_inputs.encrypted_log_preimages_length = encrypted_log_preimages_length;
        private_inputs.private_call.call_stack_item.inner.public_inputs.unencrypted_logs_hash = unencrypted_logs_hash;
        private_inputs.private_call.call_stack_item.inner.public_inputs.unencrypted_log_preimages_length = unencrypted_log_preimages_length;

        private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] = private_inputs.private_call.call_stack_item.hash();

        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = [80, 429];
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = [956, 112];
        let prev_unencrypted_log_preimages_length = 24;
        private_inputs.previous_kernel.public_inputs.end.encrypted_logs_hash = prev_encrypted_logs_hash;
        private_inputs.previous_kernel.public_inputs.end.encrypted_log_preimages_length = prev_encrypted_log_preimages_length;
        private_inputs.previous_kernel.public_inputs.end.unencrypted_logs_hash = prev_unencrypted_logs_hash;
        private_inputs.previous_kernel.public_inputs.end.unencrypted_log_preimages_length = prev_unencrypted_log_preimages_length;

        let public_inputs = private_inputs.native_private_kernel_circuit_inner();

        assert_eq(public_inputs.end.encrypted_log_preimages_length, encrypted_log_preimages_length + prev_encrypted_log_preimages_length);
        assert_eq(public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length);

        let expected_encrypted_logs_hash = compute_logs_hash(prev_encrypted_logs_hash, encrypted_logs_hash);
        assert_eq(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);

        let expected_unencrypted_logs_hash = compute_logs_hash(prev_unencrypted_logs_hash, unencrypted_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
    }

    #[test(should_fail_with="The 0th nullifier in the accumulated nullifier array is zero")]
    fn zero_0th_nullifier_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(deposit_app, params);

        private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0] = 0;

        private_inputs.native_private_kernel_circuit_inner();
    }
}
