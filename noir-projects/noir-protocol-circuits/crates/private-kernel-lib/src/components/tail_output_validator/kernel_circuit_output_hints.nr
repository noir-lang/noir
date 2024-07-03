use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    log_hash::{LogHash, NoteLogHash, ScopedLogHash, ScopedEncryptedLogHash}
},
    constants::{
    MAX_ENCRYPTED_LOGS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX, MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX,
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX
},
    hash::{
    silo_encrypted_log_hash, silo_l2_to_l1_message, silo_note_hash, silo_nullifier,
    silo_unencrypted_log_hash
},
    messaging::l2_to_l1_message::ScopedL2ToL1Message, traits::{Empty, is_empty},
    utils::arrays::{OrderHint, sort_by_counters_asc, sort_get_order_hints_asc}
};

struct Hints {
    // Note hashes.
    sorted_note_hash_hints: [OrderHint; MAX_NOTE_HASHES_PER_TX],
    siloed_note_hashes: [Field; MAX_NOTE_HASHES_PER_TX],
    // Nullifiers.
    sorted_nullifier_hints: [OrderHint; MAX_NULLIFIERS_PER_TX],
    siloed_nullifiers: [Field; MAX_NULLIFIERS_PER_TX],
    // L2 to l1 msgs.
    sorted_l2_to_l1_msg_hints: [OrderHint; MAX_L2_TO_L1_MSGS_PER_TX],
    siloed_l2_to_l1_msgs: [Field; MAX_L2_TO_L1_MSGS_PER_TX],
    // Note encrypted log hashes.
    note_encrypted_log_hashes: [LogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_log_hashes: [LogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_log_hash_hints: [OrderHint; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    // Encrypted log hashes.
    siloed_encrypted_log_hashes: [LogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_siloed_encrypted_log_hashes: [LogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hash_hints: [OrderHint; MAX_ENCRYPTED_LOGS_PER_TX],
    // Unencrypted log hashes.
    siloed_unencrypted_log_hashes: [LogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_siloed_unencrypted_log_hashes: [LogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hash_hints: [OrderHint; MAX_UNENCRYPTED_LOGS_PER_TX],
}

unconstrained pub fn generate_kernel_circuit_output_hints(previous_kernel: PrivateKernelCircuitPublicInputs) -> Hints {
    // note_hashes
    let sorted_note_hash_hints = sort_get_order_hints_asc(previous_kernel.end.note_hashes);

    let mut siloed_note_hashes = [0; MAX_NOTE_HASHES_PER_TX];

    // First nullifier is tx hash.
    let tx_hash = previous_kernel.end.nullifiers[0].value();
    let unsiloed_note_hashes = previous_kernel.end.note_hashes;
    for i in 0..unsiloed_note_hashes.len() {
        siloed_note_hashes[i] = silo_note_hash(unsiloed_note_hashes[i], tx_hash, i);
    }

    // nullifiers
    let sorted_nullifier_hints = sort_get_order_hints_asc(previous_kernel.end.nullifiers);
    let siloed_nullifiers = previous_kernel.end.nullifiers.map(silo_nullifier);

    // l2_to_l1_msgs
    let sorted_l2_to_l1_msg_hints = sort_get_order_hints_asc(previous_kernel.end.l2_to_l1_msgs);

    let tx_context = previous_kernel.constants.tx_context;
    let siloed_l2_to_l1_msgs = previous_kernel.end.l2_to_l1_msgs.map(
        |m: ScopedL2ToL1Message| silo_l2_to_l1_message(
            m,
            tx_context.version,
            tx_context.chain_id,
        )
    );

    // note_encrypted_logs
    let note_encrypted_log_hashes = previous_kernel.end.note_encrypted_logs_hashes.map(|h: NoteLogHash| h.expose_to_public());
    let sorted_note_encrypted_log_hashes = sort_by_counters_asc(previous_kernel.end.note_encrypted_logs_hashes).map(|h: NoteLogHash| h.expose_to_public());
    let sorted_note_encrypted_log_hash_hints = sort_get_order_hints_asc(previous_kernel.end.note_encrypted_logs_hashes);

    // encrypted_logs
    let mut siloed_log_hashes = previous_kernel.end.encrypted_logs_hashes;
    for i in 0..siloed_log_hashes.len() {
        siloed_log_hashes[i].log_hash.value = silo_encrypted_log_hash(previous_kernel.end.encrypted_logs_hashes[i]);
    }
    let sorted_siloed_encrypted_log_hashes = sort_by_counters_asc(siloed_log_hashes).map(|h: ScopedEncryptedLogHash| h.expose_to_public());
    let siloed_encrypted_log_hashes = siloed_log_hashes.map(|h: ScopedEncryptedLogHash| h.expose_to_public());
    let sorted_encrypted_log_hash_hints = sort_get_order_hints_asc(previous_kernel.end.encrypted_logs_hashes);

    // unencrypted_logs
    let mut siloed_log_hashes = previous_kernel.end.unencrypted_logs_hashes;
    for i in 0..siloed_log_hashes.len() {
        siloed_log_hashes[i].log_hash.value = silo_unencrypted_log_hash(previous_kernel.end.unencrypted_logs_hashes[i]);
    }
    let sorted_siloed_unencrypted_log_hashes = sort_by_counters_asc(siloed_log_hashes).map(|h: ScopedLogHash| h.inner());
    let siloed_unencrypted_log_hashes = siloed_log_hashes.map(|h: ScopedLogHash| h.inner());
    let sorted_unencrypted_log_hash_hints = sort_get_order_hints_asc(previous_kernel.end.unencrypted_logs_hashes);

    Hints {
        sorted_note_hash_hints,
        siloed_note_hashes,
        sorted_nullifier_hints,
        siloed_nullifiers,
        sorted_l2_to_l1_msg_hints,
        siloed_l2_to_l1_msgs,
        note_encrypted_log_hashes,
        sorted_note_encrypted_log_hashes,
        sorted_siloed_encrypted_log_hashes,
        sorted_note_encrypted_log_hash_hints,
        siloed_encrypted_log_hashes,
        sorted_encrypted_log_hash_hints,
        siloed_unencrypted_log_hashes,
        sorted_siloed_unencrypted_log_hashes,
        sorted_unencrypted_log_hash_hints
    }
}
