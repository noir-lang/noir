use dep::types::{
    abis::{kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, log_hash::ScopedLogHash},
    constants::{MAX_L2_TO_L1_MSGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX},
    utils::arrays::{OrderHint, sort_by_counters_asc, sort_get_order_hints_asc}
};

struct TailOutputHints {
    // L2 to l1 msgs.
    sorted_l2_to_l1_msg_hints: [OrderHint; MAX_L2_TO_L1_MSGS_PER_TX],
    // Unencrypted log hashes.
    sorted_unencrypted_log_hashes: [ScopedLogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_log_hash_hints: [OrderHint; MAX_UNENCRYPTED_LOGS_PER_TX],
}

unconstrained pub fn generate_tail_output_hints(previous_kernel: PrivateKernelCircuitPublicInputs) -> TailOutputHints {
    // l2_to_l1_msgs
    let sorted_l2_to_l1_msg_hints = sort_get_order_hints_asc(previous_kernel.end.l2_to_l1_msgs);

    // unencrypted_logs
    let sorted_unencrypted_log_hashes = sort_by_counters_asc(previous_kernel.end.unencrypted_logs_hashes);
    let sorted_unencrypted_log_hash_hints = sort_get_order_hints_asc(previous_kernel.end.unencrypted_logs_hashes);

    TailOutputHints { sorted_l2_to_l1_msg_hints, sorted_unencrypted_log_hashes, sorted_unencrypted_log_hash_hints }
}
