use crate::components::reset_output_composer::ResetOutputHints;
use dep::reset_kernel_lib::{
    PrivateValidationRequestProcessor, TransientDataIndexHint, verify_squashed_transient_data,
};
use dep::types::{
    abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
        note_hash::ScopedNoteHash,
        nullifier::ScopedNullifier,
        private_log::{PrivateLog, PrivateLogData},
        side_effect::{Ordered, OrderedValue, scoped::Scoped},
    },
    hash::{
        compute_siloed_private_log_field, compute_unique_siloed_note_hash, silo_note_hash,
        silo_nullifier,
    },
    traits::is_empty,
    utils::arrays::assert_sorted_transformed_value_array_capped_size,
};

// Verifies that the first field of a private log is siloed with the contract address.
// Apps can define the meaning of the first field: it could be a tag, part of a tag, or a random value embedded in the
// content, etc.
// This siloing ensures that the log is verifiably linked to the specified contract address.
// Without this check, an app could impersonate another by emitting a log identical to one from a different contract
// or copying a tag from another contract while using entirely different content, potentially misleading the recipient.
fn is_valid_siloed_private_log(out: PrivateLog, prev: Scoped<PrivateLogData>) -> bool {
    let expected_first_field = if prev.contract_address.is_zero() {
        prev.inner.log.fields[0]
    } else {
        compute_siloed_private_log_field(prev.contract_address, prev.inner.log.fields[0])
    };
    let mut is_valid = out.fields[0] == expected_first_field;
    for i in 1..out.fields.len() {
        is_valid &= out.fields[i] == prev.inner.log.fields[i];
    }
    is_valid & (out.length == prev.inner.log.length)
}

pub struct ResetOutputValidator<let NH_RR_PENDING: u32, let NH_RR_SETTLED: u32, let NLL_RR_PENDING: u32, let NLL_RR_SETTLED: u32, let KEY_VALIDATION_REQUESTS: u32, let NUM_TRANSIENT_DATA_INDEX_HINTS: u32> {
    output: PrivateKernelCircuitPublicInputs,
    previous_kernel: PrivateKernelCircuitPublicInputs,
    validation_request_processor: PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
    transient_data_index_hints: [TransientDataIndexHint; NUM_TRANSIENT_DATA_INDEX_HINTS],
    note_hash_siloing_amount: u32,
    nullifier_siloing_amount: u32,
    private_log_siloing_amount: u32,
    hints: ResetOutputHints,
}

impl<let NH_RR_PENDING: u32, let NH_RR_SETTLED: u32, let NLL_RR_PENDING: u32, let NLL_RR_SETTLED: u32, let KEY_VALIDATION_REQUESTS: u32, let NUM_TRANSIENT_DATA_INDEX_HINTS: u32> ResetOutputValidator<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS, NUM_TRANSIENT_DATA_INDEX_HINTS> {
    pub fn new(
        output: PrivateKernelCircuitPublicInputs,
        previous_kernel: PrivateKernelCircuitPublicInputs,
        validation_request_processor: PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
        transient_data_index_hints: [TransientDataIndexHint; NUM_TRANSIENT_DATA_INDEX_HINTS],
        note_hash_siloing_amount: u32,
        nullifier_siloing_amount: u32,
        private_log_siloing_amount: u32,
        hints: ResetOutputHints,
    ) -> Self {
        ResetOutputValidator {
            output,
            previous_kernel,
            validation_request_processor,
            transient_data_index_hints,
            note_hash_siloing_amount,
            nullifier_siloing_amount,
            private_log_siloing_amount,
            hints,
        }
    }

    pub fn validate(self) {
        self.validate_unchanged_data();

        self.validation_request_processor.validate(self.output.validation_requests);

        self.validate_transient_data();

        self.validate_sorted_siloed_data();
    }

    fn validate_unchanged_data(self) {
        assert_eq(self.output.is_private_only, self.previous_kernel.is_private_only);
        assert_eq(
            self.output.claimed_first_nullifier,
            self.previous_kernel.claimed_first_nullifier,
        );
        assert_eq(self.output.constants, self.previous_kernel.constants);

        assert_eq(
            self.output.min_revertible_side_effect_counter,
            self.previous_kernel.min_revertible_side_effect_counter,
        );

        assert_eq(
            self.output.public_teardown_call_request,
            self.previous_kernel.public_teardown_call_request,
        );

        assert_eq(self.output.fee_payer, self.previous_kernel.fee_payer);

        // accumulated_data
        assert_eq(self.output.end.l2_to_l1_msgs, self.previous_kernel.end.l2_to_l1_msgs);
        assert_eq(
            self.output.end.contract_class_logs_hashes,
            self.previous_kernel.end.contract_class_logs_hashes,
        );
        assert_eq(
            self.output.end.public_call_requests,
            self.previous_kernel.end.public_call_requests,
        );
        assert_eq(self.output.end.private_call_stack, self.previous_kernel.end.private_call_stack);
    }

