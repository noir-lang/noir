use dep::types::abis::public_call_data::PublicCallData;
use dep::types::abis::public_kernel_data::PublicKernelData;
use dep::types::PublicKernelCircuitPublicInputs;
use dep::types::abis::kernel_circuit_public_inputs::PublicKernelCircuitPublicInputsBuilder;
use dep::types::utils::arrays::array_to_bounded_vec;
use dep::types::constants::{PUBLIC_KERNEL_SETUP_INDEX, PUBLIC_KERNEL_APP_LOGIC_INDEX, PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX};
use crate::common;

global ALLOWED_PREVIOUS_CIRCUITS = [
    PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX,
    PUBLIC_KERNEL_SETUP_INDEX,
    PUBLIC_KERNEL_APP_LOGIC_INDEX,
];

struct PublicKernelAppLogicCircuitPrivateInputs {
    previous_kernel: PublicKernelData,
    public_call: PublicCallData,
}

impl PublicKernelAppLogicCircuitPrivateInputs {

    fn initialize_revert_code(self, circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder) {
        if self.previous_kernel.public_inputs.revert_code != 0 {
            circuit_outputs.revert_code = self.previous_kernel.public_inputs.revert_code;
        } else if self.public_call.call_stack_item.public_inputs.revert_code != 0 {
            circuit_outputs.revert_code = self.public_call.call_stack_item.public_inputs.revert_code;
        }
    }

    fn initialize_non_revertible_values(
        self,
        circuit_outputs: &mut PublicKernelCircuitPublicInputsBuilder
    ) {
        // initialise the end state with our provided previous kernel state
        common::initialize_non_revertible_values(self.previous_kernel, circuit_outputs);

        // We always pass the teardown call stack forward during app logic
        circuit_outputs.public_teardown_call_stack = array_to_bounded_vec(self.previous_kernel.public_inputs.public_teardown_call_stack);
    }

    fn validate_inputs(self) {
        let needs_setup = self.previous_kernel.public_inputs.needs_setup();
        assert(needs_setup == false, "Cannot run app logic circuit before setup circuit");
        let needs_app_logic = self.previous_kernel.public_inputs.needs_app_logic();
        assert(needs_app_logic == true, "Cannot run unnecessary app logic circuit");
    }

    fn public_kernel_app_logic(self) -> PublicKernelCircuitPublicInputs {
        if !dep::std::runtime::is_unconstrained() {
            // TODO(#7410) currently stubbed out until tube vk handled
            // self.previous_kernel.validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }
        // construct the circuit outputs
        let mut public_inputs = PublicKernelCircuitPublicInputsBuilder::empty();

        self.initialize_revert_code(&mut public_inputs);

        self.initialize_non_revertible_values(&mut public_inputs);

        // validate the inputs common to all invocation circumstances
        common::validate_inputs(self.public_call);

        // validate constants injected into the public call are correct or set them if this is the first public call
        common::initialize_from_or_validate_public_call_variables(self.previous_kernel, self.public_call, &mut public_inputs);

        // validate the inputs unique to having a previous public kernel
        self.validate_inputs();

        common::validate_start_gas(self.public_call, self.previous_kernel);
        common::validate_transaction_fee_is_zero(self.public_call);

        common::update_validation_requests(self.public_call, &mut public_inputs);

        common::update_revertible_gas_used(self.public_call, &mut public_inputs);

        if public_inputs.revert_code == 0 {
            common::initialize_end_values(self.previous_kernel, &mut public_inputs);
            // Pops the item from the call stack and validates it against the current execution.
            let call_request = public_inputs.end.public_call_stack.pop();
            common::validate_call_against_request(self.public_call, call_request);
            common::update_end_call_stack(self.public_call, &mut public_inputs);
            common::update_public_end_values(self.public_call, &mut public_inputs);
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
                reverted_call_request.hash == self.public_call.call_stack_item.get_compressed().hash(), "calculated public_kernel_inputs_hash does not match provided public_kernel_inputs_hash at the top of the call stack"
            );
        }

        public_inputs.finish()
    }
}

