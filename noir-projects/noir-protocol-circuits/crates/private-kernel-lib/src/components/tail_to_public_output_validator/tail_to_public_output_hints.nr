use dep::types::{
    abis::{kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs},
    constants::{MAX_L2_TO_L1_MSGS_PER_TX, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX},
    utils::arrays::{sort_get_split_order_hints_asc, sort_get_split_order_hints_desc, SplitOrderHints}
};

struct TailToPublicOutputHints {
    // L2 to l1 msgs.
    sorted_l2_to_l1_msg_hints: SplitOrderHints<MAX_L2_TO_L1_MSGS_PER_TX>,
    // Unencrypted log hashes.
    sorted_unencrypted_log_hash_hints: SplitOrderHints<MAX_UNENCRYPTED_LOGS_PER_TX>,
    // Public call requests.
    sorted_public_call_request_hints: SplitOrderHints<MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
}

unconstrained pub fn generate_tail_to_public_output_hints(previous_kernel: PrivateKernelCircuitPublicInputs) -> TailToPublicOutputHints {
    let split_counter = previous_kernel.min_revertible_side_effect_counter;

    // l2_to_l1_msgss
    let sorted_l2_to_l1_msg_hints = sort_get_split_order_hints_asc(previous_kernel.end.l2_to_l1_msgs, split_counter);

    // unencrypted_logs
    let sorted_unencrypted_log_hash_hints = sort_get_split_order_hints_asc(previous_kernel.end.unencrypted_logs_hashes, split_counter);

    // public_call_requests
    let sorted_public_call_request_hints = sort_get_split_order_hints_desc(previous_kernel.end.public_call_requests, split_counter);

    TailToPublicOutputHints { sorted_l2_to_l1_msg_hints, sorted_unencrypted_log_hash_hints, sorted_public_call_request_hints }
}
