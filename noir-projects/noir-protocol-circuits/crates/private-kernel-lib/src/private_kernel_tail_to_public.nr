use crate::components::{
    previous_kernel_validator::PreviousKernelValidator,
    tail_to_public_output_composer::TailToPublicOutputComposer,
    tail_to_public_output_validator::TailToPublicOutputValidator
};
use dep::types::{abis::{private_kernel_data::PrivateKernelData, kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs}};

struct PrivateKernelTailToPublicCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
}

impl PrivateKernelTailToPublicCircuitPrivateInputs {
    unconstrained fn generate_output(self) -> PublicKernelCircuitPublicInputs {
        TailToPublicOutputComposer::new(self.previous_kernel.public_inputs).finish()
    }

    pub fn execute(self) -> PublicKernelCircuitPublicInputs {
        // Generate output.
        let output = self.generate_output();

        // Validate inputs.
        PreviousKernelValidator::new(self.previous_kernel.public_inputs).validate_for_private_tail_to_public();
        if !std::runtime::is_unconstrained() {
            // verify/aggregate the previous kernel
            self.previous_kernel.verify();
        }

        // Validate output.
        if !dep::std::runtime::is_unconstrained() {
            TailToPublicOutputValidator::new(output, self.previous_kernel.public_inputs).validate();
        }

        output
    }
}

