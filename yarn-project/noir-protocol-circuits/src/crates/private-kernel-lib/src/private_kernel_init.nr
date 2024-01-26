use crate::common;
use dep::std::unsafe;
use dep::types::{
    abis::{
        combined_constant_data::CombinedConstantData,
        private_kernel::private_call_data::PrivateCallData,
        kernel_circuit_public_inputs::{KernelCircuitPublicInputs, KernelCircuitPublicInputsBuilder},
        side_effect::{SideEffect, SideEffectLinkedToNoteHash},
    },
    mocked::{Proof, verify_previous_kernel_state},
    transaction::request::TxRequest,
    traits::is_empty_array,
};

// Initialization struct for private inputs to the private kernel
struct PrivateKernelInputsInit {
    tx_request: TxRequest,
    private_call: PrivateCallData,
}

impl PrivateKernelInputsInit {
    fn initialize_end_values(self, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        public_inputs.constants = CombinedConstantData {
            block_header: self.private_call.call_stack_item.public_inputs.block_header,
            tx_context: self.tx_request.tx_context,
        };
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
        let origin_address_matches = tx_request.origin.eq(call_stack_item.contract_address);
        assert(origin_address_matches, "origin address does not match call stack items contract address");
        //
        // Ensure we are calling the correct initial function in the contract
        let entry_point_function_matches = tx_request.function_data.hash() == call_stack_item.function_data.hash();
        assert(entry_point_function_matches, "tx_request function_data must match call_stack_item function_data");
        //
        // Ensure we are passing the correct arguments to the function.
        let args_match = tx_request.args_hash == call_stack_item.public_inputs.args_hash;
        assert(args_match, "noir function args passed to tx_request must match args in the call_stack_item");
    }

    fn validate_inputs(self) {
        let call_stack_item = self.private_call.call_stack_item;

        let function_data = call_stack_item.function_data;
        assert(function_data.is_private, "Private kernel circuit can only execute a private function");
        assert(function_data.is_internal == false, "Cannot call an internal function directly");

        let call_context = call_stack_item.public_inputs.call_context;
        assert(call_context.is_delegate_call == false, "Users cannot make a delegatecall");
        assert(call_context.is_static_call == false, "Users cannot make a static call");
        // The below also prevents delegatecall/staticcall in the base case
        assert(call_context.storage_contract_address.eq(call_stack_item.contract_address), "Storage contract address must be that of the called contract");
    }

    fn update_end_values(self, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        // Since it's the first iteration, we need to push the the tx hash nullifier into the `new_nullifiers` array
        public_inputs.end.new_nullifiers.push(SideEffectLinkedToNoteHash{
            value: self.tx_request.hash(),
            note_hash: 0,
            counter: 0,
        });

        // Note that we do not need to nullify the transaction request nonce anymore.
        // Should an account want to additionally use nonces for replay protection or handling cancellations,
        // they will be able to do so in the account contract logic:
        // https://github.com/AztecProtocol/aztec-packages/issues/660
    }

