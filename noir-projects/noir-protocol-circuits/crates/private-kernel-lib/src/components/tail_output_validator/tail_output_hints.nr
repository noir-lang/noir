use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    log_hash::{LogHash, NoteLogHash, ScopedLogHash, ScopedEncryptedLogHash}
},
    constants::{
    MAX_ENCRYPTED_LOGS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX, MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX,
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX
},
    hash::{silo_note_hash, silo_nullifier, mask_encrypted_log_hash},
    messaging::l2_to_l1_message::ScopedL2ToL1Message, traits::{Empty, is_empty},
    utils::arrays::{OrderHint, sort_by_counters_asc, sort_get_order_hints_asc}
};

struct TailOutputHints {
    // Note hashes.
    sorted_note_hash_hints: [OrderHint; MAX_NOTE_HASHES_PER_TX],
    // Nullifiers.
    sorted_nullifier_hints: [OrderHint; MAX_NULLIFIERS_PER_TX],
    siloed_nullifiers: [Field; MAX_NULLIFIERS_PER_TX],
    // L2 to l1 msgs.
    sorted_l2_to_l1_msg_hints: [OrderHint; MAX_L2_TO_L1_MSGS_PER_TX],
    // Note encrypted log hashes.
    sorted_note_encrypted_log_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_log_hash_hints: [OrderHint; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    // Encrypted log hashes.
    masked_encrypted_log_hashes: [ScopedLogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_masked_encrypted_log_hash_hints: [OrderHint; MAX_ENCRYPTED_LOGS_PER_TX],
    // Unencrypted log hashes.
    sorted_unencrypted_log_hashes: [ScopedLogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hash_hints: [OrderHint; MAX_UNENCRYPTED_LOGS_PER_TX],
}

unconstrained pub fn generate_tail_output_hints(previous_kernel: PrivateKernelCircuitPublicInputs) -> TailOutputHints {
    // note_hashes
    let sorted_note_hash_hints = sort_get_order_hints_asc(previous_kernel.end.note_hashes);

    // nullifiers
    let sorted_nullifier_hints = sort_get_order_hints_asc(previous_kernel.end.nullifiers);
    let siloed_nullifiers = previous_kernel.end.nullifiers.map(silo_nullifier);

    // l2_to_l1_msgs
    let sorted_l2_to_l1_msg_hints = sort_get_order_hints_asc(previous_kernel.end.l2_to_l1_msgs);

    // note_encrypted_logs
    let sorted_note_encrypted_log_hashes = sort_by_counters_asc(previous_kernel.end.note_encrypted_logs_hashes);
    let sorted_note_encrypted_log_hash_hints = sort_get_order_hints_asc(previous_kernel.end.note_encrypted_logs_hashes);

    // encrypted_logs
    let mut masked_log_hashes = previous_kernel.end.encrypted_logs_hashes;
    for i in 0..masked_log_hashes.len() {
        masked_log_hashes[i].contract_address = mask_encrypted_log_hash(previous_kernel.end.encrypted_logs_hashes[i]);
    }
    let masked_encrypted_log_hashes = masked_log_hashes.map(|h: ScopedEncryptedLogHash| h.expose_to_public());
    let sorted_masked_encrypted_log_hash_hints = sort_get_order_hints_asc(previous_kernel.end.encrypted_logs_hashes);

    // unencrypted_logs
    let sorted_unencrypted_log_hashes = sort_by_counters_asc(previous_kernel.end.unencrypted_logs_hashes);
    let sorted_unencrypted_log_hash_hints = sort_get_order_hints_asc(previous_kernel.end.unencrypted_logs_hashes);

    TailOutputHints {
        sorted_note_hash_hints,
        sorted_nullifier_hints,
        siloed_nullifiers,
        sorted_l2_to_l1_msg_hints,
        sorted_note_encrypted_log_hashes,
        sorted_note_encrypted_log_hash_hints,
        masked_encrypted_log_hashes,
        sorted_masked_encrypted_log_hash_hints,
        sorted_unencrypted_log_hashes,
        sorted_unencrypted_log_hash_hints
    }
}