mod tests {
    use crate::private_kernel_tail_to_public::PrivateKernelTailToPublicCircuitPrivateInputs;
    use dep::types::constants::{DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, GENERATOR_INDEX__TSK_M};
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, gas::Gas,
        note_hash::{NoteHash, ScopedNoteHash}, nullifier::{Nullifier, ScopedNullifier},
        log_hash::{LogHash, NoteLogHash}
    },
        address::AztecAddress, hash::{silo_note_hash, silo_nullifier},
        tests::fixture_builder::FixtureBuilder, utils::{arrays::array_eq}, grumpkin_point::GrumpkinPoint
    };

    // TODO: Reduce the duplicated code/tests for PrivateKernelTailToPublicInputs and PrivateKernelTailInputs.
    struct PrivateKernelTailToPublicInputsBuilder {
        previous_kernel: FixtureBuilder,
    }

    impl PrivateKernelTailToPublicInputsBuilder {
        pub fn new() -> Self {
            let mut previous_kernel = FixtureBuilder::new();
            previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1_000_000, 1_000_000);
            previous_kernel.set_first_nullifier();
            previous_kernel.push_public_call_request(1, false);

            PrivateKernelTailToPublicInputsBuilder { previous_kernel }
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed
        // note_hashes for the given note_hashes.
        pub fn compute_output_note_hashes<N>(self, note_hashes: [ScopedNoteHash; N]) -> [NoteHash; N] {
            // First nullifier is tx hash.
            let tx_hash = self.previous_kernel.nullifiers.get_unchecked(0).value();
            let mut output = [NoteHash::empty(); N];
            for i in 0..N {
                output[i] = NoteHash {
                    value: silo_note_hash(note_hashes[i], tx_hash, i),
                    counter: 0, // Counter is cleared so it's not exposed to the public.
                };
            }
            output
        }

        pub fn compute_output_nullifiers<N>(_self: Self, nullifiers: [ScopedNullifier; N]) -> [Nullifier; N] {
            let mut output = [Nullifier::empty(); N];
            output[0].value = nullifiers[0].value();
            for i in 1..N {
                output[i] = Nullifier { value: silo_nullifier(nullifiers[i]), counter: 0, note_hash: 0 };
            }
            output
        }

        pub fn compute_output_note_logs<N>(_self: Self, logs: [NoteLogHash; N]) -> [LogHash; N] {
            let mut output = [LogHash::empty(); N];
            for i in 0..N {
                if logs[i].value != 0 {
                    output[i] = logs[i].expose_to_public(); // Counter is cleared so it's not exposed to the public.
                }
            }
            output
        }

        pub fn execute(&mut self) -> PublicKernelCircuitPublicInputs {
            let kernel = PrivateKernelTailToPublicCircuitPrivateInputs { previous_kernel: self.previous_kernel.to_private_kernel_data() };
            kernel.execute()
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }
    }

    #[test]
    fn ordering_of_note_hashes_and_nullifiers() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        builder.previous_kernel.append_note_hashes(10);
        builder.previous_kernel.append_nullifiers(10);

        let sorted_note_hashes = builder.previous_kernel.note_hashes.storage;
        let sorted_nullifiers = builder.previous_kernel.nullifiers.storage;

        let mut reversed_note_hashes = [ScopedNoteHash::empty(); 10];
        let mut reversed_nullifiers = [ScopedNullifier::empty(); 10];

        for i in 0..10 {
            reversed_note_hashes[9 - i] = builder.previous_kernel.note_hashes.pop();
            reversed_nullifiers[9 - i] = builder.previous_kernel.nullifiers.pop();
        }

        builder.previous_kernel.note_hashes.extend_from_array(reversed_note_hashes);
        builder.previous_kernel.nullifiers.extend_from_array(reversed_nullifiers);

        let public_inputs = builder.execute();

        let first_nullifier = builder.previous_kernel.nullifiers.get(0);
        assert_eq(public_inputs.end_non_revertible.nullifiers[0], first_nullifier.nullifier);
        let output_note_hashes = builder.compute_output_note_hashes(sorted_note_hashes);
        let output_nullifiers = builder.compute_output_nullifiers(sorted_nullifiers);
        for i in 0..10 {
            assert(public_inputs.end.note_hashes[i].eq(output_note_hashes[i]));
            assert(public_inputs.end.nullifiers[i].eq(output_nullifiers[i + 1]));
        }
    }

    #[test(should_fail_with="Private call stack must be empty when executing the tail circuit")]
    fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.add_private_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="Must have public calls when exporting public kernel data from the tail circuit")]
    fn no_public_calls_should_fail() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.public_call_requests = BoundedVec::new();
        builder.failed();
    }

    #[test]
    fn can_run_with_only_teardown() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.public_call_requests = BoundedVec::new();
        builder.previous_kernel.push_public_teardown_call_request(1, false);

        builder.succeeded();
    }

    #[test]
    fn split_nullifiers_into_non_revertible() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        // expect 3 non-revertible nullifiers: the tx nullifier + 2 new ones
        builder.previous_kernel.append_nullifiers(2);
        builder.previous_kernel.end_setup();

        // expect 2 revertible nullifiers
        builder.previous_kernel.append_nullifiers(2);

        let nullifiers = builder.previous_kernel.nullifiers.storage;
        let public_inputs = builder.execute();

        let output_nullifiers = builder.compute_output_nullifiers(nullifiers);

        assert(
            array_eq(
                public_inputs.end_non_revertible.nullifiers,
                [output_nullifiers[0], output_nullifiers[1], output_nullifiers[2]]
            )
        );

        assert(
            array_eq(
                public_inputs.end.nullifiers,
                [output_nullifiers[3], output_nullifiers[4]]
            )
        );

        assert_eq(public_inputs.end.gas_used, Gas::new(2 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE, 0));
        assert_eq(
            public_inputs.end_non_revertible.gas_used, Gas::new(3 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE, 0) + Gas::tx_overhead()
        );
    }

    #[test]
    fn split_note_hashes_into_non_revertible() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();

        // expect 2 non-revertible note hashes
        builder.previous_kernel.append_note_hashes_with_logs(2);
        builder.previous_kernel.end_setup();

        // expect 2 revertible note hashes
        builder.previous_kernel.append_note_hashes_with_logs(2);

        let note_hashes = builder.previous_kernel.note_hashes.storage;
        let note_logs = builder.previous_kernel.note_encrypted_logs_hashes.storage;
        let public_inputs = builder.execute();

        let siloed_note_hashes = builder.compute_output_note_hashes(note_hashes);
        let public_note_logs = builder.compute_output_note_logs(note_logs);

        assert(
            array_eq(
                public_inputs.end_non_revertible.note_hashes,
                [siloed_note_hashes[0], siloed_note_hashes[1]]
            )
        );

        assert(
            array_eq(
                public_inputs.end_non_revertible.note_encrypted_logs_hashes,
                [public_note_logs[0], public_note_logs[1]]
            )
        );

        assert(
            array_eq(
                public_inputs.end.note_hashes,
                [siloed_note_hashes[2], siloed_note_hashes[3]]
            )
        );

        assert(
            array_eq(
                public_inputs.end.note_encrypted_logs_hashes,
                [public_note_logs[2], public_note_logs[3]]
            )
        );

        let revertible_logs_len = (note_logs[2].length + note_logs[3].length) as u32;
        let non_revertible_logs_len = (note_logs[0].length + note_logs[1].length) as u32;

        assert_eq(
            public_inputs.end.gas_used, Gas::new(
                (2 * DA_BYTES_PER_FIELD + revertible_logs_len) * DA_GAS_PER_BYTE,
                0
            )
        );
        assert_eq(
            public_inputs.end_non_revertible.gas_used, Gas::new(
            (3 * DA_BYTES_PER_FIELD + non_revertible_logs_len) * DA_GAS_PER_BYTE,
            0
        )
            + Gas::tx_overhead()
        );
    }

    #[test(should_fail_with="Non empty note hash read requests")]
    fn non_empty_note_hash_read_requests() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_note_hashes(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_note_hash(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty nullifier read requests")]
    fn non_empty_nullifier_read_requests() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.append_nullifiers(3);
        let _void = builder.previous_kernel.add_read_request_for_pending_nullifier(1);
        builder.failed();
    }

    #[test(should_fail_with="Non empty key validation requests")]
    fn non_empty_key_validations() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let _void = builder.previous_kernel.add_request_for_key_validation(GrumpkinPoint::new(1, 2), 27, GENERATOR_INDEX__TSK_M);
        builder.failed();
    }

    #[test]
    fn empty_tx_consumes_teardown_limits_plus_fixed_gas() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        let public_inputs = builder.execute();

        let gas_for_first_nullifier = Gas::new(DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE, 0);
        let expected_non_revertible_gas_consumed = Gas::tx_overhead() + gas_for_first_nullifier;
        assert_eq(public_inputs.end_non_revertible.gas_used, expected_non_revertible_gas_consumed);
        let expected_revertible_gas_consumed = Gas::new(300, 300);
        assert_eq(public_inputs.end.gas_used, expected_revertible_gas_consumed);
    }

    #[test(should_fail_with="The gas used exceeds the gas limits")]
    fn gas_limits_are_enforced() {
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = Gas::new(300, 300);
        builder.previous_kernel.tx_context.gas_settings.gas_limits = Gas::new(1, 1);
        builder.failed();
    }

    #[test]
    fn propagate_fee_payer() {
        // Check that we carry forward if the fee payer is already set
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let fee_payer = AztecAddress::from_field(123);
        builder.previous_kernel.fee_payer = fee_payer;
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, fee_payer);

        // Check that the fee payer remains empty if unset
        let mut builder = PrivateKernelTailToPublicInputsBuilder::new();
        let public_inputs = builder.execute();
        assert_eq(public_inputs.fee_payer, AztecAddress::empty());
    }
}
