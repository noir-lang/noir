mod reset_output_hints;

use crate::components::reset_output_composer::{reset_output_hints::{generate_reset_output_hints, ResetOutputHints}};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    log_hash::{NoteLogHash, ScopedEncryptedLogHash}, note_hash::ScopedNoteHash,
    nullifier::ScopedNullifier
},
    address::AztecAddress,
    constants::{
    MAX_ENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_NOTE_HASHES_PER_TX,
    MAX_NULLIFIERS_PER_TX
},
    hash::{mask_encrypted_log_hash, silo_note_hash, silo_nullifier}, utils::arrays::sort_by_counters_asc
};

struct PrivateKernelResetOutputs {
    note_hashes: [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX],
    nullifiers: [ScopedNullifier; MAX_NULLIFIERS_PER_TX],
    note_encrypted_log_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    encrypted_log_hashes: [ScopedEncryptedLogHash; MAX_ENCRYPTED_LOGS_PER_TX],
}

struct ResetOutputComposer {
    previous_kernel: PrivateKernelCircuitPublicInputs,
    note_hash_siloing_amount: u32,
    nullifier_siloing_amount: u32,
    encrypted_log_siloing_amount: u32,
    hints: ResetOutputHints,
}

impl ResetOutputComposer {
    pub fn new(
        previous_kernel: PrivateKernelCircuitPublicInputs,
        transient_nullifier_indexes_for_note_hashes: [u32; MAX_NOTE_HASHES_PER_TX],
        transient_note_hash_indexes_for_nullifiers: [u32; MAX_NULLIFIERS_PER_TX],
        note_hash_siloing_amount: u32,
        nullifier_siloing_amount: u32,
        encrypted_log_siloing_amount: u32
    ) -> Self {
        let hints = generate_reset_output_hints(
            previous_kernel,
            transient_nullifier_indexes_for_note_hashes,
            transient_note_hash_indexes_for_nullifiers
        );
        ResetOutputComposer { previous_kernel, note_hash_siloing_amount, nullifier_siloing_amount, encrypted_log_siloing_amount, hints }
    }

    pub fn finish(self) -> PrivateKernelResetOutputs {
        let note_hashes = if self.note_hash_siloing_amount == 0 {
            self.hints.kept_note_hashes
        } else {
            self.get_sorted_siloed_note_hashes()
        };

        let nullifiers = if self.nullifier_siloing_amount == 0 {
            self.hints.kept_nullifiers
        } else {
            self.get_sorted_siloed_nullifiers()
        };

        let note_encrypted_log_hashes = if self.note_hash_siloing_amount == 0 {
            self.hints.kept_note_encrypted_log_hashes
        } else {
            self.get_sorted_note_encrypted_log_hashes()
        };

        let encrypted_log_hashes = if self.encrypted_log_siloing_amount == 0 {
            self.previous_kernel.end.encrypted_logs_hashes
        } else {
            self.get_sorted_masked_encrypted_log_hashes()
        };

        PrivateKernelResetOutputs { note_hashes, nullifiers, note_encrypted_log_hashes, encrypted_log_hashes }
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
