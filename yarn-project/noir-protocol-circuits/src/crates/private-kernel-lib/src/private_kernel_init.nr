use dep::aztec::constants_gen;
use crate::common;
use dep::types::mocked::{Proof, AggregationObject,verify_previous_kernel_state};
use dep::types::transaction::request::TxRequest;
use dep::types::abis::{
    combined_constant_data::CombinedConstantData,
    private_kernel::private_call_data::PrivateCallData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, KernelCircuitPublicInputsBuilder}
};
use dep::types::utils::arrays;
use dep::std::unsafe;

// Initialization struct for private inputs to the private kernel
struct PrivateKernelInputsInit {
    tx_request : TxRequest,
    private_call : PrivateCallData,
}

impl PrivateKernelInputsInit {
    fn initialize_end_values(self, public_inputs : &mut KernelCircuitPublicInputsBuilder) {
        let private_call_public_inputs = self.private_call.call_stack_item.public_inputs();
        let constants = CombinedConstantData{
            block_data : private_call_public_inputs.historical_block_data,
            tx_context : self.tx_request.tx_context,
        };

        public_inputs.constants = constants;
    }

    // Confirm that the TxRequest (user's intent) 
    // matches the private call being executed
    fn validate_this_private_call_against_tx_request(self) {
        let tx_request = self.tx_request;
        // Call stack item for the initial call
        let call_stack_item = self.private_call.call_stack_item;
        
        // Checks to ensure that the user's intent matches the initial private call
        //
        // We use the word correct to denote whether it matches the user intent.
        //
        // Ensure we are calling the correct initial contract
        let origin_address_matches = tx_request.origin.eq(call_stack_item.contract_address());
        assert(origin_address_matches, "origin address does not match call stack items contract address");
        //
        // Ensure we are calling the correct initial function in the contract
        let entry_point_function_matches = tx_request.function_data.hash() == call_stack_item.function_data().hash();
        assert(entry_point_function_matches, "tx_request.function_data must match call_stack_item.function_data");
        //
        // Ensure we are passing the correct arguments to the function.
        let args_match = tx_request.args_hash == call_stack_item.public_inputs().args_hash;
        assert(args_match, "noir function args passed to tx_request must match args in the call_stack_item");
    }

    fn validate_inputs(self) {
        let call_stack_item = self.private_call.call_stack_item;
        let function_data = call_stack_item.function_data();

        let public_inputs = call_stack_item.public_inputs();

        /* TODO(Mike/David): If we are going to have 3 initial calls on the private call stack,
        * then do we still need the `private_call_stack`
        * despite no longer needing a full `previous_kernel`
        */

        assert(function_data.is_private, "Cannot execute a non-private function with the private kernel circuit");
        assert(public_inputs.call_context.is_delegate_call == false, "Users cannot make a delegatecall");
        assert(public_inputs.call_context.is_static_call == false, "Users cannot make a static call");
        
        // The below also prevents delegatecall/staticcall in the base case
        assert(public_inputs.call_context.storage_contract_address.eq(call_stack_item.contract_address()), "Storage contract address must be that of the called contract");
    }

    fn update_end_values(self, public_inputs : &mut KernelCircuitPublicInputsBuilder) {
        // We only initialized constants member of public_inputs so far. Therefore, there must not be any
        // new nullifiers or logs as part of public_inputs.
        
        let new_commitments_empty = arrays::is_empty_array(public_inputs.end.new_commitments.storage);
        assert(new_commitments_empty, "public_inputs.end.new_commitments must start as empty in initial kernel iteration");

        let new_nullifiers_empty = arrays::is_empty_array(public_inputs.end.new_nullifiers.storage);
        assert(new_nullifiers_empty, "public_inputs.end.new_nullifiers must start as empty in initial kernel iteration");

        let end_nullified_commitments_empty = arrays::is_empty_array(public_inputs.end.nullified_commitments.storage);
        assert(end_nullified_commitments_empty, "public_inputs.end.nullified_commitments must start as empty in initial kernel iteration");

        let encrypted_logs_hash_empty = arrays::log_hashes_empty(public_inputs.end.encrypted_logs_hash);
        assert(encrypted_logs_hash_empty, "public_inputs.end.encrypted_logs_hash must start as empty in initial kernel iteration");

        let unencrypted_logs_hash_empty = arrays::log_hashes_empty(public_inputs.end.unencrypted_logs_hash);
        assert(unencrypted_logs_hash_empty, "public_inputs.end.unencrypted_logs_hash must start as empty in initial kernel iteration");

        let read_requests_empty = arrays::is_empty_array(public_inputs.end.read_requests.storage);
        assert(read_requests_empty, "public_inputs.end.read_requests must start as empty in initial kernel iteration");

        let pending_read_requests_empty = arrays::is_empty_array(public_inputs.end.pending_read_requests.storage);
        assert(pending_read_requests_empty, "public_inputs.end.pending_read_requests_empty must start as empty in initial kernel iteration");

        assert(public_inputs.end.encrypted_log_preimages_length == 0,"public_inputs.end.encrypted_log_preimages_length must start as 0 in initial kernel iteration");
        assert(public_inputs.end.unencrypted_log_preimages_length == 0,"public_inputs.end.unencrypted_log_preimages_length must start as 0 in initial kernel iteration");

        // Since it's the first iteration, we need to push the the tx hash nullifier into the `new_nullifiers` array
        public_inputs.end.new_nullifiers.push(self.tx_request.hash());
        
        // Push an empty nullified commitment too since each nullifier must
        // be paired with a nonzero (real or "empty") nullified commitment
        public_inputs.end.nullified_commitments.push(constants_gen::EMPTY_NULLIFIED_COMMITMENT);

        // Note that we do not need to nullify the transaction request nonce anymore.
        // Should an account want to additionally use nonces for replay protection or handling cancellations,
        // they will be able to do so in the account contract logic:
        // https://github.com/AztecProtocol/aztec-packages/issues/660
    }

