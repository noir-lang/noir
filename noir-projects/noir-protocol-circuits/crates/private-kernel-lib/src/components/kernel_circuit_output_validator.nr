use crate::components::{kernel_circuit_output_hints::{Hints, OrderHint}};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PrivateKernelCircuitPublicInputs},
    log_hash::{LogHash, NoteLogHash, ScopedEncryptedLogHash, ScopedLogHash}
},
    constants::MAX_NEW_NOTE_HASHES_PER_TX,
    hash::{
    compute_tx_logs_hash, compute_tx_note_logs_hash, silo_encrypted_log_hash, silo_l2_to_l1_message,
    silo_note_hash, silo_nullifier, silo_unencrypted_log_hash
},
    traits::{Empty, is_empty}, utils::arrays::assert_sorted_transformed_value_array
};

fn validate_transformed_value_array<T, S, N, Env>(
    original_array: [T; N],
    transformed_value_array: [S; N],
    is_transformed: fn[Env](T, S) -> bool
) {
    for i in 0..N {
        assert(is_transformed(original_array[i], transformed_value_array[i]), "invalid transformed value");
    }
}

fn validate_siloed_value_array<T, S, N, Env>(
    original_array: [T; N],
    transformed_value_array: [S; N],
    silo_value: fn[Env](T) -> S
) where S: Empty + Eq {
    validate_transformed_value_array(
        original_array,
        transformed_value_array,
        |original: T, transformed: S| transformed == silo_value(original)
    );
}

struct KernelCircuitOutputValidator {
    output: KernelCircuitPublicInputs,
    previous_kernel: PrivateKernelCircuitPublicInputs
}

impl KernelCircuitOutputValidator {
    pub fn new(
        output: KernelCircuitPublicInputs,
        previous_kernel: PrivateKernelCircuitPublicInputs
    ) -> Self {
        KernelCircuitOutputValidator { output, previous_kernel }
    }

    pub fn validate<N>(self, hints: Hints) {
        self.validate_empty_values();
        self.validate_propagated_values();
        self.validate_propagated_sorted_siloed_values(hints);
        self.validate_accumulated_values(hints);
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
    }

    fn validate_propagated_sorted_siloed_values(self, hints: Hints) {
        // new_note_hashes
        let first_nullifier = self.output.end.new_nullifiers[0];
        let unsiloed_note_hashes = self.previous_kernel.end.new_note_hashes;
        for i in 0..unsiloed_note_hashes.len() {
            let siloed_note_hash = silo_note_hash(unsiloed_note_hashes[i], first_nullifier, i);
            assert_eq(hints.siloed_note_hashes[i], siloed_note_hash, "mismatch siloed note hashes");
        }

        assert_sorted_transformed_value_array(
            self.previous_kernel.end.new_note_hashes,
            hints.siloed_note_hashes,
            self.output.end.new_note_hashes,
            hints.sorted_note_hash_hints
        );

        // new_nullifiers
        validate_siloed_value_array(
            self.previous_kernel.end.new_nullifiers,
            hints.siloed_nullifiers,
            silo_nullifier
        );

        assert_sorted_transformed_value_array(
            self.previous_kernel.end.new_nullifiers,
            hints.siloed_nullifiers,
            self.output.end.new_nullifiers,
            hints.sorted_nullifier_hints
        );

        // new_l2_to_l1_msgs
        let tx_context = self.previous_kernel.constants.tx_context;
        validate_siloed_value_array(
            self.previous_kernel.end.new_l2_to_l1_msgs,
            hints.siloed_l2_to_l1_msgs,
            |msg| silo_l2_to_l1_message(msg, tx_context.version, tx_context.chain_id)
        );

        assert_sorted_transformed_value_array(
            self.previous_kernel.end.new_l2_to_l1_msgs,
            hints.siloed_l2_to_l1_msgs,
            self.output.end.new_l2_to_l1_msgs,
            hints.sorted_l2_to_l1_msg_hints
        );
    }

    fn validate_accumulated_values(self, hints: Hints) {
        // note_encrypted_log_hashes
        validate_transformed_value_array(
            self.previous_kernel.end.note_encrypted_logs_hashes,
            hints.note_encrypted_log_hashes,
            |nlh: NoteLogHash, lh: LogHash| (nlh.value == lh.value) & (nlh.length == lh.length)
        );

        assert_sorted_transformed_value_array(
            self.previous_kernel.end.note_encrypted_logs_hashes,
            hints.note_encrypted_log_hashes,
            hints.sorted_note_encrypted_log_hashes,
            hints.sorted_note_encrypted_log_hash_hints
        );

        let hash = compute_tx_note_logs_hash(hints.sorted_note_encrypted_log_hashes);
        assert_eq(hash, self.output.end.note_encrypted_logs_hash, "mismatch note_encrypted_logs_hash");

        // encrypted_log_hashes
        validate_transformed_value_array(
            self.previous_kernel.end.encrypted_logs_hashes,
            hints.siloed_encrypted_log_hashes,
            |slh: ScopedEncryptedLogHash, lh: LogHash| (lh.value == silo_encrypted_log_hash(slh)) & (lh.length == slh.log_hash.length)
        );

        assert_sorted_transformed_value_array(
            self.previous_kernel.end.encrypted_logs_hashes,
            hints.siloed_encrypted_log_hashes,
            hints.sorted_siloed_encrypted_log_hashes,
            hints.sorted_encrypted_log_hash_hints
        );

        let hash = compute_tx_logs_hash(hints.sorted_siloed_encrypted_log_hashes);
        assert_eq(hash, self.output.end.encrypted_logs_hash, "mismatch encrypted_logs_hash");

        // unencrypted_log_hashes
        validate_transformed_value_array(
            self.previous_kernel.end.unencrypted_logs_hashes,
            hints.siloed_unencrypted_log_hashes,
            |slh: ScopedLogHash, lh: LogHash| (lh.value == silo_unencrypted_log_hash(slh)) & (lh.length == slh.log_hash.length)
        );

        assert_sorted_transformed_value_array(
            self.previous_kernel.end.unencrypted_logs_hashes,
            hints.siloed_unencrypted_log_hashes,
            hints.sorted_siloed_unencrypted_log_hashes,
            hints.sorted_unencrypted_log_hash_hints
        );

        let hash = compute_tx_logs_hash(hints.sorted_siloed_unencrypted_log_hashes);
        assert_eq(hash, self.output.end.unencrypted_logs_hash, "mismatch unencrypted_logs_hash");
    }
}
