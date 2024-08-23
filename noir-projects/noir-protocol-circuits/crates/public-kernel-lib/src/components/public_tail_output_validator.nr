mod output_hints;

use crate::components::public_tail_output_validator::output_hints::{generate_output_hints, OutputHints, SiloedNoteHashHint};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PublicKernelCircuitPublicInputs},
    log_hash::{LogHash, ScopedLogHash}, note_hash::{NoteHash, ScopedNoteHash}, nullifier::Nullifier
},
    hash::silo_note_hash, messaging::l2_to_l1_message::ScopedL2ToL1Message,
    partial_state_reference::PartialStateReference,
    utils::arrays::{array_length, assert_combined_sorted_transformed_value_array_asc, assert_combined_deduped_array}
};

struct PublicTailOutputValidator {
    output: KernelCircuitPublicInputs,
    previous_kernel: PublicKernelCircuitPublicInputs,
    start_state: PartialStateReference,
    hints: OutputHints
}

impl PublicTailOutputValidator {
    pub fn new(
        output: KernelCircuitPublicInputs,
        previous_kernel: PublicKernelCircuitPublicInputs,
        start_state: PartialStateReference
    ) -> Self {
        let hints = generate_output_hints(previous_kernel, output);
        PublicTailOutputValidator { output, previous_kernel, start_state, hints }
    }

    pub fn validate(self) {
        self.validate_aggregated_data();

        // CombinedAccumulatedData
        self.validate_combined_siloed_note_hashes();
        self.validate_combined_data();
        self.validate_deduped_public_data_writes();
        self.validate_gas();
    }

    fn validate_aggregated_data(self) {
        assert_eq(
            self.output.rollup_validation_requests, self.previous_kernel.validation_requests.for_rollup, "mismatch rollup_validation_requests"
        );
        assert_eq(self.output.constants, self.previous_kernel.constants, "mismatch constants");
        assert_eq(self.output.start_state, self.start_state, "mismatch start_state");
        assert_eq(self.output.revert_code, self.previous_kernel.revert_code, "mismatch revert_code");
        assert_eq(self.output.fee_payer, self.previous_kernel.fee_payer, "mismatch fee_payer");
    }

    fn validate_combined_siloed_note_hashes(self) {
        let non_revertible = self.previous_kernel.end_non_revertible;
        let revertible = self.previous_kernel.end;
        let tx_hash = self.output.end.nullifiers[0];
        let output_note_hashes = self.output.end.note_hashes;
        let siloed_note_hash_hints = self.hints.siloed_note_hash_hints;
        for i in 0..output_note_hashes.len() {
            let output_note_hash = output_note_hashes[i];
            assert_eq(output_note_hash, siloed_note_hash_hints[i].siloed_note_hash);
            if output_note_hash != 0 {
                assert_eq(i, siloed_note_hash_hints[i].index);
            }
        }

        assert_combined_sorted_transformed_value_array_asc(
            non_revertible.note_hashes,
            revertible.note_hashes,
            siloed_note_hash_hints,
            |prev: ScopedNoteHash, out: SiloedNoteHashHint| {
                let siloed_value = if prev.counter() == 0 {
                    // If counter is zero, the note hash is either empty or is emitted from private and has been siloed in private_kernel_tail_to_public.
                    prev.value()
                } else {
                    silo_note_hash(prev, tx_hash, out.index)
                };
                out.siloed_note_hash == siloed_value
            },
            self.hints.sorted_note_hash_hints
        );
    }

    fn validate_combined_data(self) {
        let non_revertible = self.previous_kernel.end_non_revertible;
        let revertible = self.previous_kernel.end;

        // nullifiers
        assert_combined_sorted_transformed_value_array_asc(
            non_revertible.nullifiers,
            revertible.nullifiers,
            self.output.end.nullifiers,
            |prev: Nullifier, out: Field| out == prev.value,
            self.hints.sorted_nullifier_hints
        );

        // l2_to_l1_msgs
        assert_combined_sorted_transformed_value_array_asc(
            non_revertible.l2_to_l1_msgs,
            revertible.l2_to_l1_msgs,
            self.output.end.l2_to_l1_msgs,
            |prev: ScopedL2ToL1Message, out: ScopedL2ToL1Message| out == prev,
            self.hints.sorted_l2_to_l1_msg_hints
        );

        // note_encrypted_logs_hashes
        assert_combined_sorted_transformed_value_array_asc(
            non_revertible.note_encrypted_logs_hashes,
            revertible.note_encrypted_logs_hashes,
            self.output.end.note_encrypted_logs_hashes,
            |prev: LogHash, out: LogHash| out == prev,
            self.hints.sorted_note_encrypted_log_hash_hints
        );

        let note_encrypted_log_preimages_length = self.output.end.note_encrypted_logs_hashes.fold(0, |a, b: LogHash| a + b.length);
        assert_eq(self.output.end.note_encrypted_log_preimages_length, note_encrypted_log_preimages_length);

        // encrypted_logs_hashes
        assert_combined_sorted_transformed_value_array_asc(
            non_revertible.encrypted_logs_hashes,
            revertible.encrypted_logs_hashes,
            self.output.end.encrypted_logs_hashes,
            |prev: ScopedLogHash, out: ScopedLogHash| out == prev,
            self.hints.sorted_encrypted_log_hash_hints
        );

        let encrypted_log_preimages_length = self.output.end.encrypted_logs_hashes.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length);
        assert_eq(self.output.end.encrypted_log_preimages_length, encrypted_log_preimages_length);

        // unencrypted_logs_hashes
        assert_combined_sorted_transformed_value_array_asc(
            non_revertible.unencrypted_logs_hashes,
            revertible.unencrypted_logs_hashes,
            self.output.end.unencrypted_logs_hashes,
            |prev: ScopedLogHash, out: ScopedLogHash| out == prev,
            self.hints.sorted_unencrypted_log_hash_hints
        );

        let unencrypted_log_preimages_length = self.output.end.unencrypted_logs_hashes.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length);
        assert_eq(self.output.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length);
    }

    fn validate_deduped_public_data_writes(self) {
        assert_combined_deduped_array(
            self.previous_kernel.end_non_revertible.public_data_update_requests,
            self.previous_kernel.end.public_data_update_requests,
            self.hints.sorted_public_data_update_requests,
            self.output.end.public_data_update_requests,
            self.hints.deduped_public_data_update_request_hints
        );
    }

    fn validate_gas(self) {
        let gas_used = self.previous_kernel.end_non_revertible.gas_used + self.previous_kernel.end.gas_used;
        assert_eq(self.output.end.gas_used, gas_used, "incorrect output gas_used");
    }
}
