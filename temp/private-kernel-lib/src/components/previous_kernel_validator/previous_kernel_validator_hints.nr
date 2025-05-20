use dep::types::{
    abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, note_hash::ScopedNoteHash,
        side_effect::OrderedValue,
    },
    constants::{MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX},
    utils::arrays::find_index_hint,
};

pub struct PreviousKernelValidatorHints {
    pub note_hash_indexes_for_nullifiers: [u32; MAX_NULLIFIERS_PER_TX],
}

pub unconstrained fn generate_previous_kernel_validator_hints(
    previous_kernel: PrivateKernelCircuitPublicInputs,
) -> PreviousKernelValidatorHints {
    let mut note_hash_indexes_for_nullifiers = [0; MAX_NULLIFIERS_PER_TX];
    let note_hashes = previous_kernel.end.note_hashes;
    let nullifiers = previous_kernel.end.nullifiers;
    for i in 0..nullifiers.len() {
        let nullified_note_hash = nullifiers[i].nullifier.note_hash;
        let note_hash_index = find_index_hint(
            note_hashes,
            |n: ScopedNoteHash| n.value() == nullified_note_hash,
        );
        if (nullified_note_hash != 0) & (note_hash_index != MAX_NOTE_HASHES_PER_TX) {
            note_hash_indexes_for_nullifiers[i] = note_hash_index;
        }
    }

    PreviousKernelValidatorHints { note_hash_indexes_for_nullifiers }
}
