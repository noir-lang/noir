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

#[test(should_fail_with="incorrect transformed value")]
fn validate_propagated_sorted_values_l2_to_l1_msgs_mismatch_hash_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    builder.output.append_exposed_l2_to_l1_msgs(2);
    // Tweak the content in the output.
    builder.output.l2_to_l1_msgs.storage[0].message.content += 1;

    builder.validate();
}

/**
 * unencrypted_log_hashes
 */

#[test]
fn validate_propagated_sorted_values_unencrypted_log_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_unencrypted_log_hashes(3);
    builder.output.append_unencrypted_log_hashes(3);
    builder.output.hash_unencrypted_log_hashes();

    builder.validate();
}

#[test]
fn validate_propagated_sorted_values_unencrypted_log_hashes_unordered_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_unencrypted_log_hashes(3);
    // Swap the items at index 0 and 2.
    swap_items(&mut builder.previous_kernel.unencrypted_logs_hashes, 0, 2);
    builder.output.append_unencrypted_log_hashes(3);
    builder.output.hash_unencrypted_log_hashes();

    builder.validate();
}

#[test(should_fail_with="incorrect transformed value")]
fn validate_propagated_sorted_values_unencrypted_log_hashes_wrong_order_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_unencrypted_log_hashes(3);
    builder.output.append_unencrypted_log_hashes(3);
    // Swap the items in the output, making them out of order
    swap_items(&mut builder.output.unencrypted_logs_hashes, 0, 2);

    builder.validate();
}