mod tests {
    use crate::{
        public_kernel_app_logic::{PublicKernelAppLogicCircuitPrivateInputs, ALLOWED_PREVIOUS_CIRCUITS},
        utils::{
        assert_eq_call_requests, assert_eq_public_data_reads, assert_eq_public_data_update_requests,
        compute_public_data_reads, compute_public_data_update_requests
    }
    };
    use dep::types::{
        abis::{
        gas::Gas, kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs,
        note_hash::{NoteHash, ScopedNoteHash}, log_hash::ScopedLogHash,
        nullifier::{Nullifier, ScopedNullifier}, public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest, read_request::ReadRequest
    },
        address::{AztecAddress, EthAddress}, contract_class_id::ContractClassId,
        hash::{compute_l2_to_l1_hash, compute_siloed_note_hash, compute_siloed_nullifier},
        messaging::l2_to_l1_message::L2ToL1Message,
        tests::{fixture_builder::FixtureBuilder, public_call_data_builder::PublicCallDataBuilder},
        utils::arrays::{array_eq, array_length}, traits::is_empty
    };
    use dep::types::constants::{
        MAX_PUBLIC_DATA_READS_PER_CALL, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
        PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX, PUBLIC_KERNEL_APP_LOGIC_INDEX, BASE_ROLLUP_INDEX
    };

