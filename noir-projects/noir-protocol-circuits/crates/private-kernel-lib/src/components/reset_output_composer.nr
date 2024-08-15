mod reset_output_hints;

use crate::components::reset_output_composer::{reset_output_hints::{generate_reset_output_hints, ResetOutputHints}};
use dep::reset_kernel_lib::{
    KeyValidationHint, NoteHashReadRequestHints, NullifierReadRequestHints, TransientDataIndexHint,
    PrivateValidationRequestProcessor
};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    log_hash::{NoteLogHash, ScopedEncryptedLogHash}, note_hash::ScopedNoteHash,
    nullifier::ScopedNullifier, validation_requests::PrivateValidationRequests
},
    address::AztecAddress,
    constants::{
    MAX_ENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_NOTE_HASHES_PER_TX,
    MAX_NULLIFIERS_PER_TX
},
    hash::{mask_encrypted_log_hash, silo_note_hash, silo_nullifier}, utils::arrays::sort_by_counters_asc
};

struct ResetOutputComposer<
    let NH_RR_PENDING: u32,
    let NH_RR_SETTLED: u32,
    let NLL_RR_PENDING: u32,
    let NLL_RR_SETTLED: u32,
    let KEY_VALIDATION_REQUESTS: u32,
> {
    previous_kernel: PrivateKernelCircuitPublicInputs,
    validation_request_processor: PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
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
> ResetOutputComposer<
    NH_RR_PENDING,
    NH_RR_SETTLED,
    NLL_RR_PENDING,
    NLL_RR_SETTLED,
    KEY_VALIDATION_REQUESTS,
> {
    pub fn new<let TRANSIENT_DATA_AMOUNT: u32>(
        previous_kernel: PrivateKernelCircuitPublicInputs,
        validation_request_processor: PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS>,
        transient_data_index_hints: [TransientDataIndexHint; TRANSIENT_DATA_AMOUNT],
        note_hash_siloing_amount: u32,
        nullifier_siloing_amount: u32,
        encrypted_log_siloing_amount: u32
    ) -> Self {
        let hints = generate_reset_output_hints(previous_kernel, transient_data_index_hints);
        ResetOutputComposer {
            previous_kernel,
            validation_request_processor,
            note_hash_siloing_amount,
            nullifier_siloing_amount,
            encrypted_log_siloing_amount,
            hints
        }
    }

    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
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

        output.end.note_encrypted_logs_hashes = if self.note_hash_siloing_amount == 0 {
            self.hints.kept_note_encrypted_log_hashes
        } else {
            self.get_sorted_note_encrypted_log_hashes()
        };

        output.end.encrypted_logs_hashes = if self.encrypted_log_siloing_amount == 0 {
            self.previous_kernel.end.encrypted_logs_hashes
        } else {
            self.get_sorted_masked_encrypted_log_hashes()
        };

        output
    }

    fn get_sorted_siloed_note_hashes(self) -> [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX] {
        let mut note_hashes = sort_by_counters_asc(self.hints.kept_note_hashes);
        let first_nullifier = self.previous_kernel.end.nullifiers[0].value();
        for i in 0..note_hashes.len() {
            note_hashes[i].note_hash.value = silo_note_hash(
                note_hashes[i],
                first_nullifier,
                i
            );
            note_hashes[i].contract_address = AztecAddress::zero();
        }
        note_hashes
    }

    fn get_sorted_siloed_nullifiers(self) -> [ScopedNullifier; MAX_NULLIFIERS_PER_TX] {
        let mut nullifiers = sort_by_counters_asc(self.hints.kept_nullifiers);
        for i in 0..nullifiers.len() {
            nullifiers[i].nullifier.value = silo_nullifier(nullifiers[i]);
            nullifiers[i].contract_address = AztecAddress::zero();
        }
        nullifiers
    }

    fn get_sorted_note_encrypted_log_hashes(self) -> [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX] {
        let mut log_hashes = sort_by_counters_asc(self.hints.kept_note_encrypted_log_hashes);
        for i in 0..log_hashes.len() {
            log_hashes[i].note_hash_counter = 0;
        }
        log_hashes
    }

    fn get_sorted_masked_encrypted_log_hashes(self) -> [ScopedEncryptedLogHash; MAX_ENCRYPTED_LOGS_PER_TX] {
        let mut log_hashes = sort_by_counters_asc(self.previous_kernel.end.encrypted_logs_hashes);
        for i in 0..log_hashes.len() {
            log_hashes[i].contract_address = mask_encrypted_log_hash(log_hashes[i]);
            log_hashes[i].log_hash.randomness = 0;
        }
        log_hashes
    }
}
