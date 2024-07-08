mod tail_to_public_output_hints;

use crate::components::{
    tail_output_validator::validate_value_transformation::{validate_transformed_values, validate_value_transformation},
    tail_to_public_output_validator::tail_to_public_output_hints::{generate_tail_to_public_output_hints, TailToPublicOutputHints}
};
use dep::types::{
    abis::{
    call_request::CallRequest,
    kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputs, PublicKernelCircuitPublicInputs},
    log_hash::{LogHash, ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash}, nullifier::Nullifier
},
    hash::{
    silo_encrypted_log_hash, silo_l2_to_l1_message, silo_note_hash, silo_nullifier,
    silo_unencrypted_log_hash
},
    traits::{Empty, is_empty_array},
    utils::arrays::{assert_split_sorted_transformed_value_arrays_asc, assert_split_sorted_transformed_value_arrays_desc}
};

struct TailToPublicOutputValidator {
    output: PublicKernelCircuitPublicInputs,
    previous_kernel: PrivateKernelCircuitPublicInputs,
}

impl TailToPublicOutputValidator {
    pub fn new(
        output: PublicKernelCircuitPublicInputs,
        previous_kernel: PrivateKernelCircuitPublicInputs
    ) -> Self {
        TailToPublicOutputValidator { output, previous_kernel }
    }

    pub fn validate<N>(self) {
        let hints = generate_tail_to_public_output_hints(self.previous_kernel);
        self.validate_empty_values();
        self.validate_propagated_values();
        self.validate_propagated_sorted_siloed_values(hints);
        self.validate_gas_limits();
    }

    fn validate_empty_values(self) {
        assert_eq(self.output.revert_code, 0, "revert_code must be empty");

        assert(
            is_empty_array(self.output.end_non_revertible.public_data_update_requests), "non-revertible public_data_update_requests must be empty"
        );
        assert(
            is_empty_array(self.output.end.public_data_update_requests), "revertible public_data_update_requests must be empty"
        );
    }

    fn validate_propagated_values(self) {
        assert_eq(self.output.constants, self.previous_kernel.constants, "mismatch constants");

        assert_eq(
            self.output.validation_requests.for_rollup, self.previous_kernel.validation_requests.for_rollup, "mismatch rollup_validation_requests"
        );

        assert_eq(self.output.fee_payer, self.previous_kernel.fee_payer, "mismatch fee_payer");
    }

    fn validate_propagated_sorted_siloed_values(self, hints: TailToPublicOutputHints) {
        let split_counter = self.previous_kernel.min_revertible_side_effect_counter;
        assert(split_counter != 0, "min_revertible_side_effect_counter must not be 0");

        let prev_data = self.previous_kernel.end;
        let output_non_revertible = self.output.end_non_revertible;
        let output_revertible = self.output.end;

        // note_hashes
        let first_nullifier = output_non_revertible.nullifiers[0].value;
        let unsiloed_note_hashes = prev_data.note_hashes;
        for i in 0..unsiloed_note_hashes.len() {
            let siloed_note_hash = silo_note_hash(unsiloed_note_hashes[i], first_nullifier, i);
            assert_eq(hints.siloed_note_hashes[i].value, siloed_note_hash, "mismatch siloed note hashes");
            assert_eq(hints.siloed_note_hashes[i].counter, 0, "cannot expose note hash counter");
        }

        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.note_hashes,
            hints.siloed_note_hashes,
            split_counter,
            output_non_revertible.note_hashes,
            output_revertible.note_hashes,
            hints.sorted_note_hash_hints
        );

        // nullifiers
        validate_value_transformation(
            prev_data.nullifiers,
            hints.siloed_nullifiers,
            |sn, n: Nullifier| (n.value == silo_nullifier(sn)) & (n.counter == 0) & (n.note_hash == 0)
        );
        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.nullifiers,
            hints.siloed_nullifiers,
            split_counter,
            output_non_revertible.nullifiers,
            output_revertible.nullifiers,
            hints.sorted_nullifier_hints
        );

        // l2_to_l1_msgs
        let tx_context = self.previous_kernel.constants.tx_context;
        validate_transformed_values(
            prev_data.l2_to_l1_msgs,
            hints.siloed_l2_to_l1_msgs,
            |msg| silo_l2_to_l1_message(msg, tx_context.version, tx_context.chain_id)
        );

        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.l2_to_l1_msgs,
            hints.siloed_l2_to_l1_msgs,
            split_counter,
            output_non_revertible.l2_to_l1_msgs,
            output_revertible.l2_to_l1_msgs,
            hints.sorted_l2_to_l1_msg_hints
        );

        // note_encrypted_logs_hashes
        validate_value_transformation(
            prev_data.note_encrypted_logs_hashes,
            hints.note_encrypted_logs_hashes,
            |nlh: NoteLogHash, lh: LogHash| (lh.value == nlh.value) & (lh.length == nlh.length) & (lh.counter == 0)
        );

        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.note_encrypted_logs_hashes,
            hints.note_encrypted_logs_hashes,
            split_counter,
            output_non_revertible.note_encrypted_logs_hashes,
            output_revertible.note_encrypted_logs_hashes,
            hints.sorted_note_encrypted_log_hash_hints
        );

        // encrypted_logs_hashes
        validate_value_transformation(
            prev_data.encrypted_logs_hashes,
            hints.siloed_encrypted_logs_hashes,
            |slh: ScopedEncryptedLogHash, lh: LogHash| (lh.value == silo_encrypted_log_hash(slh)) & (lh.length == slh.log_hash.length) & (lh.counter == 0)
        );

        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.encrypted_logs_hashes,
            hints.siloed_encrypted_logs_hashes,
            split_counter,
            output_non_revertible.encrypted_logs_hashes,
            output_revertible.encrypted_logs_hashes,
            hints.sorted_encrypted_log_hash_hints
        );

        // unencrypted_logs_hashes
        validate_value_transformation(
            prev_data.unencrypted_logs_hashes,
            hints.siloed_unencrypted_logs_hashes,
            |slh: ScopedLogHash, lh: LogHash| (lh.value == silo_unencrypted_log_hash(slh)) & (lh.length == slh.log_hash.length) & (lh.counter == 0)
        );

        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.unencrypted_logs_hashes,
            hints.siloed_unencrypted_logs_hashes,
            split_counter,
            output_non_revertible.unencrypted_logs_hashes,
            output_revertible.unencrypted_logs_hashes,
            hints.sorted_unencrypted_log_hash_hints
        );

        // public_call_stack
        validate_value_transformation(
            prev_data.public_call_stack,
            hints.public_call_requests,
            |cr: CallRequest, public_cr: CallRequest| (public_cr.hash == cr.hash)
                & (public_cr.caller_contract_address == cr.caller_contract_address)
                & (public_cr.caller_context == cr.caller_context)
                // TODO: Hide the counter of a public call request.
                // & (public_cr.start_side_effect_counter == 0)
                & (public_cr.end_side_effect_counter == 0)
        );

        assert_split_sorted_transformed_value_arrays_desc(
            prev_data.public_call_stack,
            hints.public_call_requests,
            split_counter,
            output_non_revertible.public_call_stack,
            output_revertible.public_call_stack,
            hints.sorted_public_call_request_hints
        )
    }

    fn validate_gas_limits(self) {
        let limits = self.previous_kernel.constants.tx_context.gas_settings.gas_limits;
        let total_gas_used = self.output.end_non_revertible.gas_used + self.output.end.gas_used;
        assert(total_gas_used.within(limits), "The gas used exceeds the gas limits");
    }
}
