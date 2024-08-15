mod get_transient_or_propagated_note_hash_indexes_for_logs;
mod squash_transient_data;

use crate::components::reset_output_composer::reset_output_hints::{
    get_transient_or_propagated_note_hash_indexes_for_logs::get_transient_or_propagated_note_hash_indexes_for_logs,
    squash_transient_data::squash_transient_data
};
use dep::reset_kernel_lib::TransientDataIndexHint;
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, log_hash::NoteLogHash,
    note_hash::ScopedNoteHash, nullifier::ScopedNullifier
},
    constants::{
    MAX_ENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_NOTE_HASHES_PER_TX,
    MAX_NULLIFIERS_PER_TX
},
    utils::arrays::{OrderHint, sort_get_order_hints_asc}
};

struct ResetOutputHints {
    // note_hashes
    kept_note_hashes: [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX],
    sorted_note_hash_indexes: [u32; MAX_NOTE_HASHES_PER_TX],
    // nullifiers
    kept_nullifiers: [ScopedNullifier; MAX_NULLIFIERS_PER_TX],
    sorted_nullifier_indexes: [u32; MAX_NULLIFIERS_PER_TX],
    // note_encrypted_log_hashes
    kept_note_encrypted_log_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_log_hash_indexes: [u32; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    transient_or_propagated_note_hash_indexes_for_logs: [u32; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    // encrypted_log_hashes
    sorted_encrypted_log_hash_indexes: [u32; MAX_ENCRYPTED_LOGS_PER_TX],
}

unconstrained pub fn generate_reset_output_hints<let NUM_TRANSIENT_DATA_INDEX_HINTS: u32>(
    previous_kernel: PrivateKernelCircuitPublicInputs,
    transient_data_index_hints: [TransientDataIndexHint; NUM_TRANSIENT_DATA_INDEX_HINTS]
) -> ResetOutputHints {
    let (kept_note_hashes, kept_nullifiers, kept_note_encrypted_log_hashes) = squash_transient_data(
        previous_kernel.end.note_hashes,
        previous_kernel.end.nullifiers,
        previous_kernel.end.note_encrypted_logs_hashes,
        transient_data_index_hints
    );

    // note_hashes
    let sorted_note_hash_indexes = sort_get_order_hints_asc(kept_note_hashes).map(|h: OrderHint| h.sorted_index);

    // nullifiers
    let sorted_nullifier_indexes = sort_get_order_hints_asc(kept_nullifiers).map(|h: OrderHint| h.sorted_index);

    // note_encrypted_log_hashes
    let sorted_note_encrypted_log_hash_indexes = sort_get_order_hints_asc(kept_note_encrypted_log_hashes).map(|h: OrderHint| h.sorted_index);
    let transient_or_propagated_note_hash_indexes_for_logs = get_transient_or_propagated_note_hash_indexes_for_logs(
        previous_kernel.end.note_encrypted_logs_hashes,
        previous_kernel.end.note_hashes,
        kept_note_hashes,
        transient_data_index_hints
    );

    // encrypted_log_hashes
    let sorted_encrypted_log_hash_indexes = sort_get_order_hints_asc(previous_kernel.end.encrypted_logs_hashes).map(|h: OrderHint| h.sorted_index);

    ResetOutputHints {
        kept_note_hashes,
        sorted_note_hash_indexes,
        kept_nullifiers,
        sorted_nullifier_indexes,
        kept_note_encrypted_log_hashes,
        sorted_note_encrypted_log_hash_indexes,
        transient_or_propagated_note_hash_indexes_for_logs,
        sorted_encrypted_log_hash_indexes
    }
}