    fn validate_transient_data(self) {
        if NUM_TRANSIENT_DATA_INDEX_HINTS == 0 {
            assert_eq(
                self.hints.kept_note_hashes,
                self.previous_kernel.end.note_hashes,
                "mismatch kept note hashes",
            );
            assert_eq(
                self.hints.kept_nullifiers,
                self.previous_kernel.end.nullifiers,
                "mismatch kept nullifiers",
            );
            assert_eq(
                self.hints.kept_private_logs,
                self.previous_kernel.end.private_logs,
                "mismatch kept private logs",
            );
        } else {
            verify_squashed_transient_data(
                self.previous_kernel.end.note_hashes,
                self.previous_kernel.end.nullifiers,
                self.previous_kernel.end.private_logs,
                self.hints.kept_note_hashes,
                self.hints.kept_nullifiers,
                self.hints.kept_private_logs,
                self.transient_data_index_hints,
                self.hints.transient_or_propagated_note_hash_indexes_for_logs,
                self.output.validation_requests.split_counter.unwrap_unchecked(),
            );
        }
    }

    fn validate_sorted_siloed_data(self) {
        // note_hashes
        if self.note_hash_siloing_amount == 0 {
            assert_eq(
                self.output.end.note_hashes,
                self.hints.kept_note_hashes,
                "output note hashes mismatch",
            );
        } else {
            self.validate_sorted_siloed_note_hashes();
        }

        // nullifiers
        if self.nullifier_siloing_amount == 0 {
            assert_eq(
                self.output.end.nullifiers,
                self.hints.kept_nullifiers,
                "output nullifiers mismatch",
            );
        } else {
            self.validate_sorted_siloed_nullifiers();
        }

        // private_logs
        if self.private_log_siloing_amount == 0 {
            assert_eq(
                self.output.end.private_logs,
                self.hints.kept_private_logs,
                "output private logs mismatch",
            );
        } else {
            self.validate_sorted_siloed_private_logs();
        }
    }

    fn validate_sorted_siloed_note_hashes(self) {
        // Check that the values are not already siloed in a previous reset.
        // Note hashes need to be siloed all together because new note hashes added later might affect the ordering and result in wrong nonces.
        // We only need to check the first item, since we always start siloing from index 0.
        // The first item should either be empty or not siloed (contract_address != 0).
        let note_hash = self.previous_kernel.end.note_hashes[0];
        assert(
            is_empty(note_hash) | !note_hash.contract_address.is_zero(),
            "note hashes have been siloed in a previous reset",
        );

        // Check siloing.
        let kept_note_hashes = self.hints.kept_note_hashes;
        let output_note_hashes = self.output.end.note_hashes;
        let sorted_indexes = self.hints.sorted_note_hash_indexes;
        let is_private_only = self.output.is_private_only;
        let min_revertible_side_effect_counter = self.output.min_revertible_side_effect_counter;

        for i in 0..kept_note_hashes.len() {
            if i < self.note_hash_siloing_amount {
                let sorted_index = sorted_indexes[i];
                let output_note_hash = output_note_hashes[sorted_index];
                let note_hash = kept_note_hashes[i];
                let siloed_note_hash = silo_note_hash(note_hash);
                let siloed_unique_note_hash = compute_unique_siloed_note_hash(
                    siloed_note_hash,
                    self.output.claimed_first_nullifier,
                    sorted_index,
                );
                // We don't silo with nonce revertible note hashes, since we don't know their final position in the tx
                let expected_value = if is_private_only
                    | (note_hash.counter() < min_revertible_side_effect_counter) {
                    siloed_unique_note_hash
                } else {
                    siloed_note_hash
                };
                assert_eq(output_note_hash.value(), expected_value, "incorrect siloed note hashes");
            } else {
                // Don't have to check empty items here.
                // assert_sorted_transformed_value_array_capped_size ensures that there are the same amount of empty items padded in kept_note_hashes and in self.output.end.note_hashes.
            }
        }

        // Check ordering.
        assert_sorted_transformed_value_array_capped_size(
            kept_note_hashes,
            output_note_hashes,
            |prev: ScopedNoteHash, out: ScopedNoteHash| {
                out.contract_address.is_zero() & (out.counter() == prev.counter())
            },
            sorted_indexes,
            self.note_hash_siloing_amount,
        );
    }

    fn validate_sorted_siloed_nullifiers(self) {
        // Unlike note hashes, we don't have to check that the nullifiers haven't been siloed.
        // silo_nullifier() will return the already-siloed value if contract address is zero.
        assert_sorted_transformed_value_array_capped_size(
            self.hints.kept_nullifiers,
            self.output.end.nullifiers,
            |prev: ScopedNullifier, out: ScopedNullifier| {
                (out.value() == silo_nullifier(prev))
                    & (out.counter() == prev.counter())
                    & (out.nullifier.note_hash == prev.nullifier.note_hash)
                    & out.contract_address.is_zero()
            },
            self.hints.sorted_nullifier_indexes,
            self.nullifier_siloing_amount,
        );
    }

    fn validate_sorted_siloed_private_logs(self) {
        assert_sorted_transformed_value_array_capped_size(
            self.hints.kept_private_logs,
            self.output.end.private_logs,
            |prev: Scoped<PrivateLogData>, out: Scoped<PrivateLogData>| {
                is_valid_siloed_private_log(out.inner.log, prev)
                    & (out.inner.note_hash_counter == prev.inner.note_hash_counter)
                    & (out.inner.counter == prev.inner.counter)
                    & out.contract_address.is_zero()
            },
            self.hints.sorted_private_log_indexes,
            self.private_log_siloing_amount,
        );
    }
}
