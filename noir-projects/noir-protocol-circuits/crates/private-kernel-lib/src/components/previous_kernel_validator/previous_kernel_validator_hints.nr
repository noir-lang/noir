use dep::types::{
    abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    constants::{MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX}, hash::silo_note_hash,
    utils::arrays::{find_index, sort_get_order_hints_asc}
};

struct PreviousKernelValidatorHints {
    siloed_note_hashes: [Field; MAX_NOTE_HASHES_PER_TX],
    note_hash_indexes_for_nullifiers: [u32; MAX_NULLIFIERS_PER_TX],
}

unconstrained pub fn generate_previous_kernel_validator_hints(previous_kernel: PrivateKernelCircuitPublicInputs) -> PreviousKernelValidatorHints {
    let mut siloed_note_hashes = [0; MAX_NOTE_HASHES_PER_TX];
    let unsilod_note_hashes = previous_kernel.end.note_hashes;
    let sorted_note_hash_hints = sort_get_order_hints_asc(unsilod_note_hashes);
    let tx_hash = previous_kernel.end.nullifiers[0].value(); // First nullifier is tx hash.
    for i in 0..unsilod_note_hashes.len() {
        siloed_note_hashes[i] = silo_note_hash(unsilod_note_hashes[i], tx_hash, sorted_note_hash_hints[i].sorted_index);
    }

    let mut note_hash_indexes_for_nullifiers = [0; MAX_NULLIFIERS_PER_TX];
    let nullifiers = previous_kernel.end.nullifiers;
    for i in 0..nullifiers.len() {
        let nullified_note_hash = nullifiers[i].nullifier.note_hash;
        let note_hash_index = find_index(siloed_note_hashes, |n: Field| n == nullified_note_hash);
        if (nullified_note_hash != 0) & (note_hash_index != MAX_NOTE_HASHES_PER_TX) {
            note_hash_indexes_for_nullifiers[i] = note_hash_index;
        }
    }

    PreviousKernelValidatorHints { siloed_note_hashes, note_hash_indexes_for_nullifiers }
}
