mod tail_output_hints;

use crate::components::{
    tail_output_composer::meter_gas_used::meter_gas_used,
    tail_output_validator::{tail_output_hints::{generate_tail_output_hints, TailOutputHints}}
};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PrivateKernelCircuitPublicInputs},
    log_hash::{LogHash, NoteLogHash, ScopedEncryptedLogHash, ScopedLogHash}
},
    messaging::l2_to_l1_message::ScopedL2ToL1Message, traits::is_empty,
    utils::arrays::assert_exposed_sorted_transformed_value_array
};

struct TailOutputValidator {
    output: KernelCircuitPublicInputs,
    previous_kernel: PrivateKernelCircuitPublicInputs,
    hints: TailOutputHints,
}

impl TailOutputValidator {
    pub fn new(
        output: KernelCircuitPublicInputs,
        previous_kernel: PrivateKernelCircuitPublicInputs
    ) -> Self {
        let hints = generate_tail_output_hints(previous_kernel);
        TailOutputValidator { output, previous_kernel, hints }
    }

    pub fn validate(self) {
        self.validate_empty_values();
        self.validate_propagated_values();
        self.validate_propagated_sorted_values();
        self.validate_gas_used();
    }

    fn validate_empty_values(self) {
        assert(is_empty(self.output.start_state), "start_state must be empty");
        assert_eq(self.output.revert_code, 0, "revert_code must be empty");
    }

    fn validate_propagated_values(self) {
        assert_eq(self.output.constants, self.previous_kernel.constants, "mismatch constants");

        assert_eq(
            self.output.rollup_validation_requests, self.previous_kernel.validation_requests.for_rollup, "mismatch rollup_validation_requests"
        );

        assert_eq(self.output.fee_payer, self.previous_kernel.fee_payer, "mismatch fee_payer");

        // note_hashes
        let note_hashes = self.previous_kernel.end.note_hashes;
        for i in 0..note_hashes.len() {
            assert_eq(note_hashes[i].value(), self.output.end.note_hashes[i], "mismatch note_hashes");
        }

        // nullifiers
        let nullifiers = self.previous_kernel.end.nullifiers;
        for i in 0..nullifiers.len() {
            assert_eq(nullifiers[i].value(), self.output.end.nullifiers[i], "mismatch nullifiers");
        }

        // note_encrypted_logs_hashes
        assert_eq(
            self.previous_kernel.end.note_encrypted_logs_hashes.map(|log: NoteLogHash| log.expose_to_public()), self.output.end.note_encrypted_logs_hashes, "mismatch note_encrypted_logs_hashes"
        );

        // encrypted_logs_hashes
        assert_eq(
            self.previous_kernel.end.encrypted_logs_hashes.map(|log: ScopedEncryptedLogHash| log.expose_to_public()), self.output.end.encrypted_logs_hashes, "mismatch encrypted_logs_hashes"
        );
    }

    fn validate_propagated_sorted_values(self) {
        // l2_to_l1_msgs
        assert_exposed_sorted_transformed_value_array(
            self.previous_kernel.end.l2_to_l1_msgs,
            self.output.end.l2_to_l1_msgs,
            |prev: ScopedL2ToL1Message, out: ScopedL2ToL1Message| out == prev.expose_to_public(),
            self.hints.sorted_l2_to_l1_msg_hints
        );

        // unencrypted_log_hashes
        assert_exposed_sorted_transformed_value_array(
            self.previous_kernel.end.unencrypted_logs_hashes,
            self.output.end.unencrypted_logs_hashes,
            |prev: ScopedLogHash, out: ScopedLogHash| out == prev.expose_to_public(),
            self.hints.sorted_unencrypted_log_hash_hints
        );
    }

    fn validate_gas_used(self) {
        let gas_used = meter_gas_used(
            self.output.end,
            self.output.constants.tx_context.gas_settings
        );
        assert(self.output.end.gas_used == gas_used, "incorrect metered gas used");

        let limits = self.previous_kernel.constants.tx_context.gas_settings.gas_limits;
        assert(self.output.end.gas_used.within(limits), "The gas used exceeds the gas limits");
    }
}