    pub fn native_private_kernel_circuit_initial(self) -> KernelCircuitPublicInputs {
        let mut public_inputs : KernelCircuitPublicInputsBuilder = unsafe::zeroed();
        public_inputs.is_private = true;
        
        self.initialize_end_values(&mut public_inputs);

        self.validate_inputs();

        common::validate_arrays(self.private_call.call_stack_item.public_inputs());

        self.validate_this_private_call_against_tx_request();

        common::validate_call_stack(self.private_call);

        common::validate_read_requests(
            public_inputs.constants.block_data.note_hash_tree_root(),
            self.private_call.call_stack_item.public_inputs().read_requests,
            self.private_call.read_request_membership_witnesses
        );

        // TODO(dbanks12): feels like update_end_values should happen after contract logic
        self.update_end_values(&mut public_inputs);
        common::update_end_values(self.private_call, &mut public_inputs);

        common::contract_logic(self.private_call,
                          &mut public_inputs,
                          self.tx_request.tx_context.contract_deployment_data,
                          self.tx_request.function_data);

        // TODO(David): What previous kernel proof are we talking about, since this is private-kernel-init
        let (is_previous_state_valid, updated_aggregation_object) = verify_previous_kernel_state(public_inputs.end.aggregation_object,self.private_call.proof);
        assert(is_previous_state_valid);

        public_inputs.end.aggregation_object = updated_aggregation_object;

        public_inputs.finish()
    }
}

mod tests {
    use crate::private_kernel_init::PrivateKernelInputsInit;
    use dep::types::abis::{
        kernel_circuit_public_inputs::{KernelCircuitPublicInputs, KernelCircuitPublicInputsBuilder},
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        read_request_membership_witness::ReadRequestMembershipWitness,
    };
    use dep::types::tests::{
        testing_harness::{
            create_private_call_data,
            generate_read_requests,
            build_tx_request,
            PrivateAppInputs,
            validate_no_new_deployed_contract,
            validate_deployed_contract_address,
            non_zero_items,
        },
        apps::deposit::{
            deposit_app,
            DepositParams,
        },
        apps::constructor::{
            constructor_app,
            ConstructorParams,
        }
    };
    use dep::types::address::Address;
    use dep::types::hash::{accumulate_sha256};
    use dep::types::utils::uint128::U128;
    use dep::aztec::constants_gen::{
        MAX_READ_REQUESTS_PER_CALL,
        MAX_NEW_COMMITMENTS_PER_CALL,
        MAX_NEW_NULLIFIERS_PER_CALL,
        EMPTY_NULLIFIED_COMMITMENT,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
        NOTE_HASH_TREE_HEIGHT,
    };

    fn build_inputs<T>(
        is_constructor: bool,
        function: fn (PrivateAppInputs, T) -> PrivateCircuitPublicInputs,
        app_params: T,
    ) -> PrivateKernelInputsInit {
        let msg_sender = Address::from_field(27);
        let (private_call, contract_deployment_data) = create_private_call_data(
            is_constructor,
            function,
            app_params,
            msg_sender,
        );

        let tx_request = build_tx_request(
            is_constructor,
            contract_deployment_data,
            private_call,
        );

        PrivateKernelInputsInit {
            tx_request,
            private_call,
        }
    }

