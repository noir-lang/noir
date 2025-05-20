use dep::types::{
    abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    constants::{
        MAX_CONTRACT_CLASS_LOGS_PER_TX, MAX_ENQUEUED_CALLS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX,
    },
    utils::arrays::{get_split_order_hints_asc, get_split_order_hints_desc, SplitOrderHints},
};

pub struct TailToPublicOutputHints {
    // L2 to l1 msgs.
    pub sorted_l2_to_l1_msg_hints: SplitOrderHints<MAX_L2_TO_L1_MSGS_PER_TX>,
    // Contract class log hashes.
    pub sorted_contract_class_log_hash_hints: SplitOrderHints<MAX_CONTRACT_CLASS_LOGS_PER_TX>,
    // Public call requests.
    pub sorted_public_call_request_hints: SplitOrderHints<MAX_ENQUEUED_CALLS_PER_TX>,
}

pub unconstrained fn generate_tail_to_public_output_hints(
    previous_kernel: PrivateKernelCircuitPublicInputs,
) -> TailToPublicOutputHints {
    let split_counter = previous_kernel.min_revertible_side_effect_counter;

    // l2_to_l1_msgss
    let sorted_l2_to_l1_msg_hints =
        get_split_order_hints_asc(previous_kernel.end.l2_to_l1_msgs, split_counter);

    // contract_class_logs
    let sorted_contract_class_log_hash_hints = get_split_order_hints_asc(
        previous_kernel.end.contract_class_logs_hashes,
        split_counter,
    );

    // public_call_requests
    let sorted_public_call_request_hints =
        get_split_order_hints_desc(previous_kernel.end.public_call_requests, split_counter);

    TailToPublicOutputHints {
        sorted_l2_to_l1_msg_hints,
        sorted_contract_class_log_hash_hints,
        sorted_public_call_request_hints,
    }
}
