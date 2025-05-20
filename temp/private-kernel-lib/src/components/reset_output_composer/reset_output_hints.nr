pub mod get_transient_or_propagated_note_hash_indexes_for_logs;
pub mod squash_transient_data;

use crate::components::reset_output_composer::reset_output_hints::{
    get_transient_or_propagated_note_hash_indexes_for_logs::get_transient_or_propagated_note_hash_indexes_for_logs,
    squash_transient_data::squash_transient_data,
};
use dep::reset_kernel_lib::TransientDataIndexHint;
use dep::types::{
    abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, note_hash::ScopedNoteHash,
        nullifier::ScopedNullifier, private_log::PrivateLogData, side_effect::scoped::Scoped,
    },
    constants::{MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_PRIVATE_LOGS_PER_TX},
    utils::arrays::{get_order_hints_asc, OrderHint},
};

pub struct ResetOutputHints {
    // note_hashes
    pub kept_note_hashes: [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX],
    pub sorted_note_hash_indexes: [u32; MAX_NOTE_HASHES_PER_TX],
    // nullifiers
    pub kept_nullifiers: [ScopedNullifier; MAX_NULLIFIERS_PER_TX],
    pub sorted_nullifier_indexes: [u32; MAX_NULLIFIERS_PER_TX],
    // private_logs
    pub kept_private_logs: [Scoped<PrivateLogData>; MAX_PRIVATE_LOGS_PER_TX],
    pub transient_or_propagated_note_hash_indexes_for_logs: [u32; MAX_PRIVATE_LOGS_PER_TX],
    pub sorted_private_log_indexes: [u32; MAX_PRIVATE_LOGS_PER_TX],
}

pub unconstrained fn generate_reset_output_hints<let NUM_TRANSIENT_DATA_INDEX_HINTS: u32>(
    previous_kernel: PrivateKernelCircuitPublicInputs,
    transient_data_index_hints: [TransientDataIndexHint; NUM_TRANSIENT_DATA_INDEX_HINTS],
) -> ResetOutputHints {
    let (kept_note_hashes, kept_nullifiers, kept_private_logs) = squash_transient_data(
        previous_kernel.end.note_hashes,
        previous_kernel.end.nullifiers,
        previous_kernel.end.private_logs,
        transient_data_index_hints,
    );

    // note_hashes
    let sorted_note_hash_indexes =
        get_order_hints_asc(kept_note_hashes).map(|h: OrderHint| h.sorted_index);

    // nullifiers
    let sorted_nullifier_indexes =
        get_order_hints_asc(kept_nullifiers).map(|h: OrderHint| h.sorted_index);

    // private_logs
    let transient_or_propagated_note_hash_indexes_for_logs = get_transient_or_propagated_note_hash_indexes_for_logs(
        previous_kernel.end.private_logs,
        previous_kernel.end.note_hashes,
        kept_note_hashes,
        transient_data_index_hints,
    );
    let sorted_private_log_indexes =
        get_order_hints_asc(kept_private_logs).map(|h: OrderHint| h.sorted_index);

    ResetOutputHints {
        kept_note_hashes,
        sorted_note_hash_indexes,
        kept_nullifiers,
        sorted_nullifier_indexes,
        kept_private_logs,
        transient_or_propagated_note_hash_indexes_for_logs,
        sorted_private_log_indexes,
    }
}