    #[test]
    fn deposit() {
        let params = dep::std::unsafe::zeroed();
        let private_inputs = build_inputs(false, deposit_app, params);
        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_no_new_deployed_contract(public_inputs);

        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

        // Log preimages length should increase by `(un)encrypted_log_preimages_length` from private input
        assert_eq(
            public_inputs.end.encrypted_log_preimages_length, 
            private_inputs.private_call.call_stack_item.inner.public_inputs.encrypted_log_preimages_length
        );
        assert_eq(
            public_inputs.end.unencrypted_log_preimages_length, 
            private_inputs.private_call.call_stack_item.inner.public_inputs.unencrypted_log_preimages_length
        );

        // Logs hashes should be a sha256 hash of a 0 value and the `(un)encrypted_logs_hash` from private input
        let expected_encrypted_logs_hash = accumulate_sha256([U128::from_field(0), U128::from_field(0), params.encrypted_logs_hash[0], params.encrypted_logs_hash[1]]);
        assert_eq(expected_encrypted_logs_hash, public_inputs.end.encrypted_logs_hash);

        let expected_unencrypted_logs_hash = accumulate_sha256([U128::from_field(0), U128::from_field(0), params.unencrypted_logs_hash[0], params.unencrypted_logs_hash[1]]);
        assert_eq(expected_unencrypted_logs_hash, public_inputs.end.unencrypted_logs_hash);
    }

    #[test]
    fn basic_contract_deployment() {
        let params = dep::std::unsafe::zeroed();
        let private_inputs = build_inputs(true, constructor_app, params);
        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_deployed_contract_address(private_inputs.tx_request, private_inputs.private_call, public_inputs);

        // Since there are no logs, log preimages length should be 0 and both logs hashes should be a sha256 hash of 2 zero
        // values
        assert_eq(public_inputs.end.encrypted_log_preimages_length, 0);
        assert_eq(public_inputs.end.unencrypted_log_preimages_length, 0);

        let expected_logs_hash = accumulate_sha256([U128::from_field(0), U128::from_field(0), U128::from_field(0), U128::from_field(0)]);

        assert_eq(public_inputs.end.encrypted_logs_hash, expected_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_logs_hash);
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_return_values() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let malformed_return_values = [0,0,0,553];
        private_inputs.private_call.call_stack_item.inner.public_inputs.return_values = malformed_return_values;
        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_deployed_contract_address(private_inputs.tx_request, private_inputs.private_call, public_inputs);
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_read_requests() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let mut malformed_read_requests = [0; MAX_READ_REQUESTS_PER_CALL];
        malformed_read_requests[1] = 9123;
        malformed_read_requests[3] = 12;
        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = malformed_read_requests;
        
        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let mut malformed_commitments = [0; MAX_NEW_COMMITMENTS_PER_CALL];
        malformed_commitments[1] = 9123;
        private_inputs.private_call.call_stack_item.inner.public_inputs.new_commitments = malformed_commitments;
        
        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_nullifiers() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let mut malformed_nullifiers = [0; MAX_NEW_NULLIFIERS_PER_CALL];
        malformed_nullifiers[MAX_NEW_NULLIFIERS_PER_CALL-1] = 12;
        private_inputs.private_call.call_stack_item.inner.public_inputs.new_nullifiers = malformed_nullifiers;
        
        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_nullified_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let mut malformed_nullified_commitments = [0; MAX_NEW_NULLIFIERS_PER_CALL];
        malformed_nullified_commitments[2] = EMPTY_NULLIFIED_COMMITMENT;
        private_inputs.private_call.call_stack_item.inner.public_inputs.nullified_commitments = malformed_nullified_commitments;
        
        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_private_call_stack() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let mut malformed_private_call_stack = [0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL];
        malformed_private_call_stack[1] = 888;
        private_inputs.private_call.call_stack_item.inner.public_inputs.private_call_stack = malformed_private_call_stack;
        
        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_public_call_stack() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let mut malformed_public_call_stack = [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL];
        malformed_public_call_stack[1] = 888;
        private_inputs.private_call.call_stack_item.inner.public_inputs.public_call_stack = malformed_public_call_stack;
        
        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_new_l2_to_l1_msgs() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let mut malformed_new_l2_to_l1_msgs = [0; MAX_NEW_L2_TO_L1_MSGS_PER_CALL];
        malformed_new_l2_to_l1_msgs[1] = 888;
        private_inputs.private_call.call_stack_item.inner.public_inputs.new_l2_to_l1_msgs = malformed_new_l2_to_l1_msgs;
        
        private_inputs.native_private_kernel_circuit_initial();
    }

