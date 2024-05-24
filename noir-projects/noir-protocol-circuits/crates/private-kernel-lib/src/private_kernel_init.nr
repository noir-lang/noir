use crate::{
    private_call_data_validator::PrivateCallDataValidator,
    private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer
};
use dep::types::{
    abis::{
    private_kernel::private_call_data::PrivateCallData,
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs
},
    constants::MAX_NEW_NOTE_HASHES_PER_CALL, transaction::tx_request::TxRequest
};

struct PrivateKernelInitHints {
    note_hash_nullifier_counters: [u32; MAX_NEW_NOTE_HASHES_PER_CALL],
    first_revertible_private_call_request_index: u64,
}

// Initialization struct for private inputs to the private kernel
struct PrivateKernelInitCircuitPrivateInputs {
    tx_request: TxRequest,
    private_call: PrivateCallData,
    hints: PrivateKernelInitHints,
}

impl PrivateKernelInitCircuitPrivateInputs {
    pub fn native_private_kernel_circuit_initial(self) -> PrivateKernelCircuitPublicInputs {
        // verify/aggregate the private call proof
        self.private_call.verify();

        let privateCallDataValidator = PrivateCallDataValidator::new(self.private_call);
        privateCallDataValidator.validate();
        privateCallDataValidator.validate_as_first_call(self.hints.first_revertible_private_call_request_index);
        privateCallDataValidator.validate_against_tx_request(self.tx_request);

        let private_call_public_inputs = self.private_call.call_stack_item.public_inputs;
        PrivateKernelCircuitPublicInputsComposer::new_from_tx_request(self.tx_request, private_call_public_inputs).compose(
            private_call_public_inputs,
            privateCallDataValidator.array_lengths,
            self.private_call.call_stack_item.contract_address,
            self.hints.note_hash_nullifier_counters,
            self.private_call.public_call_stack,
            self.private_call.public_teardown_call_request
        ).finish()
    }
}