    pub fn native_private_kernel_circuit_initial(self) -> KernelCircuitPublicInputs {
        let mut public_inputs: KernelCircuitPublicInputsBuilder = unsafe::zeroed();
        public_inputs.is_private = true;
        
        self.initialize_end_values(&mut public_inputs);

        self.validate_inputs();

        common::validate_arrays(self.private_call.call_stack_item.public_inputs);

        self.validate_this_private_call_against_tx_request();

        common::validate_read_requests(
            public_inputs.constants.block_header.note_hash_tree_root(),
            self.private_call.call_stack_item.public_inputs.read_requests,
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
    use dep::types::constants::{
        MAX_READ_REQUESTS_PER_CALL,
    };
    use dep::types::{
        abis::{
            complete_address::CompleteAddress,
            kernel_circuit_public_inputs::KernelCircuitPublicInputs,
            nullifier_key_validation_request::NullifierKeyValidationRequest,
            private_kernel::private_call_data::PrivateCallData,
            side_effect::{SideEffect, SideEffectLinkedToNoteHash},
        },
        address::AztecAddress,
        grumpkin_point::GrumpkinPoint,
        grumpkin_private_key::GrumpkinPrivateKey,
        hash::{
            compute_constructor_hash,
            compute_logs_hash,
            stdlib_recursion_verification_key_compress_native_vk,
        },
        tests::private_call_data_builder::PrivateCallDataBuilder,
        transaction::request::TxRequest,
        utils::arrays::array_length,
    };

    struct PrivateKernelInitInputsBuilder {
        tx_request: TxRequest,
        private_call: PrivateCallDataBuilder,
    }

    impl PrivateKernelInitInputsBuilder {
        pub fn new() -> Self {
            PrivateKernelInitInputsBuilder::new_with_config(false)
        }

        pub fn new_constructor() -> Self {
            PrivateKernelInitInputsBuilder::new_with_config(true)
        }

        fn new_with_config(is_constructor: bool) -> Self {
            let private_call = PrivateCallDataBuilder::new(is_constructor);
            let tx_request = private_call.build_tx_request();

            PrivateKernelInitInputsBuilder {
                tx_request,
                private_call,
            }
        }

        pub fn execute(self) -> KernelCircuitPublicInputs {
            let kernel = PrivateKernelInputsInit {
                tx_request: self.tx_request,
                private_call: self.private_call.finish(),
            };

            kernel.native_private_kernel_circuit_initial()
        }

        pub fn failed(self) {
            let _ = self.execute();
        }
    }

    pub fn validate_deployed_contract_address(
        tx_request: TxRequest,
        private_call: PrivateCallDataBuilder,
        public_inputs: KernelCircuitPublicInputs
    ) {
        assert_eq(public_inputs.end.new_contracts.len(), 1);
        let cdd = tx_request.tx_context.contract_deployment_data;

        let private_circuit_vk_hash = stdlib_recursion_verification_key_compress_native_vk(private_call.vk);
        let constructor_hash = compute_constructor_hash(
            tx_request.function_data,
            tx_request.args_hash,
            private_circuit_vk_hash
        );
        let contract_address = CompleteAddress::compute(
            cdd.deployer_public_key,
            cdd.contract_address_salt,
            cdd.function_tree_root,
            constructor_hash
        ).address;
        assert(public_inputs.end.new_contracts[0].contract_address.eq(contract_address));
    }

    pub fn validate_no_new_deployed_contract(public_inputs: KernelCircuitPublicInputs) {
        for new_contract in public_inputs.end.new_contracts {
            assert(new_contract.is_empty());
        }
    }

    #[test]
    fn deposit() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Logs for the private call.
        let encrypted_logs_hash = [16, 69];
        let encrypted_log_preimages_length = 100;
        let unencrypted_logs_hash = [26, 47];
        let unencrypted_log_preimages_length = 50;
        builder.private_call.set_encrypted_logs(encrypted_logs_hash, encrypted_log_preimages_length);
        builder.private_call.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

        let public_inputs = builder.execute();

        validate_no_new_deployed_contract(public_inputs);

        // Check the first nullifier is hash of the signed tx request
        let tx_hash = builder.tx_request.hash();
        assert_eq(public_inputs.end.new_nullifiers[0].value, tx_hash);

        // Log preimages length should increase by `(un)encrypted_log_preimages_length` from private input
        assert_eq(public_inputs.end.encrypted_log_preimages_length, encrypted_log_preimages_length);
        assert_eq(public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length);

        // Logs hashes should be a sha256 hash of a 0 value (the previous log hash) and the `(un)encrypted_logs_hash` from private input
        let expected_encrypted_logs_hash = compute_logs_hash([0, 0], encrypted_logs_hash);
        assert_eq(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);

        let expected_unencrypted_logs_hash = compute_logs_hash([0, 0], unencrypted_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
    }

    #[test]
    fn basic_contract_deployment() {
        let builder = PrivateKernelInitInputsBuilder::new_constructor();

        let public_inputs = builder.execute();

        validate_deployed_contract_address(builder.tx_request, builder.private_call, public_inputs);

        // Since there are no logs, log preimages length should be 0 and both logs hashes should be a sha256 hash of 2 zero
        // values
        assert_eq(public_inputs.end.encrypted_log_preimages_length, 0);
        assert_eq(public_inputs.end.unencrypted_log_preimages_length, 0);

        let expected_logs_hash = compute_logs_hash([0, 0], [0, 0]);
        assert_eq(public_inputs.end.encrypted_logs_hash, expected_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_logs_hash);
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_return_values() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.private_call.public_inputs.return_values.push_array([0, 9123]);

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_read_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.private_call.public_inputs.read_requests.push_array(
            [
            SideEffect { value: 0, counter: 0 },
            SideEffect { value: 9123, counter: 1 }
        ]
        );

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_commitments() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.private_call.public_inputs.new_commitments.push_array(
            [
            SideEffect { value: 0, counter: 0 },
            SideEffect { value: 9123, counter: 1 }
        ]
        );

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_nullifiers() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.private_call.public_inputs.new_nullifiers.push_array(
            [
            SideEffectLinkedToNoteHash { value: 0, note_hash: 0, counter: 0 },
            SideEffectLinkedToNoteHash { value: 9123, note_hash: 0, counter: 1 }
        ]
        );

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_private_call_stack() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.private_call.public_inputs.private_call_stack_hashes.push_array([0, 9123]);

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_public_call_stack() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.private_call.public_inputs.public_call_stack_hashes.push_array([0, 9123]);

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_new_l2_to_l1_msgs() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.private_call.public_inputs.new_l2_to_l1_msgs.push_array([0, 9123]);

        builder.failed();
    }

    #[test(should_fail_with="contract address supplied does not match derived address")]
    fn contract_deployment_incorrect_contract_address_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        let random_address = AztecAddress::from_field(27);
        builder.private_call.public_inputs.call_context.storage_contract_address = random_address;
        builder.tx_request.origin = random_address;
        builder.private_call.contract_address = random_address;

        builder.failed();
    }

    #[test(should_fail_with="origin address does not match call stack items contract address")]
    fn contract_deployment_contract_address_mismatch_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        let random_address = AztecAddress::from_field(27);
        builder.private_call.public_inputs.call_context.storage_contract_address = random_address;
        builder.private_call.contract_address = random_address;

        builder.failed();
    }

