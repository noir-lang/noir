use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    log_hash::{LogHash, NoteLogHash, ScopedLogHash, ScopedEncryptedLogHash}, note_hash::NoteHash,
    nullifier::Nullifier, public_call_request::PublicCallRequest
},
    constants::{
    MAX_ENCRYPTED_LOGS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX, MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX,
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX
},
    hash::{silo_encrypted_log_hash, silo_l2_to_l1_message, silo_note_hash, silo_nullifier},
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    utils::arrays::{sort_get_split_order_hints_asc, sort_get_split_order_hints_desc, SplitOrderHints}
};

struct TailToPublicOutputHints {
    // Note hashes.
    siloed_note_hashes: [NoteHash; MAX_NOTE_HASHES_PER_TX],
    sorted_note_hash_hints: SplitOrderHints<MAX_NOTE_HASHES_PER_TX>,
    // Nullifiers.
    siloed_nullifiers: [Nullifier; MAX_NULLIFIERS_PER_TX],
    sorted_nullifier_hints: SplitOrderHints<MAX_NULLIFIERS_PER_TX>,
    // L2 to l1 msgs.
    siloed_l2_to_l1_msgs: [Field; MAX_L2_TO_L1_MSGS_PER_TX],
    sorted_l2_to_l1_msg_hints: SplitOrderHints<MAX_L2_TO_L1_MSGS_PER_TX>,
    // Note encrypted log hashes.
    note_encrypted_logs_hashes: [LogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_log_hash_hints: SplitOrderHints<MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    // Encrypted log hashes.
    siloed_encrypted_logs_hashes: [LogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_log_hash_hints: SplitOrderHints<MAX_ENCRYPTED_LOGS_PER_TX>,
    // Unencrypted log hashes.
    sorted_unencrypted_log_hash_hints: SplitOrderHints<MAX_UNENCRYPTED_LOGS_PER_TX>,
    // Public call requests.
    public_call_requests: [PublicCallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    sorted_public_call_request_hints: SplitOrderHints<MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
}

unconstrained pub fn generate_tail_to_public_output_hints(previous_kernel: PrivateKernelCircuitPublicInputs) -> TailToPublicOutputHints {
    let split_counter = previous_kernel.min_revertible_side_effect_counter;

    // note_hashes
    let unsiloed_note_hashes = previous_kernel.end.note_hashes;

    let first_nullifier = previous_kernel.end.nullifiers[0].value();
    let mut siloed_note_hashes = [NoteHash::empty(); MAX_NOTE_HASHES_PER_TX];
    for i in 0..unsiloed_note_hashes.len() {
        siloed_note_hashes[i].value = silo_note_hash(unsiloed_note_hashes[i], first_nullifier, i);
    }

    let sorted_note_hash_hints = sort_get_split_order_hints_asc(unsiloed_note_hashes, split_counter);

    // nullifiers
    let unsiloed_nullifiers = previous_kernel.end.nullifiers;

    let mut siloed_nullifiers = [Nullifier::empty(); MAX_NULLIFIERS_PER_TX];
    for i in 0..unsiloed_nullifiers.len() {
        siloed_nullifiers[i].value = silo_nullifier(unsiloed_nullifiers[i]);
    }

    let sorted_nullifier_hints = sort_get_split_order_hints_asc(unsiloed_nullifiers, split_counter);

    // l2_to_l1_msgs
    let unsiloed_l2_to_l1_msgs = previous_kernel.end.l2_to_l1_msgs;

    let tx_context = previous_kernel.constants.tx_context;
    let siloed_l2_to_l1_msgs = unsiloed_l2_to_l1_msgs.map(
        |m: ScopedL2ToL1Message| silo_l2_to_l1_message(
            m,
            tx_context.version,
            tx_context.chain_id,
        )
    );

    let sorted_l2_to_l1_msg_hints = sort_get_split_order_hints_asc(unsiloed_l2_to_l1_msgs, split_counter);

    // note_encrypted_logs
    let note_encrypted_logs_hashes = previous_kernel.end.note_encrypted_logs_hashes.map(|h: NoteLogHash| h.expose_to_public());
    let sorted_note_encrypted_log_hash_hints = sort_get_split_order_hints_asc(previous_kernel.end.note_encrypted_logs_hashes, split_counter);

    // encrypted_logs
    let mut siloed_log_hashes = previous_kernel.end.encrypted_logs_hashes;
    for i in 0..siloed_log_hashes.len() {
        siloed_log_hashes[i].log_hash.value = silo_encrypted_log_hash(previous_kernel.end.encrypted_logs_hashes[i]);
    }
    let siloed_encrypted_logs_hashes = siloed_log_hashes.map(|h: ScopedEncryptedLogHash| h.expose_to_public());
    let sorted_encrypted_log_hash_hints = sort_get_split_order_hints_asc(previous_kernel.end.encrypted_logs_hashes, split_counter);

    // unencrypted_logs
    let sorted_unencrypted_log_hash_hints = sort_get_split_order_hints_asc(previous_kernel.end.unencrypted_logs_hashes, split_counter);

    // public_call_requests
    let public_call_requests = previous_kernel.end.public_call_requests.map(|cr: PublicCallRequest| cr.expose_to_public());
    let sorted_public_call_request_hints = sort_get_split_order_hints_desc(previous_kernel.end.public_call_requests, split_counter);

    TailToPublicOutputHints {
        siloed_note_hashes,
        sorted_note_hash_hints,
        sorted_nullifier_hints,
        siloed_nullifiers,
        sorted_l2_to_l1_msg_hints,
        siloed_l2_to_l1_msgs,
        note_encrypted_logs_hashes,
        sorted_note_encrypted_log_hash_hints,
        siloed_encrypted_logs_hashes,
        sorted_encrypted_log_hash_hints,
        sorted_unencrypted_log_hash_hints,
        public_call_requests,
        sorted_public_call_request_hints
    }
}