    struct PublicKernelAppLogicCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        public_call: PublicCallDataBuilder,
    }

    impl PublicKernelAppLogicCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new().as_parent_contract().in_vk_tree(PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX);
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
            let hash = public_call.call_stack_item.get_compressed().hash();
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
        builder.previous_kernel = FixtureBuilder::new().in_vk_tree(PUBLIC_KERNEL_APP_LOGIC_INDEX);
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
        // Setup 2 new note hashes and logs on the previous kernel.
        builder.previous_kernel.append_note_hashes_with_logs(2);
        let previous = builder.previous_kernel.note_hashes.storage.map(|n: ScopedNoteHash| n.note_hash);
        let prev_data = builder.previous_kernel.to_public_accumulated_data();
        let prev_note_logs = prev_data.note_encrypted_logs_hashes;
        // Setup 2 new note hashes on the current public inputs.
        let current = [
            NoteHash { value: previous[1].value + 1, counter: 5 },
            NoteHash { value: previous[1].value + 2, counter: 6 }
        ];
        builder.public_call.public_inputs.note_hashes.extend_from_array(current);
        let siloed = current.map(|c: NoteHash| compute_siloed_note_hash(contract_address, c.value));
        let note_hashes = [
            previous[0], previous[1], NoteHash { value: siloed[0], counter: 5 }, NoteHash { value: siloed[1], counter: 6 }
        ];

        let public_inputs = builder.execute();

        assert(array_eq(public_inputs.end.note_hashes, note_hashes));
        assert(
            array_eq(
                public_inputs.end.note_encrypted_logs_hashes,
                [prev_note_logs[0], prev_note_logs[1]]
            )
        );
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
        builder.previous_kernel.append_nullifiers(2);
        let previous = builder.previous_kernel.nullifiers.storage.map(|n: ScopedNullifier| n.nullifier);
        // Setup 2 new note hashes on the current public inputs.

        let current = [
            Nullifier { value: previous[1].value + 1, note_hash: 0, counter: 4 }, Nullifier { value: previous[1].value + 2, note_hash: 0, counter: 5 }
        ];
        builder.public_call.public_inputs.nullifiers.extend_from_array(current);

        let siloed = current.map(
            |current: Nullifier|
        Nullifier { value: compute_siloed_nullifier(contract_address, current.value), note_hash: current.note_hash, counter: current.counter }
        );

        // There are 2 revertible nullifiers in the previous kernel.
        // The tx nullifier is part of the non-revertible nullifiers.
        let nullifiers = [previous[0], previous[1], siloed[0], siloed[1]];

        let public_inputs = builder.execute();

        assert(array_eq(public_inputs.end.nullifiers, nullifiers));
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_l2_to_l1_msg() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        let contract_address = builder.public_call.contract_address;
        let portal_contract_address = EthAddress::from_field(0xdead);
        // Setup 1 new l2 to l1 message on the previous kernel.
        let previous = [12345];

        builder.previous_kernel.add_l2_to_l1_message(previous[0], portal_contract_address);
        // Setup 1 new l2 to l1 message on the current public inputs.
        let current = [L2ToL1Message { recipient: portal_contract_address, content: 67890, counter: 0 }];
        builder.public_call.public_inputs.l2_to_l1_msgs.extend_from_array(current);
        let tx_context = builder.previous_kernel.tx_context;
        let version = tx_context.version;
        let chain_id = tx_context.chain_id;
        let siloed = current.map(
            |c: L2ToL1Message| compute_l2_to_l1_hash(contract_address, c.recipient, c.content, version, chain_id)
        );
        let l2_to_l1_msgs = [previous[0], siloed[0]];

        let public_inputs = builder.execute();
        assert(array_eq(public_inputs.end.l2_to_l1_msgs, l2_to_l1_msgs));
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
        builder.previous_kernel.add_encrypted_log_hash(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.add_unencrypted_log_hash(
            prev_unencrypted_logs_hash,
            prev_unencrypted_log_preimages_length
        );
        let prev_data = builder.previous_kernel.to_public_accumulated_data();

        let expected_unencrypted_logs = [
            prev_data.unencrypted_logs_hashes[0], ScopedLogHash {
                log_hash: builder.public_call.public_inputs.unencrypted_logs_hashes.storage[0],
                contract_address: builder.public_call.contract_address
            }
        ];
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

    #[test(should_fail_with="note_hashes must be empty for static calls")]
    fn public_kernel_fails_creating_note_hashes_on_static_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.public_inputs.note_hashes.push(NoteHash { value: 1, counter: 0 });

        builder.failed();
    }

    #[test(should_fail_with="nullifiers must be empty for static calls")]
    fn public_kernel_fails_creating_nullifiers_on_static_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.public_inputs.nullifiers.push(Nullifier { value: 1, note_hash: 0, counter: 0 });

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

        let request = end_requests[0];
        assert_eq(request.read_request, request_0);
        assert_eq(request.contract_address, storage_contract_address);

        let request = end_requests[1];
        assert_eq(request.read_request, request_1);
        assert_eq(request.contract_address, storage_contract_address);
    }

    #[test]
    fn updates_revertible_gas_used() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        // Transaction gas limit is 1k
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1000, 1000);

        // Revertible has already used 300
        builder.previous_kernel.gas_used = Gas::new(300, 300);

        // This call starts with 700 gas left
        builder.public_call.public_inputs.start_gas_left = Gas::new(700, 700);

        // And uses 200, ending with 500 left
        builder.public_call.public_inputs.end_gas_left = Gas::new(500, 500);

        // So the updated gas used by revertible must go up by 200, and non-revertible must stay the same
        let output = builder.execute();
        assert_eq(output.end.gas_used, Gas::new(500, 500));
        assert_eq(output.end_non_revertible.gas_used, Gas::new(0, 0));
    }

    #[test(should_fail_with="Start gas for public phase does not match transaction gas left")]
    fn validates_start_gas() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.is_execution_request = true; // don't need to check start gas for nested calls
        builder.public_call.public_inputs.start_gas_left = Gas::new(100, 100);

        builder.failed();
    }

    #[test(should_fail_with="Transaction fee must be zero on setup and app phases")]
    fn validates_transaction_fee() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.public_inputs.transaction_fee = 10;

        builder.failed();
    }

    #[test]
    fn propagates_global_variables_if_empty() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.public_inputs.global_variables.block_number = 11;

        let public_inputs = builder.execute();

        assert_eq(public_inputs.constants.global_variables.block_number, 11);
    }

    #[test(should_fail_with="Global variables injected into the public call do not match constants")]
    fn validates_global_variables() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.global_variables.block_number = 10;
        builder.public_call.public_inputs.global_variables.block_number = 11;

        builder.failed();
    }

    #[test]
    unconstrained fn propagate_fee_payer() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        let fee_payer = AztecAddress::from_field(123);
        builder.previous_kernel.set_fee_payer(fee_payer);
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer is not set if is_fee_payer is false
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        assert_eq(is_empty(builder.previous_kernel.fee_payer), true);
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());
    }

    #[test]
    unconstrained fn correctly_updates_revert_code() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        let public_inputs = builder.execute();
        assert_eq(public_inputs.revert_code, 0);

        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.revert_code = 1;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.revert_code, 1);

        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.previous_kernel.revert_code = 1;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.revert_code, 1);
    }

    #[test]
    fn clears_stack_if_current_call_revert_code_is_set() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.public_inputs.revert_code = 1;
        let public_call = builder.public_call.finish();
        let hash = public_call.call_stack_item.get_compressed().hash();

        builder.previous_kernel.push_public_call_request(hash, false);
        builder.previous_kernel.push_public_call_request(hash, false);

        builder.previous_kernel.push_public_teardown_call_request(hash, false);
        builder.previous_kernel.push_public_teardown_call_request(hash, false);

        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);
        let kernel = PublicKernelAppLogicCircuitPrivateInputs { previous_kernel, public_call };

        let public_inputs = kernel.public_kernel_app_logic();

        assert_eq(array_length(public_inputs.end.public_call_stack), 0);
        assert_eq(array_length(public_inputs.public_teardown_call_stack), 2);
    }

    #[test]
    fn valid_previous_kernel() {
        for i in 0..ALLOWED_PREVIOUS_CIRCUITS.len() {
            let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
            builder.previous_kernel = builder.previous_kernel.in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS[i]);

            let _res = builder.execute();
        }
    }
    // TODO(#7410) we need the tube vk to reinstate this
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_kernel() {
    //     let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
    //     builder.previous_kernel = builder.previous_kernel.in_vk_tree(BASE_ROLLUP_INDEX);
    //     let _res = builder.execute();
    // }
}
