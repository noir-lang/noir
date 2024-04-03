use dep::types::abis::public_call_data::PublicCallData;
use dep::types::abis::kernel_data::PublicKernelData;
use dep::types::PublicKernelCircuitPublicInputs;
use dep::types::abis::kernel_circuit_public_inputs::PublicKernelCircuitPublicInputsBuilder;
use dep::types::utils::arrays::array_to_bounded_vec;
use crate::common;
use dep::std::unsafe;

struct PublicKernelAppLogicCircuitPrivateInputs {
    previous_kernel: PublicKernelData,
    public_call: PublicCallData,
}

impl PublicKernelAppLogicCircuitPrivateInputs {

    fn validate_inputs(self) {
        let needs_setup = self.previous_kernel.public_inputs.needs_setup();
        assert(needs_setup == false, "Cannot run app logic circuit before setup circuit");
        let needs_app_logic = self.previous_kernel.public_inputs.needs_app_logic();
        assert(needs_app_logic == true, "Cannot run unnecessary app logic circuit");
    }

    fn public_kernel_app_logic(self) -> PublicKernelCircuitPublicInputs {
        // construct the circuit outputs
        let mut public_inputs: PublicKernelCircuitPublicInputsBuilder = unsafe::zeroed();
        common::initialize_revert_code(self.previous_kernel, self.public_call, &mut public_inputs);

        // initialise the end state with our provided previous kernel state
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        // validate the inputs common to all invocation circumstances
        common::validate_inputs(self.public_call);

        // validate the inputs unique to having a previous public kernel
        self.validate_inputs();

        common::update_validation_requests(self.public_call, &mut public_inputs);

        if public_inputs.revert_code == 0 {
            // Pops the item from the call stack and validates it against the current execution.
            let call_request = public_inputs.end.public_call_stack.pop();
            common::validate_call_against_request(self.public_call, call_request);
            common::update_public_end_values(self.public_call, &mut public_inputs);
            common::accumulate_unencrypted_logs(
                self.public_call,
                self.previous_kernel.public_inputs.end.unencrypted_logs_hash,
                self.previous_kernel.public_inputs.end.unencrypted_log_preimages_length,
                &mut public_inputs
            );
        } else {
            let mut remaining_calls = array_to_bounded_vec(self.previous_kernel.public_inputs.end.public_call_stack);
            let reverted_call_request = remaining_calls.pop();
            // even though we reverted, we still need to make sure the correct call was made
            // but don't do the full `validate_call_against_request` because 
            // that makes a bunch of assertions that we don't want to make
            // e.g. that msg_sender is self in the case of internal.
            // We don't want to make those checks because we already know we reverted,
            // and we aren't updating the public end values, so we want this kernel circuit to solve.
            // So just check that the call request is the same as the one we expected.
            assert(
                reverted_call_request.hash == self.public_call.call_stack_item.hash(), "calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the call stack"
            );
        }

        public_inputs.finish()
    }
}

