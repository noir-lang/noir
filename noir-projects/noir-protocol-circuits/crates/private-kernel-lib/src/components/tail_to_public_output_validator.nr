mod tail_to_public_output_hints;

use crate::components::{
    tail_to_public_output_composer::meter_gas_used::{meter_gas_used_non_revertible, meter_gas_used_revertible},
    tail_to_public_output_validator::tail_to_public_output_hints::{generate_tail_to_public_output_hints, TailToPublicOutputHints}
};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputs, PublicKernelCircuitPublicInputs},
    log_hash::{LogHash, ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash}, note_hash::ScopedNoteHash,
    nullifier::{Nullifier, ScopedNullifier}, public_call_request::PublicCallRequest
},
    messaging::l2_to_l1_message::ScopedL2ToL1Message, address::AztecAddress,
    traits::{is_empty, is_empty_array},
    utils::arrays::{
    assert_split_sorted_transformed_value_arrays_asc, assert_split_sorted_transformed_value_arrays_desc,
    assert_split_transformed_value_arrays, validate_array
}
};

struct TailToPublicOutputValidator {
    output: PublicKernelCircuitPublicInputs,
    previous_kernel: PrivateKernelCircuitPublicInputs,
    hints: TailToPublicOutputHints,
}

impl TailToPublicOutputValidator {
    pub fn new(
        output: PublicKernelCircuitPublicInputs,
        previous_kernel: PrivateKernelCircuitPublicInputs
    ) -> Self {
        let hints = generate_tail_to_public_output_hints(previous_kernel);
        TailToPublicOutputValidator { output, previous_kernel, hints }
    }

    pub fn validate(self) {
        self.validate_empty_values();
        self.validate_propagated_values();
        self.validate_propagated_split_values();
        self.validate_propagated_sorted_values();
        self.validate_gas_used();
    }

    fn validate_empty_values(self) {
        assert_eq(self.output.revert_code, 0, "revert_code must be empty");

        assert(
            is_empty_array(self.output.end_non_revertible.public_data_update_requests), "non-revertible public_data_update_requests must be empty"
        );
        assert(
            is_empty_array(self.output.end.public_data_update_requests), "revertible public_data_update_requests must be empty"
        );

        // public_data_update_requests
        assert(
            is_empty_array(self.output.end_non_revertible.public_data_update_requests), "unexpected non-revertible public_data_update_requests"
        );
        assert(
            is_empty_array(self.output.end.public_data_update_requests), "unexpected revertible public_data_update_requests"
        );
    }

    fn validate_propagated_values(self) {
        assert_eq(self.output.constants, self.previous_kernel.constants, "mismatch constants");

        assert_eq(
            self.output.validation_requests.for_rollup, self.previous_kernel.validation_requests.for_rollup, "mismatch rollup_validation_requests"
        );

        assert_eq(self.output.fee_payer, self.previous_kernel.fee_payer, "mismatch fee_payer");

        // public_teardown_call_stack
        assert_eq(
            self.output.public_teardown_call_stack[0].item, self.previous_kernel.public_teardown_call_request.item, "mismatch public_teardown_call_request"
        );
        assert_eq(
            self.output.public_teardown_call_stack[0].counter, 0, "cannot expose teardown call request counter"
        );
        let teardown_call_stack = self.output.public_teardown_call_stack;
        for i in 1..teardown_call_stack.len() {
            assert(is_empty(teardown_call_stack[i]), "unexpected teardown call request");
        }
    }

    fn validate_propagated_split_values(self) {
        let split_counter = self.previous_kernel.min_revertible_side_effect_counter;
        let prev_data = self.previous_kernel.end;
        let output_non_revertible = self.output.end_non_revertible;
        let output_revertible = self.output.end;

        // note_hashes
        assert_split_transformed_value_arrays(
            prev_data.note_hashes,
            output_non_revertible.note_hashes,
            output_revertible.note_hashes,
            |prev: ScopedNoteHash, out: ScopedNoteHash| out == prev.expose_to_public(),
            split_counter
        );

        // nullifiers
        assert_split_transformed_value_arrays(
            prev_data.nullifiers,
            output_non_revertible.nullifiers,
            output_revertible.nullifiers,
            |prev: ScopedNullifier, out: Nullifier| out == prev.expose_to_public(),
            split_counter
        );

        // note_encrypted_logs_hashes
        assert_split_transformed_value_arrays(
            prev_data.note_encrypted_logs_hashes,
            output_non_revertible.note_encrypted_logs_hashes,
            output_revertible.note_encrypted_logs_hashes,
            |prev: NoteLogHash, out: LogHash| out == prev.expose_to_public(),
            split_counter
        );

        // encrypted_logs_hashes
        assert_split_transformed_value_arrays(
            prev_data.encrypted_logs_hashes,
            output_non_revertible.encrypted_logs_hashes,
            output_revertible.encrypted_logs_hashes,
            |prev: ScopedEncryptedLogHash, out: ScopedLogHash| out == prev.expose_to_public(),
            split_counter
        );
    }

    fn validate_propagated_sorted_values(self) {
        let split_counter = self.previous_kernel.min_revertible_side_effect_counter;
        let prev_data = self.previous_kernel.end;
        let output_non_revertible = self.output.end_non_revertible;
        let output_revertible = self.output.end;
        let hints = self.hints;

        // l2_to_l1_msgs
        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.l2_to_l1_msgs,
            prev_data.l2_to_l1_msgs.map(|log: ScopedL2ToL1Message| log.expose_to_public()),
            split_counter,
            output_non_revertible.l2_to_l1_msgs,
            output_revertible.l2_to_l1_msgs,
            hints.sorted_l2_to_l1_msg_hints
        );

        // unencrypted_logs_hashes
        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.unencrypted_logs_hashes,
            prev_data.unencrypted_logs_hashes.map(|log: ScopedLogHash| log.expose_to_public()),
            split_counter,
            output_non_revertible.unencrypted_logs_hashes,
            output_revertible.unencrypted_logs_hashes,
            hints.sorted_unencrypted_log_hash_hints
        );

        // public_call_requests
        assert_split_sorted_transformed_value_arrays_desc(
            prev_data.public_call_requests,
            prev_data.public_call_requests.map(|cr: PublicCallRequest| cr.expose_to_public()),
            split_counter,
            output_non_revertible.public_call_stack,
            output_revertible.public_call_stack,
            hints.sorted_public_call_request_hints
        )
    }

    fn validate_gas_used(self) {
        let gas_used = meter_gas_used_non_revertible(self.output.end_non_revertible);
        assert(
            self.output.end_non_revertible.gas_used == gas_used, "incorrect metered non-revertible gas used"
        );

        let gas_used = meter_gas_used_revertible(
            self.output.end,
            self.output.constants.tx_context.gas_settings.teardown_gas_limits
        );
        assert(self.output.end.gas_used == gas_used, "incorrect metered revertible gas used");

        let limits = self.previous_kernel.constants.tx_context.gas_settings.gas_limits;
        let total_gas_used = self.output.end_non_revertible.gas_used + self.output.end.gas_used;
        assert(total_gas_used.within(limits), "The gas used exceeds the gas limits");
    }
}
