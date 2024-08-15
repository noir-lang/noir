use crate::components::{reset_output_composer::ResetOutputHints};
use dep::reset_kernel_lib::{PrivateValidationRequestProcessor, TransientDataIndexHint, verify_squashed_transient_data};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    log_hash::{NoteLogHash, ScopedEncryptedLogHash}, note_hash::ScopedNoteHash,
    nullifier::ScopedNullifier
},
    hash::{mask_encrypted_log_hash, silo_note_hash, silo_nullifier}, traits::is_empty,
    utils::arrays::{assert_sorted_transformed_value_array, assert_sorted_transformed_value_array_capped_size}
};

struct ResetOutputValidator<
    let NH_RR_PENDING: u32,
    let NH_RR_SETTLED: u32,
    let NLL_RR_PENDING: u32,
    let NLL_RR_SETTLED: u32,
    let KEY_VALIDATION_REQUESTS: u32, 
    let NUM_TRANSIENT_DATA_INDEX_HINTS: u32
> {
    output: PrivateKernelCircuitPublicInputs,
    previous_kernel: PrivateKernelCircuitPublicInputs,
    validation_request_processor: PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
    transient_data_index_hints: [TransientDataIndexHint; NUM_TRANSIENT_DATA_INDEX_HINTS],
    note_hash_siloing_amount: u32,
    nullifier_siloing_amount: u32,
    encrypted_log_siloing_amount: u32,
    hints: ResetOutputHints,
}

impl<
    let NH_RR_PENDING: u32,
    let NH_RR_SETTLED: u32,
    let NLL_RR_PENDING: u32,
    let NLL_RR_SETTLED: u32,
    let KEY_VALIDATION_REQUESTS: u32, 
    let NUM_TRANSIENT_DATA_INDEX_HINTS: u32