mod tests {
    use crate::{
        public_kernel_app_logic::PublicKernelAppLogicCircuitPrivateInputs,
        utils::{
        assert_eq_call_requests, assert_eq_public_data_reads, assert_eq_public_data_update_requests,
        compute_public_data_reads, compute_public_data_update_requests
    }
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest, read_request::ReadRequest,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash}
    },
        address::{AztecAddress, EthAddress}, contract_class_id::ContractClassId,
        hash::{compute_l2_to_l1_hash, compute_logs_hash, silo_note_hash, silo_nullifier},
        messaging::l2_to_l1_message::L2ToL1Message,
        tests::{fixture_builder::FixtureBuilder, public_call_data_builder::PublicCallDataBuilder},
        utils::arrays::{array_eq, array_length}
    };
    use dep::types::constants::{MAX_PUBLIC_DATA_READS_PER_CALL, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL};

    struct PublicKernelAppLogicCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        public_call: PublicCallDataBuilder,
    }

    impl PublicKernelAppLogicCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new();
            let public_call = PublicCallDataBuilder::new();

            PublicKernelAppLogicCircuitPrivateInputsBuilder { previous_kernel, public_call }
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
            let previous_kernel = self.previous_kernel.to_public_kernel_data(true);

            let kernel = PublicKernelAppLogicCircuitPrivateInputs { previous_kernel, public_call };

            kernel.public_kernel_app_logic()
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }
    }

    #[test]
    fn public_kernel_circuit_with_public_previous_kernel_should_succeed() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.succeeded();
    }

    // TODO: Find another way to test this. Currently it will crash because we are popping from an empty array:
    // The application panicked (crashed). Message:  Expected array index to fit in u64
    // #[test(should_fail_with="Public call stack can not be empty")]
    // fn public_previous_kernel_empty_public_call_stack_should_fail() {
    //     let builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
    //     let previous_kernel = builder.previous_kernel.finish();
    //     let public_call = builder.public_call.finish();
    //     let kernel = PublicKernelAppLogicCircuitPrivateInputs {
    //         previous_kernel,
    //         public_call,
    //     };
    //
    //     let _ = kernel.public_kernel_app_logic();
    // }

    #[test(should_fail_with="Cannot run unnecessary app logic circuit")]
    fn public_previous_kernel_private_previous_kernel_should_fail() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.previous_kernel = FixtureBuilder::new();
        let public_call = builder.public_call.finish();

        // the key difference in this test versus those that use builder.execute()
        // is that we do not add a public call request to the previous kernel.
        // this means that when we call `to_public_kernel_data` below, 
        // it will say needs_app_logic is false

        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);

        let kernel = PublicKernelAppLogicCircuitPrivateInputs { previous_kernel, public_call };

        let _ = kernel.public_kernel_app_logic();
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_commitments() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        let contract_address = builder.public_call.contract_address;
        // Setup 2 new note hashes on the previous kernel.
        builder.previous_kernel.append_new_note_hashes(2);
        let previous = builder.previous_kernel.new_note_hashes.storage;
        // Setup 2 new note hashes on the current public inputs.
        let current = [
            SideEffect { value: previous[1].value + 1, counter: 3 },
            SideEffect { value: previous[1].value + 2, counter: 4 }
        ];
        builder.public_call.public_inputs.new_note_hashes.extend_from_array(current);
        let siloed = current.map(|c: SideEffect| silo_note_hash(contract_address, c.value));
        let new_note_hashes = [
            previous[0], previous[1], SideEffect { value: siloed[0], counter: 3 }, SideEffect { value: siloed[1], counter: 4 }
        ];

        let public_inputs = builder.execute();

        assert(array_eq(public_inputs.end.new_note_hashes, new_note_hashes));
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_update_requests() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        // Setup 2 data writes on the previous kernel.
        builder.previous_kernel.append_public_data_update_requests(2);
        let previous = builder.previous_kernel.public_data_update_requests.storage;
        // Setup 2 data writes on the current public inputs.
        builder.public_call.append_update_requests(2);
        let current = builder.get_current_public_data_update_requests();
        let public_data_update_requests = [previous[0], previous[1], current[0], current[1]];

        let public_inputs = builder.execute();
        assert_eq_public_data_update_requests(
            public_inputs.end.public_data_update_requests,
            public_data_update_requests
        );
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_read_requests() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        // Setup 2 data reads on the previous kernel.
        builder.previous_kernel.append_public_data_read_requests(2);
        let previous = builder.previous_kernel.public_data_reads.storage;
        // Setup 2 data reads on the current public inputs.
        builder.public_call.append_public_data_read_requests(2);
        let current = builder.get_current_public_data_reads();
        let public_data_read_requests = [previous[0], previous[1], current[0], current[1]];
        let public_inputs = builder.execute();

        assert_eq_public_data_reads(
            public_inputs.validation_requests.public_data_reads,
            public_data_read_requests
        );
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_nullifiers() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        let contract_address = builder.public_call.contract_address;

        // Setup 2 new nullifiers on the previous kernel.
        builder.previous_kernel.append_new_nullifiers(2);
        let previous = builder.previous_kernel.new_nullifiers.storage;
        // Setup 2 new note hashes on the current public inputs.

        let current = [
            SideEffectLinkedToNoteHash { value: previous[1].value + 1, note_hash: 0, counter: 4 }, SideEffectLinkedToNoteHash { value: previous[1].value + 2, note_hash: 0, counter: 5 }
        ];

        let siloed = current.map(
            |current: SideEffectLinkedToNoteHash| 
        SideEffectLinkedToNoteHash { value: silo_nullifier(contract_address, current.value), note_hash: current.note_hash, counter: current.counter }
        );

        builder.public_call.public_inputs.new_nullifiers.extend_from_array(current);
        // There are 2 revertible nullifiers in the previous kernel.
        // The tx nullifier is part of the non-revertible nullifiers.
        let new_nullifiers = [previous[0], previous[1], siloed[0], siloed[1]];

        let public_inputs = builder.execute();

        assert(array_eq(public_inputs.end.new_nullifiers, new_nullifiers));
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_l2_to_l1_msg() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        let contract_address = builder.public_call.contract_address;
        let portal_contract_address = builder.public_call.portal_contract_address;
        // Setup 1 new l2 to l1 message on the previous kernel.
        let previous = [12345];

        builder.previous_kernel.new_l2_to_l1_msgs.extend_from_array(previous);
        // Setup 1 new l2 to l1 message on the current public inputs.
        let current = [L2ToL1Message { recipient: portal_contract_address, content: 67890 }];
        builder.public_call.public_inputs.new_l2_to_l1_msgs.extend_from_array(current);
        let tx_context = builder.previous_kernel.tx_context;
        let version = tx_context.version;
        let chain_id = tx_context.chain_id;
        let siloed = current.map(|c| compute_l2_to_l1_hash(contract_address, version, chain_id, c));
        let new_l2_to_l1_msgs = [previous[0], siloed[0]];

        let public_inputs = builder.execute();
        assert(array_eq(public_inputs.end.new_l2_to_l1_msgs, new_l2_to_l1_msgs));
    }
    // TODO: Break up this test into smaller tests.
    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_public_kernel_logs() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        // Logs for the current call stack.
        let unencrypted_logs_hash = 26;
        let unencrypted_log_preimages_length = 50;
        builder.public_call.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = 80;
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = 956;
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.set_encrypted_logs(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.set_unencrypted_logs(
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

    #[test(should_fail_with="No contract storage update requests are allowed for static calls")]
    fn previous_public_kernel_fails_if_contract_storage_update_requests_on_static_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.append_update_requests(1);

        builder.failed();
    }

    #[test(should_fail_with="curent contract address must not match storage contract address for delegate calls")]
    fn previous_public_kernel_fails_if_incorrect_storage_contract_on_delegate_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new().is_delegate_call();

        builder.public_call.contract_address = builder.public_call.public_inputs.call_context.storage_contract_address;

        builder.failed();
    }

    #[test(should_fail_with="new_note_hashes must be empty for static calls")]
    fn public_kernel_fails_creating_new_note_hashes_on_static_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.public_inputs.new_note_hashes.push(SideEffect { value: 1, counter: 0 });

        builder.failed();
    }

    #[test(should_fail_with="new_nullifiers must be empty for static calls")]
    fn public_kernel_fails_creating_new_nullifiers_on_static_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.public_inputs.new_nullifiers.push(SideEffectLinkedToNoteHash { value: 1, note_hash: 0, counter: 0 });

        builder.failed();
    }

    #[test(should_fail_with="call stack storage address does not match expected contract address for delegate calls")]
    fn public_kernel_circuit_fails_on_incorrect_storage_contract_in_delegate_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new().is_delegate_call();

        // Set the storage contract address to be the contract address.
        builder.public_call.public_inputs.call_context.storage_contract_address = builder.public_call.contract_address;

        builder.failed();
    }

    #[test(should_fail_with = "call requests length does not match the expected length")]
    fn incorrect_call_requests_length_fails() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests(2, false);
        // Remove one call request.
        let _ = builder.public_call.public_call_stack.pop();

        builder.failed();
    }

    #[test]
    fn only_valid_update_requests_should_be_propagated() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.append_update_requests(1);
        builder.public_call.append_empty_update_requests(1);
        builder.public_call.append_update_requests(1);
        let public_data_update_requests = builder.get_current_public_data_update_requests();

        let public_inputs = builder.execute();

        // Only the 2 valid reads should have been propagated.
        let expected = [public_data_update_requests[0], public_data_update_requests[2]];
        assert_eq_public_data_update_requests(public_inputs.end.public_data_update_requests, expected);
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_private_kernel() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.append_public_call_requests_for_regular_calls(2);
        let storage = builder.public_call.public_call_stack.storage;
        let public_call_stack = [storage[0], storage[1]];

        builder.public_call.append_update_requests(2);
        let storage = builder.get_current_public_data_update_requests();
        let update_requests = [storage[0], storage[1]];

        builder.public_call.append_public_data_read_requests(3);
        let storage = builder.get_current_public_data_reads();
        let read_requests = [storage[0], storage[1], storage[2]];

        let public_inputs = builder.execute();

        assert_eq_call_requests(public_inputs.end.public_call_stack, public_call_stack);
        assert_eq_public_data_update_requests(
            public_inputs.end.public_data_update_requests,
            update_requests
        );
        assert_eq_public_data_reads(
            public_inputs.validation_requests.public_data_reads,
            read_requests
        );
    }

    #[test]
    fn propagate_nullifier_non_existent_read_requests() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
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
}