    // TODO this is invalid as should_fail_with "call stack item hash doesn't match preimage hash"
    #[test(should_fail)]
    fn contract_deployment_call_stack_item_hash_mismatch_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        private_inputs.private_call.call_stack_item.inner.public_inputs.private_call_stack[0] = 27;
        
        private_inputs.native_private_kernel_circuit_initial();
    }

    // TODO this is invalid as should_fail_with "contract address supplied doesn't match derived address"
    #[test(should_fail)]
    fn contract_deployment_incorrect_contract_address_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let random_address = Address::from_field(27);
        private_inputs.private_call.call_stack_item.inner.public_inputs.call_context.storage_contract_address = random_address;
        private_inputs.tx_request.origin = random_address;
        private_inputs.private_call.call_stack_item.inner.contract_address = random_address;

        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with="origin address does not match call stack items contract address")]
    fn contract_deployment_contract_address_mismatch_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        let random_contract_address = Address::from_field(27);
        private_inputs.private_call.call_stack_item.inner.public_inputs.call_context.storage_contract_address =
            random_contract_address;
        private_inputs.private_call.call_stack_item.inner.contract_address = random_contract_address;

        private_inputs.native_private_kernel_circuit_initial();
    }

    // TODO this is invalid as should_fail_with "tx_request.function_data must match call_stack_item.function_data"
    #[test(should_fail)]
    fn contract_deployment_function_data_mismatch_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        private_inputs.tx_request.function_data.selector.inner = 27;

        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with="noir function args passed to tx_request must match args in the call_stack_item")]
    fn contract_deployment_args_hash_mismatch_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        // Modify the args hash in tx request.
        private_inputs.tx_request.args_hash = 29;

        private_inputs.native_private_kernel_circuit_initial();
    }

    // TODO this is invalid as should_fail_with "Cannot execute a non-private function with the private kernel circuit"
    #[test(should_fail)]
    fn private_function_is_private_false_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        // Set is_private in function data to false.
        private_inputs.private_call.call_stack_item.inner.function_data.is_private = false;

        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with="Users cannot make a static call")]
    fn private_function_static_call_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        // Set is_static_call to true.
        private_inputs.private_call.call_stack_item.inner.public_inputs.call_context.is_static_call = true;

        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with="Users cannot make a delegatecall")]
    fn private_function_delegate_call_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        // Set is_static_call to true.
        private_inputs.private_call.call_stack_item.inner.public_inputs.call_context.is_delegate_call = true;

        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with="Storage contract address must be that of the called contract")]
    fn private_function_incorrect_storage_contract_address_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(true, constructor_app, params);

        // Set the storage_contract_address to a random scalar.
        private_inputs.private_call.call_stack_item.inner.public_inputs.call_context.storage_contract_address = Address::from_field(356);

        private_inputs.native_private_kernel_circuit_initial();
    }


    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_request() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(2);

        // tweak read_request so it gives wrong root when paired with its sibling path
        read_requests[1] += 1;

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_leaf_index() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(2);

        // tweak leaf index so it gives wrong root when paired with its request and sibling path
        read_request_membership_witnesses[1].leaf_index += 1;
        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test(should_fail_with="private data tree root mismatch")]
    fn native_read_request_bad_sibling_path() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(2);

        // tweak sibling path so it gives wrong root when paired with its request
        read_request_membership_witnesses[1].sibling_path[1] += 1;
        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        private_inputs.native_private_kernel_circuit_initial();
    }

    #[test]
    fn native_no_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(0);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_one_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(1);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_two_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(2);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_max_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(MAX_READ_REQUESTS_PER_CALL as u64);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_one_transient_read_requests_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(1);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses.map(|mut witness: ReadRequestMembershipWitness<NOTE_HASH_TREE_HEIGHT>| {
            witness.is_transient = true;
            witness
        });

        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 1);
    }

    #[test]
    fn native_max_read_requests_one_transient_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(MAX_READ_REQUESTS_PER_CALL as u64);

        read_request_membership_witnesses[1].is_transient = true;
        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), 1);
    }

     #[test]
    fn native_max_read_requests_all_transient_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(false, deposit_app, params);

        let (mut read_requests, mut read_request_membership_witnesses) = generate_read_requests(MAX_READ_REQUESTS_PER_CALL as u64);

        private_inputs.private_call.call_stack_item.inner.public_inputs.read_requests = read_requests;
        private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses.map(|mut witness: ReadRequestMembershipWitness<NOTE_HASH_TREE_HEIGHT>| {
            witness.is_transient = true;
            witness
        });

        let public_inputs = private_inputs.native_private_kernel_circuit_initial();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(non_zero_items(public_inputs.end.read_requests), MAX_READ_REQUESTS_PER_CALL as u64);
    }
}