    #[test(should_fail_with="tx_request function_data must match call_stack_item function_data")]
    fn contract_deployment_function_data_mismatch_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.tx_request.function_data.selector.inner = 27;

        builder.failed();
    }

    #[test(should_fail_with="noir function args passed to tx_request must match args in the call_stack_item")]
    fn contract_deployment_args_hash_mismatch_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        // Modify the args hash in tx request.
        builder.tx_request.args_hash = 29;

        builder.failed();
    }

    #[test(should_fail_with="Private kernel circuit can only execute a private function")]
    fn private_function_is_private_false_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        // Set is_private in function data to false.
        builder.private_call.function_data.is_private = false;

        builder.failed();
    }

    #[test(should_fail_with="Cannot call an internal function directly")]
    fn private_function_is_internal_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        builder.private_call.function_data.is_internal = true;

        builder.failed();
    }

    #[test(should_fail_with="Users cannot make a static call")]
    fn private_function_static_call_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        // Set is_static_call to true.
        builder.private_call.public_inputs.call_context.is_static_call = true;

        builder.failed();
    }

    #[test(should_fail_with="Users cannot make a delegatecall")]
    fn private_function_delegate_call_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        // Set is_delegate_call to true.
        builder.private_call.public_inputs.call_context.is_delegate_call = true;

        builder.failed();
    }

    #[test(should_fail_with="Storage contract address must be that of the called contract")]
    fn private_function_incorrect_storage_contract_address_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        // Set the storage_contract_address to a random scalar.
        builder.private_call.public_inputs.call_context.storage_contract_address = AztecAddress::from_field(356);

        builder.failed();
    }

    #[test(should_fail_with="computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_contract_leaf_index_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set the leaf index of the contract leaf to a wrong value (the correct value + 1).
        let leaf_index = builder.private_call.contract_leaf_membership_witness.leaf_index;
        builder.private_call.contract_leaf_membership_witness.leaf_index = leaf_index + 1;

        builder.failed();
    }

    #[test(should_fail_with="computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_contract_leaf_sibling_path_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set the first value of the sibling path to a wrong value (the correct value + 1).
        let sibling_path_0 = builder.private_call.contract_leaf_membership_witness.sibling_path[0];
        builder.private_call.contract_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

        builder.failed();
    }

    #[test(should_fail_with="computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_function_leaf_index_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set the leaf index of the function leaf to a wrong value (the correct value + 1).
        let leaf_index = builder.private_call.function_leaf_membership_witness.leaf_index;
        builder.private_call.function_leaf_membership_witness.leaf_index = leaf_index + 1;

        builder.failed();
    }

    #[test(should_fail_with="computed_contract_tree_root does not match purported_contract_tree_root")]
    fn private_function_incorrect_function_leaf_sibling_path_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set the first value of the sibling path to a wrong value (the correct value + 1).
        let sibling_path_0 = builder.private_call.function_leaf_membership_witness.sibling_path[0];
        builder.private_call.function_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

        builder.failed();
    }

    #[test(should_fail_with="note hash tree root mismatch")]
    fn native_read_request_bad_request() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_read_requests(2);

        // tweak read_request so it gives wrong root when paired with its sibling path
        let read_request = builder.private_call.public_inputs.read_requests.pop();
        builder.private_call.public_inputs.read_requests.push(SideEffect { value: read_request.value + 1, counter: read_request.counter });

        builder.failed();
    }

    #[test(should_fail_with="note hash tree root mismatch")]
    fn native_read_request_bad_leaf_index() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_read_requests(2);

        // tweak leaf index so it gives wrong root when paired with its request and sibling path
        let mut read_request_membership_witness = builder.private_call.read_request_membership_witnesses.pop();
        read_request_membership_witness.leaf_index += 1;
        builder.private_call.read_request_membership_witnesses.push(read_request_membership_witness);

        builder.failed();
    }

    #[test(should_fail_with="note hash tree root mismatch")]
    fn native_read_request_bad_sibling_path() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_read_requests(2);

        // tweak sibling path so it gives wrong root when paired with its request
        let mut read_request_membership_witness = builder.private_call.read_request_membership_witnesses.pop();
        read_request_membership_witness.sibling_path[1] += 1;
        builder.private_call.read_request_membership_witnesses.push(read_request_membership_witness);

        builder.failed();
    }

    #[test]
    fn native_no_read_requests_works() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        let public_inputs = builder.execute();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0].value, builder.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_one_read_requests_works() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_read_requests(1);

        let public_inputs = builder.execute();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0].value, builder.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_two_read_requests_works() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_read_requests(2);

        let public_inputs = builder.execute();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0].value, builder.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_max_read_requests_works() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_read_requests(MAX_READ_REQUESTS_PER_CALL);

        let public_inputs = builder.execute();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0].value, builder.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 0);
    }

    #[test]
    fn native_one_transient_read_requests_works() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_transient_read_requests(1);

        let public_inputs = builder.execute();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0].value, builder.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 1);
    }

    #[test]
    fn native_max_read_requests_one_transient_works() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_transient_read_requests(1);
        builder.private_call.append_read_requests(MAX_READ_REQUESTS_PER_CALL - 1);

        let public_inputs = builder.execute();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0].value, builder.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), 1);
    }

    #[test]
    fn native_max_read_requests_all_transient_works() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.append_transient_read_requests(MAX_READ_REQUESTS_PER_CALL);

        let public_inputs = builder.execute();

        validate_no_new_deployed_contract(public_inputs);
        // Check the first nullifier is hash of the signed tx request
        assert_eq(public_inputs.end.new_nullifiers[0].value, builder.tx_request.hash());

        // non-transient read requests are NOT forwarded
        assert_eq(array_length(public_inputs.end.read_requests), MAX_READ_REQUESTS_PER_CALL);
    }

    #[test]
    fn propagate_nullifier_key_validation_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new_constructor();

        let request = NullifierKeyValidationRequest { public_key: GrumpkinPoint { x: 1, y: 2 }, secret_key: GrumpkinPrivateKey { high: 3, low: 4 } };
        builder.private_call.public_inputs.nullifier_key_validation_requests.push(request);

        let public_inputs = builder.execute();

        assert_eq(array_length(public_inputs.end.nullifier_key_validation_requests), 1);

        let request_context = public_inputs.end.nullifier_key_validation_requests[0];
        assert_eq(request_context.public_key, request.public_key);
        assert_eq(request_context.secret_key, request.secret_key);
        assert_eq(
            request_context.contract_address, builder.private_call.public_inputs.call_context.storage_contract_address
        );
    }
}
