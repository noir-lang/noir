use crate::tests::tail_output_validator_builder::TailOutputValidatorBuilder;
use dep::types::tests::utils::swap_items;

/**
 * l2_to_l1_msgs
 */
#[test]
fn validate_propagated_sorted_values_l2_to_l1_msgs_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    builder.output.append_exposed_l2_to_l1_msgs(2);

    builder.validate();
}

#[test]
fn validate_propagated_sorted_values_l2_to_l1_msgs_unordered_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    swap_items(&mut builder.previous_kernel.l2_to_l1_msgs, 0, 1);
    builder.output.append_exposed_l2_to_l1_msgs(2);

    builder.validate();
}

#[test(should_fail_with = "incorrect transformed value")]
fn validate_propagated_sorted_values_l2_to_l1_msgs_mismatch_hash_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    builder.output.append_exposed_l2_to_l1_msgs(2);
    // Tweak the content in the output.
    let mut msg = builder.output.l2_to_l1_msgs.get(0);
    msg.message.content += 1;
    builder.output.l2_to_l1_msgs.set(0, msg);

    builder.validate();
}

/**
 * contract_class_log_hashes
 */
#[test]
fn validate_propagated_sorted_values_contract_class_log_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.add_contract_class_log_hash(3, 2);
    builder.output.add_contract_class_log_hash(3, 2);

    builder.validate();
}

#[test(should_fail_with = "incorrect transformed value")]
fn validate_propagated_sorted_values_contract_class_log_hashes_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.add_contract_class_log_hash(3, 2);
    builder.output.add_contract_class_log_hash(3, 2);
    // Tweak the log hash in the output.
    let mut log_hash = builder.output.contract_class_logs_hashes.get(0);
    log_hash.inner.inner.value += 1;
    builder.output.contract_class_logs_hashes.set(0, log_hash);

    builder.validate();
}