mod tests {
    use crate::private_kernel_init::{PrivateKernelInitHints, PrivateKernelInitCircuitPrivateInputs};
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, note_hash::NoteHash,
        validation_requests::{KeyValidationRequest, KeyValidationRequestAndGenerator},
        private_kernel::private_call_data::PrivateCallData, read_request::ReadRequest,
        nullifier::Nullifier, log_hash::{ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash}
    },
        address::{AztecAddress, EthAddress},
        constants::{MAX_NEW_NOTE_HASHES_PER_CALL, GENERATOR_INDEX__NSK_M},
        grumpkin_point::GrumpkinPoint, messaging::l2_to_l1_message::L2ToL1Message,
        tests::private_call_data_builder::PrivateCallDataBuilder, transaction::tx_request::TxRequest,
        utils::arrays::{array_length, array_eq}
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
            let hints = PrivateKernelInitHints {
                note_hash_nullifier_counters: [0; MAX_NEW_NOTE_HASHES_PER_CALL],
                first_revertible_private_call_request_index: 0
            };

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
        builder.private_call.public_inputs.add_encrypted_log(encrypted_logs_hashes[0], encrypted_log_preimages_length[0]);
        builder.private_call.public_inputs.add_unencrypted_log(
            unencrypted_logs_hashes[0],
            unencrypted_log_preimages_length[0]
        );
        builder.private_call.public_inputs.add_encrypted_log(encrypted_logs_hashes[1], encrypted_log_preimages_length[1]);
        builder.private_call.public_inputs.add_unencrypted_log(
            unencrypted_logs_hashes[1],
            unencrypted_log_preimages_length[1]
        );

        let expected_unencrypted_logs = [
            builder.private_call.public_inputs.unencrypted_logs_hashes.storage[0], builder.private_call.public_inputs.unencrypted_logs_hashes.storage[1]
        ];
        let expected_encrypted_logs = [
            builder.private_call.public_inputs.encrypted_logs_hashes.storage[0], builder.private_call.public_inputs.encrypted_logs_hashes.storage[1]
        ];

        let public_inputs = builder.execute();

        // Check the first nullifier is hash of the signed tx request
        let tx_hash = builder.tx_request.hash();
        assert_eq(public_inputs.end.new_nullifiers[0].value(), tx_hash);
        assert(
            array_eq(
                public_inputs.end.encrypted_logs_hashes.map(|l: ScopedEncryptedLogHash| l.log_hash),
                expected_encrypted_logs
            )
        );
        assert(
            array_eq(
                public_inputs.end.unencrypted_logs_hashes.map(|l: ScopedLogHash| l.log_hash),
                expected_unencrypted_logs
            )
        );
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
        builder.private_call.public_inputs.set_tx_max_block_number(42);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.validation_requests.for_rollup.max_block_number.unwrap(), 42);
    }

    #[test]
    fn propagate_note_hash_read_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        let storage_contract_address = builder.private_call.public_inputs.call_context.storage_contract_address;

        builder.private_call.public_inputs.append_note_hash_read_requests(2);
        let new_read_requests = builder.private_call.public_inputs.note_hash_read_requests.storage;

        let public_inputs = builder.execute();

        let end_note_hash_read_requests = public_inputs.validation_requests.note_hash_read_requests;
        assert_eq(array_length(end_note_hash_read_requests), 2);

        let request = end_note_hash_read_requests[0];
        assert_eq(request.read_request, new_read_requests[0]);
        assert_eq(request.contract_address, storage_contract_address);

        let request = end_note_hash_read_requests[1];
        assert_eq(request.read_request, new_read_requests[1]);
        assert_eq(request.contract_address, storage_contract_address);
    }

    #[test]
    fn propagate_nullifier_read_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        let storage_contract_address = builder.private_call.public_inputs.call_context.storage_contract_address;

        builder.private_call.public_inputs.append_nullifier_read_requests(2);
        let requests = builder.private_call.public_inputs.nullifier_read_requests.storage;

        let public_inputs = builder.execute();

        let end_nullifier_read_requests = public_inputs.validation_requests.nullifier_read_requests;
        assert_eq(array_length(end_nullifier_read_requests), 2);

        let request = end_nullifier_read_requests[0];
        assert_eq(request.read_request, requests[0]);
        assert_eq(request.contract_address, storage_contract_address);

        let request = end_nullifier_read_requests[1];
        assert_eq(request.read_request, requests[1]);
        assert_eq(request.contract_address, storage_contract_address);
    }

    #[test]
    fn propagate_key_validation_requests() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        let request_0 = KeyValidationRequest { pk_m: GrumpkinPoint { x: 1, y: 2 }, sk_app: 3 };
        let request_and_generator_0 = KeyValidationRequestAndGenerator { request: request_0, sk_app_generator: GENERATOR_INDEX__NSK_M };
        builder.private_call.public_inputs.key_validation_requests_and_generators.push(request_and_generator_0);

        let public_inputs = builder.execute();

        assert_eq(
            array_length(public_inputs.validation_requests.scoped_key_validation_requests_and_generators), 1
        );

        let request = public_inputs.validation_requests.scoped_key_validation_requests_and_generators[0];
        assert_eq(request.request, request_and_generator_0);
        assert_eq(
            request.contract_address, builder.private_call.public_inputs.call_context.storage_contract_address
        );
    }

    #[test(should_fail_with = "Could not find note hash linked to note log.")]
    fn input_validation_note_log_not_linked() {
        let mut builder = PrivateKernelInitInputsBuilder::new();

        builder.private_call.public_inputs.note_encrypted_logs_hashes.extend_from_array([NoteLogHash { value: 9123, counter: 2, length: 2, note_hash_counter: 1 }]);

        builder.failed();
    }

    #[test]
    unconstrained fn propagate_fee_payer() {
        let mut builder = PrivateKernelInitInputsBuilder::new();
        let fee_payer = builder.private_call.public_inputs.call_context.storage_contract_address;
        builder.private_call.public_inputs.is_fee_payer = true;

        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer is not set if is_fee_payer is false
        let mut builder = PrivateKernelInitInputsBuilder::new();
        assert_eq(builder.private_call.public_inputs.is_fee_payer, false);
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());
    }
}
