use crate::{common, private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer};
use dep::types::{
    abis::{
    private_kernel::private_call_data::PrivateCallData,
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs
},
    constants::MAX_NEW_NOTE_HASHES_PER_CALL, mocked::verify_private_function_proof,
    transaction::tx_request::TxRequest
};

struct PrivateKernelInitHints {
    note_hash_nullifier_counters: [u32; MAX_NEW_NOTE_HASHES_PER_CALL],
}

// Initialization struct for private inputs to the private kernel
struct PrivateKernelInitCircuitPrivateInputs {
    tx_request: TxRequest,
    private_call: PrivateCallData,
    hints: PrivateKernelInitHints,
}

impl PrivateKernelInitCircuitPrivateInputs {
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
        assert(
            entry_point_function_matches, "tx_request function_data must match call_stack_item function_data"
        );
        //
        // Ensure we are passing the correct arguments to the function.
        let args_match = tx_request.args_hash == call_stack_item.public_inputs.args_hash;
        assert(args_match, "noir function args passed to tx_request must match args in the call_stack_item");
        //
        // Ensure we are passing the correct tx context
        let tx_context_matches = tx_request.tx_context == call_stack_item.public_inputs.tx_context;
        assert(tx_context_matches, "tx_context in tx_request must match tx_context in call_stack_item");
    }

    fn validate_inputs(self) {
        let call_stack_item = self.private_call.call_stack_item;

        let function_data = call_stack_item.function_data;
        assert(function_data.is_private, "Private kernel circuit can only execute a private function");

        let call_context = call_stack_item.public_inputs.call_context;
        assert(call_context.is_delegate_call == false, "Users cannot make a delegatecall");
        assert(call_context.is_static_call == false, "Users cannot make a static call");
        // The below also prevents delegatecall/staticcall in the base case
        assert(
            call_context.storage_contract_address.eq(call_stack_item.contract_address), "Storage contract address must be that of the called contract"
        );
    }

    pub fn native_private_kernel_circuit_initial(self) -> PrivateKernelCircuitPublicInputs {
        let private_call_public_inputs = self.private_call.call_stack_item.public_inputs;

        self.validate_inputs();

        common::validate_private_call_data(self.private_call);

        self.validate_this_private_call_against_tx_request();

        assert(verify_private_function_proof(self.private_call.proof), "Invalid private function proof.");

        PrivateKernelCircuitPublicInputsComposer::new_from_tx_request(self.tx_request, private_call_public_inputs).compose(
            private_call_public_inputs,
            self.hints.note_hash_nullifier_counters,
            self.private_call.private_call_stack,
            self.private_call.public_call_stack
        ).finish()
    }
}

