pub mod reset_output_hints;

pub use reset_output_hints::ResetOutputHints;

use crate::components::reset_output_composer::reset_output_hints::generate_reset_output_hints;
use dep::reset_kernel_lib::{PrivateValidationRequestProcessor, TransientDataIndexHint};
use dep::types::{
    abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
        note_hash::ScopedNoteHash,
        nullifier::ScopedNullifier,
        private_log::PrivateLogData,
        side_effect::{Ordered, scoped::Scoped},
    },
    address::AztecAddress,
    constants::{MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_PRIVATE_LOGS_PER_TX},
    hash::{compute_unique_siloed_note_hash, silo_note_hash, silo_nullifier, silo_private_log},
    utils::arrays::sort_by_counter_asc,
};

pub struct ResetOutputComposer<let NH_RR_PENDING: u32, let NH_RR_SETTLED: u32, let NLL_RR_PENDING: u32, let NLL_RR_SETTLED: u32, let KEY_VALIDATION_REQUESTS: u32> {
    pub previous_kernel: PrivateKernelCircuitPublicInputs,
    pub validation_request_processor: PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
    pub note_hash_siloing_amount: u32,
    pub nullifier_siloing_amount: u32,
    pub private_log_siloing_amount: u32,
    pub hints: ResetOutputHints,
}

impl<let NH_RR_PENDING: u32, let NH_RR_SETTLED: u32, let NLL_RR_PENDING: u32, let NLL_RR_SETTLED: u32, let KEY_VALIDATION_REQUESTS: u32> ResetOutputComposer<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> {
    pub unconstrained fn new<let TRANSIENT_DATA_AMOUNT: u32>(
        previous_kernel: PrivateKernelCircuitPublicInputs,
        validation_request_processor: PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
        transient_data_index_hints: [TransientDataIndexHint; TRANSIENT_DATA_AMOUNT],
        note_hash_siloing_amount: u32,
        nullifier_siloing_amount: u32,
        private_log_siloing_amount: u32,
    ) -> Self {
        let hints = generate_reset_output_hints(previous_kernel, transient_data_index_hints);
        ResetOutputComposer {
            previous_kernel,
            validation_request_processor,
            note_hash_siloing_amount,
            nullifier_siloing_amount,
            private_log_siloing_amount,
            hints,
        }
    }

    pub unconstrained fn finish(self) -> PrivateKernelCircuitPublicInputs {
        let mut output = self.previous_kernel;

        output.validation_requests = self.validation_request_processor.compose();

        output.end.note_hashes = if self.note_hash_siloing_amount == 0 {
            self.hints.kept_note_hashes
        } else {
            self.get_sorted_siloed_note_hashes()
        };

        output.end.nullifiers = if self.nullifier_siloing_amount == 0 {
            self.hints.kept_nullifiers
        } else {
            self.get_sorted_siloed_nullifiers()
        };

        output.end.private_logs = if self.private_log_siloing_amount == 0 {
            self.hints.kept_private_logs
        } else {
            self.get_sorted_siloed_private_logs()
        };

        output
    }

    unconstrained fn get_sorted_siloed_note_hashes(
        self,
    ) -> [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX] {
        let is_private_only = self.previous_kernel.is_private_only;
        let min_revertible_side_effect_counter =
            self.previous_kernel.min_revertible_side_effect_counter;
        let mut note_hashes = sort_by_counter_asc(self.hints.kept_note_hashes);
        for i in 0..note_hashes.len() {
            let note_hash = note_hashes[i];
            let siloed_note_hash = silo_note_hash(note_hash);
            let unique_note_hash = compute_unique_siloed_note_hash(
                siloed_note_hash,
                self.previous_kernel.claimed_first_nullifier,
                i,
            );
            // We don't silo with nonce revertible note hashes, since we don't know their final position in the tx
            note_hashes[i].note_hash.value = if is_private_only
                | (note_hash.counter() < min_revertible_side_effect_counter) {
                unique_note_hash
            } else {
                siloed_note_hash
            };
            note_hashes[i].contract_address = AztecAddress::zero();
        }
        note_hashes
    }

    unconstrained fn get_sorted_siloed_nullifiers(
        self,
    ) -> [ScopedNullifier; MAX_NULLIFIERS_PER_TX] {
        let mut nullifiers = sort_by_counter_asc(self.hints.kept_nullifiers);
        for i in 0..nullifiers.len() {
            nullifiers[i].nullifier.value = silo_nullifier(nullifiers[i]);
            nullifiers[i].contract_address = AztecAddress::zero();
        }
        nullifiers
    }

    unconstrained fn get_sorted_siloed_private_logs(
        self,
    ) -> [Scoped<PrivateLogData>; MAX_PRIVATE_LOGS_PER_TX] {
        let private_logs = sort_by_counter_asc(self.hints.kept_private_logs);
        private_logs.map(|l: Scoped<PrivateLogData>| {
            PrivateLogData {
                log: silo_private_log(l),
                note_hash_counter: l.inner.note_hash_counter,
                counter: l.inner.counter,
            }
                .scope(AztecAddress::zero())
        })
    }
}
