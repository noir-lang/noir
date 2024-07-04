mod squash_transient_data;

use crate::components::reset_output_composer::squash_transient_data::squash_transient_data;
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, log_hash::NoteLogHash,
    note_hash::ScopedNoteHash, nullifier::ScopedNullifier
},
    constants::{MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX}
};

struct PrivateKernelResetOutputs {
    note_hashes: [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX],
    nullifiers: [ScopedNullifier; MAX_NULLIFIERS_PER_TX],
    note_encrypted_log_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
}

struct ResetOutputComposer {
    output: PrivateKernelResetOutputs,
}

impl ResetOutputComposer {
    pub fn new(
        previous_kernel: PrivateKernelCircuitPublicInputs,
        transient_nullifier_indexes_for_note_hashes: [u32; MAX_NOTE_HASHES_PER_TX],
        transient_note_hash_indexes_for_nullifiers: [u32; MAX_NULLIFIERS_PER_TX]
    ) -> Self {
        let (note_hashes, nullifiers, note_encrypted_log_hashes) = squash_transient_data(
            previous_kernel.end.note_hashes,
            previous_kernel.end.nullifiers,
            previous_kernel.end.note_encrypted_logs_hashes,
            transient_nullifier_indexes_for_note_hashes,
            transient_note_hash_indexes_for_nullifiers
        );
        let output = PrivateKernelResetOutputs { note_hashes, nullifiers, note_encrypted_log_hashes };
        ResetOutputComposer { output }
    }

    pub fn finish(self) -> PrivateKernelResetOutputs {
        self.output
    }
}