mod tests {
    use crate::private_kernel_init::{PrivateKernelInitHints, PrivateKernelInitCircuitPrivateInputs};
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, note_hash::NoteHash,
        nullifier_key_validation_request::NullifierKeyValidationRequest,
        private_kernel::private_call_data::PrivateCallData, read_request::ReadRequest,
        nullifier::Nullifier, side_effect::SideEffect
    },
        address::{AztecAddress, EthAddress}, constants::MAX_NEW_NOTE_HASHES_PER_CALL,
        grumpkin_point::GrumpkinPoint, messaging::l2_to_l1_message::L2ToL1Message,
        tests::private_call_data_builder::PrivateCallDataBuilder, transaction::tx_request::TxRequest,
        utils::arrays::array_length
    };

    struct PrivateKernelInitInputsBuilder {
        tx_request: TxRequest,
        private_call: PrivateCallDataBuilder,
        hints: PrivateKernelInitHints,
    }

    impl PrivateKernelInitInputsBuilder {
        pub fn new() -> Self {
            let private_call = PrivateCallDataBuilder::new();
            let tx_request = private_call.build_tx_request();
            let hints = PrivateKernelInitHints { note_hash_nullifier_counters: [0; MAX_NEW_NOTE_HASHES_PER_CALL] };

            PrivateKernelInitInputsBuilder { tx_request, private_call, hints }
        }

        pub fn execute(self) -> PrivateKernelCircuitPublicInputs {
            let kernel = PrivateKernelInitCircuitPrivateInputs { tx_request: self.tx_request, private_call: self.private_call.finish(), hints: self.hints };

            kernel.native_private_kernel_circuit_initial()
        }

        pub fn failed(self) {
            let _ = self.execute();
        }
    }

    #[test]
    fn deposit() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Logs for the private call.
        let encrypted_logs_hashes = [16, 36];
        let encrypted_log_preimages_length = [100, 75];
        let unencrypted_logs_hashes = [26, 46];
        let unencrypted_log_preimages_length = [50, 25];
        builder.private_call.set_encrypted_logs(encrypted_logs_hashes[0], encrypted_log_preimages_length[0]);
        builder.private_call.set_unencrypted_logs(
            unencrypted_logs_hashes[0],
            unencrypted_log_preimages_length[0]
        );
        builder.private_call.set_encrypted_logs(encrypted_logs_hashes[1], encrypted_log_preimages_length[1]);
        builder.private_call.set_unencrypted_logs(
            unencrypted_logs_hashes[1],
            unencrypted_log_preimages_length[1]
        );

        let public_inputs = builder.execute();

        // Check the first nullifier is hash of the signed tx request
        let tx_hash = builder.tx_request.hash();
        assert_eq(public_inputs.end.new_nullifiers[0].value, tx_hash);

        // Log preimages length should increase by `(un)encrypted_log_preimages_length` from private input
        assert_eq(
            public_inputs.end.encrypted_log_preimages_length, encrypted_log_preimages_length.reduce(|a, b| a + b)
        );
        assert_eq(
            public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length.reduce(|a, b| a + b)
        );
        assert_eq(public_inputs.end.encrypted_logs_hashes[0].value, encrypted_logs_hashes[0]);
        assert_eq(public_inputs.end.unencrypted_logs_hashes[0].value, unencrypted_logs_hashes[0]);
        assert_eq(public_inputs.end.encrypted_logs_hashes[1].value, encrypted_logs_hashes[1]);
        assert_eq(public_inputs.end.unencrypted_logs_hashes[1].value, unencrypted_logs_hashes[1]);
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_note_hash_read_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.public_inputs.note_hash_read_requests.extend_from_array(
            [
            ReadRequest::empty(),
            ReadRequest { value: 9123, counter: 1 }
        ]
        );

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_note_hashes() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.public_inputs.new_note_hashes.extend_from_array(
            [
            NoteHash { value: 0, counter: 0 },
            NoteHash { value: 9123, counter: 1 }
        ]
        );

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_nullifiers() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.public_inputs.new_nullifiers.extend_from_array(
            [
            Nullifier { value: 0, note_hash: 0, counter: 0 },
            Nullifier { value: 9123, note_hash: 0, counter: 1 }
        ]
        );

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_private_call_stack() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.public_inputs.private_call_stack_hashes.extend_from_array([0, 9123]);

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_public_call_stack() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.public_inputs.public_call_stack_hashes.extend_from_array([0, 9123]);

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_new_l2_to_l1_msgs() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.public_inputs.new_l2_to_l1_msgs.extend_from_array(
            [
            L2ToL1Message::empty(),
            L2ToL1Message { recipient: EthAddress::from_field(6), content: 9123 }
        ]
        );

        builder.failed();
    }

    #[test(should_fail_with = "invalid array")]
    fn input_validation_malformed_arrays_logs() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.public_inputs.encrypted_logs_hashes.extend_from_array(
            [
            SideEffect { value: 0, counter: 0 },
            SideEffect { value: 9123, counter: 1 }
        ]
        );

        builder.failed();
    }

    #[test(should_fail_with="Private kernel circuit can only execute a private function")]
    fn private_function_is_private_false_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set is_private in function data to false.
        builder.private_call.function_data.is_private = false;

        builder.failed();
    }

    #[test(should_fail_with="Users cannot make a static call")]
    fn private_function_static_call_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set is_static_call to true.
        builder.private_call.public_inputs.call_context.is_static_call = true;

        builder.failed();
    }

    #[test(should_fail_with="Users cannot make a delegatecall")]
    fn private_function_delegate_call_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set is_delegate_call to true.
        builder.private_call.public_inputs.call_context.is_delegate_call = true;

        builder.failed();
    }

    #[test(should_fail_with="Storage contract address must be that of the called contract")]
    fn private_function_incorrect_storage_contract_address_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set the storage_contract_address to a random scalar.
        builder.private_call.public_inputs.call_context.storage_contract_address = AztecAddress::from_field(356);

        builder.failed();
    }

    #[test(should_fail_with="computed contract address does not match expected one")]
    fn private_function_incorrect_function_leaf_index_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set the leaf index of the function leaf to a wrong value (the correct value + 1).
        let leaf_index = builder.private_call.function_leaf_membership_witness.leaf_index;
        builder.private_call.function_leaf_membership_witness.leaf_index = leaf_index + 1;

        builder.failed();
    }

    #[test(should_fail_with="computed contract address does not match expected one")]
    fn private_function_incorrect_function_leaf_sibling_path_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        // Set the first value of the sibling path to a wrong value (the correct value + 1).
        let sibling_path_0 = builder.private_call.function_leaf_membership_witness.sibling_path[0];
        builder.private_call.function_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

        builder.failed();
    }

    #[test(should_fail_with="computed contract address does not match expected one")]
    fn private_function_incorrect_contract_class_preimage_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        builder.private_call.contract_class_artifact_hash = builder.private_call.contract_class_artifact_hash + 1;
        builder.failed();
    }

    #[test(should_fail_with="computed contract address does not match expected one")]
    fn private_function_incorrect_partial_address_preimage_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        builder.private_call.salted_initialization_hash.inner = builder.private_call.salted_initialization_hash.inner + 1;
        builder.failed();
    }

    #[test(should_fail_with="computed contract address does not match expected one")]
    fn private_function_incorrect_address_preimage_fails() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        builder.private_call.public_keys_hash.inner = builder.private_call.public_keys_hash.inner + 1;
        builder.failed();
    }

    #[test]
    fn default_max_block_number() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        let public_inputs = builder.execute();

        assert(public_inputs.validation_requests.for_rollup.max_block_number.is_none());
    }

    #[test]
    fn propagate_max_block_number_request() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        builder.private_call.set_tx_max_block_number(42);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.validation_requests.for_rollup.max_block_number.unwrap(), 42);
    }

    #[test]
    fn propagate_note_hash_read_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        let storage_contract_address = builder.private_call.public_inputs.call_context.storage_contract_address;

        let request_0 = ReadRequest { value: 123, counter: 4567 };
        builder.private_call.public_inputs.note_hash_read_requests.push(request_0);
        let request_1 = ReadRequest { value: 777888, counter: 90 };
        builder.private_call.public_inputs.note_hash_read_requests.push(request_1);

        let public_inputs = builder.execute();

        let end_note_hash_read_requests = public_inputs.validation_requests.note_hash_read_requests;
        assert_eq(array_length(end_note_hash_read_requests), 2);

        let request_context = end_note_hash_read_requests[0];
        assert_eq(request_context.value, request_0.value);
        assert_eq(request_context.counter, request_0.counter);
        assert_eq(request_context.contract_address, storage_contract_address);

        let request_context = end_note_hash_read_requests[1];
        assert_eq(request_context.value, request_1.value);
        assert_eq(request_context.counter, request_1.counter);
        assert_eq(request_context.contract_address, storage_contract_address);
    }

    #[test]
    fn propagate_nullifier_read_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        let storage_contract_address = builder.private_call.public_inputs.call_context.storage_contract_address;

        let request_0 = ReadRequest { value: 123, counter: 4567 };
        builder.private_call.public_inputs.nullifier_read_requests.push(request_0);
        let request_1 = ReadRequest { value: 777888, counter: 90 };
        builder.private_call.public_inputs.nullifier_read_requests.push(request_1);

        let public_inputs = builder.execute();

        let end_nullifier_read_requests = public_inputs.validation_requests.nullifier_read_requests;
        assert_eq(array_length(end_nullifier_read_requests), 2);

        let request_context = end_nullifier_read_requests[0];
        assert_eq(request_context.value, request_0.value);
        assert_eq(request_context.counter, request_0.counter);
        assert_eq(request_context.contract_address, storage_contract_address);

        let request_context = end_nullifier_read_requests[1];
        assert_eq(request_context.value, request_1.value);
        assert_eq(request_context.counter, request_1.counter);
        assert_eq(request_context.contract_address, storage_contract_address);
    }

    #[test]
    fn propagate_nullifier_key_validation_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        let request = NullifierKeyValidationRequest { master_nullifier_public_key: GrumpkinPoint { x: 1, y: 2 }, app_nullifier_secret_key: 3 };
        builder.private_call.public_inputs.nullifier_key_validation_requests.push(request);

        let public_inputs = builder.execute();

        assert_eq(array_length(public_inputs.validation_requests.nullifier_key_validation_requests), 1);

        let request_context = public_inputs.validation_requests.nullifier_key_validation_requests[0];
        assert_eq(request_context.master_nullifier_public_key, request.master_nullifier_public_key);
        assert_eq(request_context.app_nullifier_secret_key, request.app_nullifier_secret_key);
        assert_eq(
            request_context.contract_address, builder.private_call.public_inputs.call_context.storage_contract_address
        );
    }
}
