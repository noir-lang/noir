use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PublicKernelCircuitPublicInputs},
    public_data_update_request::PublicDataUpdateRequest
},
    constants::{
    MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
    MAX_ENCRYPTED_LOGS_PER_TX
},
    hash::silo_note_hash, traits::Empty,
    utils::arrays::{
    array_merge, CombinedOrderHint, DedupedHints, get_combined_order_hints_asc, get_deduped_hints,
    sort_by_position_then_counter
}
};

struct SiloedNoteHashHint {
    siloed_note_hash: Field,
    index: u32,
}

impl Empty for SiloedNoteHashHint {
    fn empty() -> Self {
        SiloedNoteHashHint { siloed_note_hash: 0, index: 0 }
    }
}

impl Eq for SiloedNoteHashHint {
    fn eq(self, other: Self) -> bool {
        (self.siloed_note_hash == other.siloed_note_hash) & (self.index == other.index)
    }
}

struct OutputHints {
    siloed_note_hash_hints: [SiloedNoteHashHint; MAX_NOTE_HASHES_PER_TX],
    sorted_note_hash_hints: [CombinedOrderHint; MAX_NOTE_HASHES_PER_TX],
    sorted_nullifier_hints: [CombinedOrderHint; MAX_NULLIFIERS_PER_TX],
    sorted_l2_to_l1_msg_hints: [CombinedOrderHint; MAX_L2_TO_L1_MSGS_PER_TX],
    sorted_note_encrypted_log_hash_hints: [CombinedOrderHint; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hash_hints: [CombinedOrderHint; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hash_hints: [CombinedOrderHint; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    deduped_public_data_update_request_hints: DedupedHints<MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
}

unconstrained pub fn generate_output_hints(
    previous_kernel: PublicKernelCircuitPublicInputs,
    output: KernelCircuitPublicInputs
) -> OutputHints {
    let non_revertible = previous_kernel.end_non_revertible;
    let revertible = previous_kernel.end;

    let mut siloed_note_hash_hints = output.end.note_hashes.map(|siloed_note_hash| SiloedNoteHashHint { siloed_note_hash, index: 0 });
    for i in 0..siloed_note_hash_hints.len() {
        if siloed_note_hash_hints[i].siloed_note_hash != 0 {
            siloed_note_hash_hints[i].index = i;
        }
    }

    OutputHints {
        siloed_note_hash_hints,
        sorted_note_hash_hints: get_combined_order_hints_asc(non_revertible.note_hashes, revertible.note_hashes),
        sorted_nullifier_hints: get_combined_order_hints_asc(non_revertible.nullifiers, revertible.nullifiers),
        sorted_l2_to_l1_msg_hints: get_combined_order_hints_asc(non_revertible.l2_to_l1_msgs, revertible.l2_to_l1_msgs),
        sorted_note_encrypted_log_hash_hints: get_combined_order_hints_asc(
            non_revertible.note_encrypted_logs_hashes,
            revertible.note_encrypted_logs_hashes
        ),
        sorted_encrypted_log_hash_hints: get_combined_order_hints_asc(
            non_revertible.encrypted_logs_hashes,
            revertible.encrypted_logs_hashes
        ),
        sorted_unencrypted_log_hash_hints: get_combined_order_hints_asc(
            non_revertible.unencrypted_logs_hashes,
            revertible.unencrypted_logs_hashes
        ),
        sorted_public_data_update_requests: sort_by_position_then_counter(
            array_merge(
                non_revertible.public_data_update_requests,
                revertible.public_data_update_requests
            )
        ),
        deduped_public_data_update_request_hints: get_deduped_hints(
            non_revertible.public_data_update_requests,
            revertible.public_data_update_requests
        )
    }
}