> ResetOutputValidator<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS, NUM_TRANSIENT_DATA_INDEX_HINTS> {
    pub fn new(
        output: PrivateKernelCircuitPublicInputs,
        previous_kernel: PrivateKernelCircuitPublicInputs,
        validation_request_processor: PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
        transient_data_index_hints: [TransientDataIndexHint; NUM_TRANSIENT_DATA_INDEX_HINTS],
        note_hash_siloing_amount: u32,
        nullifier_siloing_amount: u32,
        encrypted_log_siloing_amount: u32,
        hints: ResetOutputHints
    ) -> Self {
        ResetOutputValidator {
            output,
            previous_kernel,
            validation_request_processor,
            transient_data_index_hints,
            note_hash_siloing_amount,
            nullifier_siloing_amount,
            encrypted_log_siloing_amount,
            hints
        }
    }

    pub fn validate(self) {
        self.validate_unchanged_data();

        self.validation_request_processor.validate(self.output.validation_requests);

        verify_squashed_transient_data(
            self.previous_kernel.end.note_hashes,
            self.previous_kernel.end.nullifiers,
            self.previous_kernel.end.note_encrypted_logs_hashes,
            self.hints.kept_note_hashes,
            self.hints.kept_nullifiers,
            self.hints.kept_note_encrypted_log_hashes,
            self.transient_data_index_hints,
            self.hints.transient_or_propagated_note_hash_indexes_for_logs,
            self.output.validation_requests.split_counter.unwrap_unchecked()
        );

        self.validate_sorted_siloed_data();
    }

    fn validate_unchanged_data(self) {
        assert_eq(self.output.constants, self.previous_kernel.constants);

        assert_eq(
            self.output.min_revertible_side_effect_counter, self.previous_kernel.min_revertible_side_effect_counter
        );

        assert_eq(self.output.public_teardown_call_request, self.previous_kernel.public_teardown_call_request);

        assert_eq(self.output.fee_payer, self.previous_kernel.fee_payer);

        // accumulated_data
        assert_eq(self.output.end.l2_to_l1_msgs, self.previous_kernel.end.l2_to_l1_msgs);
        assert_eq(self.output.end.unencrypted_logs_hashes, self.previous_kernel.end.unencrypted_logs_hashes);
        assert_eq(self.output.end.public_call_requests, self.previous_kernel.end.public_call_requests);
        assert_eq(self.output.end.private_call_stack, self.previous_kernel.end.private_call_stack);
    }

    fn validate_sorted_siloed_data(self) {
        // note_hashes
        // note_encrypted_logs_hashes
        if self.note_hash_siloing_amount == 0 {
            assert_eq(self.output.end.note_hashes, self.hints.kept_note_hashes, "output note hashes mismatch");
            assert_eq(
                self.output.end.note_encrypted_logs_hashes, self.hints.kept_note_encrypted_log_hashes, "output note logs mismatch"
            );
        } else {
            self.validate_sorted_siloed_note_hashes();
            self.validate_sorted_note_logs();
        }

        // nullifiers
        if self.nullifier_siloing_amount == 0 {
            assert_eq(self.output.end.nullifiers, self.hints.kept_nullifiers, "output nullifiers mismatch");
        } else {
            self.validate_sorted_siloed_nullifiers();
        }

        // encrypted_logs_hashes
        if self.encrypted_log_siloing_amount == 0 {
            assert_eq(
                self.output.end.encrypted_logs_hashes, self.previous_kernel.end.encrypted_logs_hashes, "output encrypted logs mismatch"
            );
        } else {
            self.validate_sorted_masked_encrypted_logs();
        }
    }

    fn validate_sorted_siloed_note_hashes(self) {
        // Check that the values are not already siloed in a previous reset.
        // Note hashes need to be siloed alltogether because new note hashes added later might affect the ordering and result in wrong nonces.
        // We only need to check the first item, since we always start siloing from index 0.
        // The first item should either be empty or not siloed (contract_address != 0).
        let note_hash = self.previous_kernel.end.note_hashes[0];
        assert(
            is_empty(note_hash) | !note_hash.contract_address.is_zero(), "note hashes have been siloed in a previous reset"
        );

        // Check siloing.
        let kept_note_hashes = self.hints.kept_note_hashes;
        let siloed_note_hashes = self.output.end.note_hashes;
        let sorted_indexes = self.hints.sorted_note_hash_indexes;
        let tx_hash = self.output.end.nullifiers[0].value(); // First nullifier is tx hash.
        for i in 0..kept_note_hashes.len() {
            if i < self.note_hash_siloing_amount {
                let note_hash = kept_note_hashes[i];
                let sorted_index = sorted_indexes[i];
                let siloed_note_hash = siloed_note_hashes[sorted_index];
                let siloed_value = silo_note_hash(note_hash, tx_hash, sorted_index);
                assert_eq(siloed_note_hash.value(), siloed_value, "incorrect siloed note hashes");
            } else {
                // Don't have to check empty items here.
                // assert_sorted_transformed_value_array_capped_size ensures that there are the same amount of empty items padded in kept_note_hashes and in self.output.end.note_hashes.
            }
        }

        // Check ordering.
        assert_sorted_transformed_value_array_capped_size(
            kept_note_hashes,
            siloed_note_hashes,
            |prev: ScopedNoteHash, out: ScopedNoteHash| out.contract_address.is_zero() & (out.counter() == prev.counter()),
            sorted_indexes,
            self.note_hash_siloing_amount
        );
    }

    fn validate_sorted_siloed_nullifiers(self) {
        // Unlike note hashes, we don't have to check that the nullifiers haven't been siloed.
        // silo_nullifier() will return the already-siloed value if contract address is zero.
        assert_sorted_transformed_value_array_capped_size(
            self.hints.kept_nullifiers,
            self.output.end.nullifiers,
            |prev: ScopedNullifier, out: ScopedNullifier|
                (out.value() == silo_nullifier(prev)) &
                (out.counter() == prev.counter()) &
                (out.nullifier.note_hash == prev.nullifier.note_hash) &
                out.contract_address.is_zero(),
            self.hints.sorted_nullifier_indexes,
            self.nullifier_siloing_amount
        );
    }

    fn validate_sorted_note_logs(self) {
        // This will be called together with validate_sorted_siloed_note_hashes().
        // Same as validate_sorted_siloed_note_hashes, it will only be run once. 
        // This is fine because we don't allow emitting logs for notes emitted in another function at the moment.
        // All the note logs emitted in a function call must link to note hashes emitted in the same call. 
        // This is checked in PrivateCallDataValidator > validate_note_logs.

        // note_hash_counter was used when squashing the note log along with its corresponding note hash.
        // It won't be used later on, so we can set it to 0 here.
        // It serves as a clue for the tail circuit to check that all the note logs are sorted in a reset circuit.

        // This is not capped because we don't know how many logs there are. There can be any number of logs for each note hash.
        // Consider adding a constant for it only when this becomes too costly.
        assert_sorted_transformed_value_array(
            self.hints.kept_note_encrypted_log_hashes,
            self.output.end.note_encrypted_logs_hashes,
            |prev: NoteLogHash, out: NoteLogHash|
                (out.value == prev.value) &
                (out.length == prev.length) &
                (out.counter == prev.counter) &
                (out.note_hash_counter == 0),
            self.hints.sorted_note_encrypted_log_hash_indexes
        );
    }

    fn validate_sorted_masked_encrypted_logs(self) {
        // Don't need to check that the logs are already masked.
        // If run repeatedly, it will return the masked contract address when randomness becomes 0.
        assert_sorted_transformed_value_array_capped_size(
            self.previous_kernel.end.encrypted_logs_hashes,
            self.output.end.encrypted_logs_hashes,
            |prev: ScopedEncryptedLogHash, out: ScopedEncryptedLogHash|
                (out.contract_address == mask_encrypted_log_hash(prev)) &
                (out.log_hash.value == prev.log_hash.value) &
                (out.log_hash.length == prev.log_hash.length) &
                (out.log_hash.counter == prev.log_hash.counter) &
                (out.log_hash.randomness == 0),
            self.hints.sorted_encrypted_log_hash_indexes,
            self.encrypted_log_siloing_amount
        );
    }
}
