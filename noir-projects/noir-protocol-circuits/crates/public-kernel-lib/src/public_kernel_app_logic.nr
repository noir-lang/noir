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
    constants::{PUBLIC_KERNEL_SETUP_INDEX, PUBLIC_KERNEL_APP_LOGIC_INDEX, PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX}
};

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
    fn execute(self) -> PublicKernelCircuitPublicInputs {
        let phase = PublicKernelPhase.APP_LOGIC;

        let previous_kernel_validator = PreviousKernelValidator::new(self.previous_kernel);
        previous_kernel_validator.validate_phase(phase);
        previous_kernel_validator.validate_proof(ALLOWED_PREVIOUS_CIRCUITS);

        let call_data_validator = PublicCallDataValidator::new(self.public_call, PublicKernelPhase.APP_LOGIC);
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
    use crate::{public_kernel_app_logic::{PublicKernelAppLogicCircuitPrivateInputs, ALLOWED_PREVIOUS_CIRCUITS}};
    use dep::types::{
        abis::{
        gas::Gas, kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs,
        note_hash::{NoteHash, ScopedNoteHash}, log_hash::ScopedLogHash,
        nullifier::{Nullifier, ScopedNullifier},
        public_call_stack_item_compressed::PublicCallStackItemCompressed,
        public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest,
        read_request::ReadRequest
    },
        constants::{PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX, PUBLIC_KERNEL_APP_LOGIC_INDEX, BASE_ROLLUP_INDEX},
        contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
        address::AztecAddress, hash::compute_siloed_nullifier,
        tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq}, traits::is_empty
    };

    struct PublicKernelAppLogicCircuitPrivateInputsBuilder {
        previous_kernel: FixtureBuilder,
        previous_non_revertible: FixtureBuilder,
        public_call: FixtureBuilder,
    }

    impl PublicKernelAppLogicCircuitPrivateInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = FixtureBuilder::new().as_parent_contract().in_vk_tree(PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX);
            let previous_non_revertible = FixtureBuilder::new().as_parent_contract();
            let mut public_call = FixtureBuilder::new().is_public_function();
            public_call.value_offset = 9999; // Add an offset so that the mock data won't be the same as the values in previous_kernel.

            PublicKernelAppLogicCircuitPrivateInputsBuilder { previous_kernel, previous_non_revertible, public_call }
        }

        pub fn is_delegate_call(&mut self) -> Self {
            let _ = self.public_call.is_delegate_call();
            *self
        }

        pub fn to_public_data_reads<let N: u32>(self, contract_storage_reads: [StorageRead; N]) -> [PublicDataRead; N] {
            let storage_contract_address = self.public_call.storage_contract_address;
            contract_storage_reads.map(|r: StorageRead| PublicDataRead::from_contract_storage_read(storage_contract_address, r))
        }

        pub fn to_public_data_update_requests<let N: u32>(
            self,
            storage_update_requests: [StorageUpdateRequest; N]
        ) -> [PublicDataUpdateRequest; N] {
            let storage_contract_address = self.public_call.storage_contract_address;
            storage_update_requests.map(
                |r: StorageUpdateRequest|
                PublicDataUpdateRequest::from_contract_storage_update_request(storage_contract_address, r)
            )
        }

        pub fn to_siloed_nullifiers<let N: u32>(self, nullifiers: [ScopedNullifier; N]) -> [Nullifier; N] {
            let storage_contract_address = self.public_call.storage_contract_address;
            nullifiers.map(
                |n: ScopedNullifier| Nullifier {
                    value: compute_siloed_nullifier(storage_contract_address, n.value()),
                    note_hash: n.nullifier.note_hash,
                    counter: n.counter()
                }
            )
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            let public_call = self.public_call.to_public_call_data();
            // Adjust the call stack item hash for the current call in the previous iteration.
            let compressed = public_call.call_stack_item.get_compressed();
            self.previous_kernel.push_public_call_request(compressed);
            let mut previous_kernel = self.previous_kernel.to_public_kernel_data(true);
            previous_kernel.public_inputs.end_non_revertible = self.previous_non_revertible.to_public_accumulated_data();

            let kernel = PublicKernelAppLogicCircuitPrivateInputs { previous_kernel, public_call };

            kernel.execute()
        }

        pub fn get_call_request_item(self) -> PublicCallStackItemCompressed {
            self.public_call.to_public_call_stack_item().get_compressed()
        }

        pub fn failed_with_call_request_item(&mut self, item: PublicCallStackItemCompressed) {
            let public_call = self.public_call.to_public_call_data();
            self.previous_kernel.push_public_call_request(item);
            let previous_kernel = self.previous_kernel.to_public_kernel_data(true);
            let kernel = PublicKernelAppLogicCircuitPrivateInputs { previous_kernel, public_call };
            let _ = kernel.execute();
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

    #[test(should_fail_with="Cannot run unnecessary app logic circuit")]
    fn public_previous_kernel_private_previous_kernel_should_fail() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.previous_kernel = FixtureBuilder::new().in_vk_tree(PUBLIC_KERNEL_APP_LOGIC_INDEX);
        let public_call = builder.public_call.to_public_call_data();

        // the key difference in this test versus those that use builder.execute()
        // is that we do not add a public call request to the previous kernel.
        // this means that when we call `to_public_kernel_data` below,
        // it will say needs_app_logic is false

        let previous_kernel = builder.previous_kernel.to_public_kernel_data(true);

        let kernel = PublicKernelAppLogicCircuitPrivateInputs { previous_kernel, public_call };

        let _ = kernel.execute();
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_commitments() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        // Setup 1 non-revertible note hash and log on the previous kernel.
        builder.previous_non_revertible.append_note_hashes_with_logs(1);
        let prev_non_rev_notes = builder.previous_non_revertible.note_hashes.storage;
        let prev_non_rev_logs = builder.previous_non_revertible.to_public_accumulated_data().note_encrypted_logs_hashes;

        // Setup 2 revertible note hashes and logs on the previous kernel.
        builder.previous_kernel.append_note_hashes_with_logs(2);
        let prev_notes = builder.previous_kernel.note_hashes.storage;
        let prev_logs = builder.previous_kernel.to_public_accumulated_data().note_encrypted_logs_hashes;

        // Setup 2 note hashes and logs on the current public call.
        builder.public_call.append_note_hashes(2);
        let curr_notes = builder.public_call.note_hashes.storage;

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.end_non_revertible.note_hashes,
            [prev_non_rev_notes[0]]
        );
        assert_array_eq(
            public_inputs.end.note_hashes,
            [prev_notes[0], prev_notes[1], curr_notes[0], curr_notes[1]]
        );
        assert_array_eq(
            public_inputs.end_non_revertible.note_encrypted_logs_hashes,
            [prev_non_rev_logs[0]]
        );
        assert_array_eq(
            public_inputs.end.note_encrypted_logs_hashes,
            [prev_logs[0], prev_logs[1]]
        );
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_update_requests() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        // Setup 2 data writes on the previous kernel.
        builder.previous_kernel.append_public_data_update_requests(2);
        let previous = builder.previous_kernel.public_data_update_requests.storage;
        // Setup 2 data writes on the current public inputs.
        builder.public_call.append_contract_storage_update_requests(2);
        let current = builder.to_public_data_update_requests(builder.public_call.contract_storage_update_requests.storage);

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.end.public_data_update_requests,
            [previous[0], previous[1], current[0], current[1]]
        );
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_read_requests() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        // Setup 2 data reads on the previous kernel.
        builder.previous_kernel.append_public_data_read_requests(2);
        let previous = builder.previous_kernel.public_data_reads.storage;
        // Setup 2 data reads on the current public inputs.
        builder.public_call.append_contract_storage_read_requests(2);
        let current = builder.to_public_data_reads(builder.public_call.contract_storage_reads.storage);

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.validation_requests.public_data_reads,
            [previous[0], previous[1], current[0], current[1]]
        );
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_nullifiers() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        // Setup 2 new nullifiers on the previous kernel.
        builder.previous_kernel.append_nullifiers(2);
        let previous = builder.previous_kernel.nullifiers.storage.map(|n: ScopedNullifier| n.nullifier);
        // Setup 2 new nullifiers on the current public call.
        builder.public_call.append_nullifiers(2);
        let siloed = builder.to_siloed_nullifiers(builder.public_call.nullifiers.storage);

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.end.nullifiers,
            [previous[0], previous[1], siloed[0], siloed[1]]
        );
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_l2_to_l1_msg() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.append_l2_to_l1_msgs(1);
        let previous = builder.previous_kernel.l2_to_l1_msgs.storage;
        // Setup 1 new l2 to l1 message on the current public inputs.
        builder.public_call.append_l2_to_l1_msgs(1);
        let current = builder.public_call.l2_to_l1_msgs.storage;

        let public_inputs = builder.execute();

        assert_array_eq(public_inputs.end.l2_to_l1_msgs, [previous[0], current[0]]);
    }
    // TODO: Break up this test into smaller tests.
    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_public_kernel_logs() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
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

    #[test(should_fail_with="No contract storage update requests are allowed for static calls")]
    fn previous_public_kernel_fails_if_contract_storage_update_requests_on_static_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.is_static_call = true;
        builder.public_call.append_contract_storage_update_requests(1);

        builder.failed();
    }

    #[test(should_fail_with="curent contract address must not match storage contract address for delegate calls")]
    fn previous_public_kernel_fails_if_incorrect_storage_contract_on_delegate_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new().is_delegate_call();

        builder.public_call.contract_address = builder.public_call.storage_contract_address;

        builder.failed();
    }

    #[test(should_fail_with="note_hashes must be empty for static calls")]
    fn public_kernel_fails_creating_note_hashes_on_static_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.is_static_call = true;
        builder.public_call.append_note_hashes(1);

        builder.failed();
    }

    #[test(should_fail_with="nullifiers must be empty for static calls")]
    fn public_kernel_fails_creating_nullifiers_on_static_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();
        builder.public_call.is_static_call = true;
        builder.public_call.append_nullifiers(1);

        builder.failed();
    }

    #[test(should_fail_with="call stack item does not match item at the top of the call stack")]
    fn public_kernel_circuit_fails_on_incorrect_storage_contract_in_delegate_call() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new().is_delegate_call();

        let mut item = builder.get_call_request_item();
        // Set the storage contract address to be the contract address.
        item.call_context.storage_contract_address = builder.public_call.contract_address;

        builder.failed_with_call_request_item(item);
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_public_call_requests() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.append_public_call_requests(2);
        let prev_calls = builder.previous_kernel.public_call_requests.storage;

        builder.public_call.append_public_call_requests(2);
        let curr_calls = builder.public_call.public_call_requests.storage;

        let public_inputs = builder.execute();

        assert_array_eq(
            public_inputs.end.public_call_stack,
            [prev_calls[0], prev_calls[1], curr_calls[0], curr_calls[1]]
        );
    }

    #[test(should_fail_with="Cannot run app logic circuit before setup circuit")]
    fn previous_kernel_non_empty_non_revertible_public_call_requests_should_fail() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.previous_non_revertible.append_public_call_requests(1);
        builder.public_call.append_public_call_requests(1);

        builder.failed();
    }

    #[test]
    fn propagate_nullifier_non_existent_read_requests() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.append_nullifier_non_existent_read_requests(2);
        let requests = builder.public_call.nullifier_non_existent_read_requests.storage;

        let public_inputs = builder.execute();

        let end_requests = public_inputs.validation_requests.nullifier_non_existent_read_requests;
        assert_array_eq(end_requests, [requests[0], requests[1]]);
    }

    #[test]
    fn updates_revertible_gas_used() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        // Transaction gas limit is 1k
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1000, 1000);

        // Revertible has already used 300
        builder.previous_kernel.gas_used = Gas::new(300, 300);

        // This call starts with 700 gas left
        builder.public_call.start_gas_left = Gas::new(700, 700);

        // And uses 200, ending with 500 left
        builder.public_call.end_gas_left = Gas::new(500, 500);

        // So the updated gas used by revertible must go up by 200, and non-revertible must stay the same
        let output = builder.execute();
        assert_eq(output.end.gas_used, Gas::new(500, 500));
        assert_eq(output.end_non_revertible.gas_used, Gas::new(0, 0));
    }

    #[test(should_fail_with="Start gas for public phase does not match transaction gas left")]
    fn validates_start_gas() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.is_execution_request = true; // don't need to check start gas for nested calls
        builder.public_call.start_gas_left = Gas::new(100, 100);

        builder.failed();
    }

    #[test(should_fail_with="Transaction fee must be zero on setup and app phases")]
    fn validates_transaction_fee() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.transaction_fee = 10;

        builder.failed();
    }

    #[test]
    fn propagates_global_variables_if_empty() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.public_call.global_variables.block_number = 11;

        let public_inputs = builder.execute();

        assert_eq(public_inputs.constants.global_variables.block_number, 11);
    }

    #[test(should_fail_with="Global variables injected into the public call do not match constants")]
    fn validates_global_variables() {
        let mut builder = PublicKernelAppLogicCircuitPrivateInputsBuilder::new();

        builder.previous_kernel.global_variables.block_number = 10;
        builder.public_call.global_variables.block_number = 11;

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
        builder.public_call.revert_code = 1;
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
        builder.public_call.revert_code = 1;

        builder.previous_kernel.append_public_call_requests(2);

        builder.previous_kernel.append_public_teardown_call_requests(2);
        let teardown_calls = builder.previous_kernel.public_teardown_call_stack.storage;

        let public_inputs = builder.execute();

        assert_array_eq(public_inputs.end.public_call_stack, []);
        assert_array_eq(
            public_inputs.public_teardown_call_stack,
            [teardown_calls[0], teardown_calls[1]]
        );
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
