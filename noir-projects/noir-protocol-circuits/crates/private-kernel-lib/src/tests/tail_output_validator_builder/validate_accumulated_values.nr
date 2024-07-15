use crate::tests::tail_output_validator_builder::TailOutputValidatorBuilder;
use dep::types::tests::utils::swap_items;

// TODO: Add tests that fail to validate with tweaked hints.

/**
 * note_encrypted_log_hashes
 */

#[test]
fn validate_accumulated_values_note_encrypted_log_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_encrypted_log_hashes(3);
    builder.output.append_note_encrypted_log_hashes(3);
    builder.output.hash_note_encrypted_log_hashes();

    builder.validate();
}

#[test]
fn validate_accumulated_values_note_encrypted_log_hashes_unordered_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_encrypted_log_hashes(3);
    // Swap the items at index 0 and 2.
    swap_items(&mut builder.previous_kernel.note_encrypted_logs_hashes, 0, 2);
    builder.output.append_note_encrypted_log_hashes(3);
    builder.output.hash_note_encrypted_log_hashes();

    builder.validate();
}

#[test(should_fail_with="mismatch note_encrypted_logs_hash")]
fn validate_accumulated_values_note_encrypted_log_hashes_wrong_hash_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_encrypted_log_hashes(3);
    builder.output.append_note_encrypted_log_hashes(3);
    // Swap the items in the output, making them out of order, and then hash.
    swap_items(&mut builder.output.note_encrypted_logs_hashes, 0, 2);
    builder.output.hash_note_encrypted_log_hashes();

    builder.validate();
}

/**
 * encrypted_log_hashes
 */

#[test]
fn validate_accumulated_values_encrypted_log_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_encrypted_log_hashes(3);
    builder.output.append_encrypted_log_hashes(3);
    builder.output.hash_encrypted_log_hashes();

    builder.validate();
}

#[test]
fn validate_accumulated_values_encrypted_log_hashes_unordered_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_encrypted_log_hashes(3);
    // Swap the items at index 0 and 2.
    swap_items(&mut builder.previous_kernel.encrypted_logs_hashes, 0, 2);
    builder.output.append_encrypted_log_hashes(3);
    builder.output.hash_encrypted_log_hashes();

    builder.validate();
}

#[test(should_fail_with="mismatch encrypted_logs_hash")]
fn validate_accumulated_values_encrypted_log_hashes_wrong_hash_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_encrypted_log_hashes(3);
    builder.output.append_encrypted_log_hashes(3);
    // Swap the items in the output, making them out of order, and then hash.
    swap_items(&mut builder.output.encrypted_logs_hashes, 0, 2);
    builder.output.hash_encrypted_log_hashes();

    builder.validate();
}

/**
 * unencrypted_log_hashes
 */

#[test]
fn validate_accumulated_values_unencrypted_log_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_unencrypted_log_hashes(3);
    builder.output.append_unencrypted_log_hashes(3);
    builder.output.hash_unencrypted_log_hashes();

    builder.validate();
}

#[test]
fn validate_accumulated_values_unencrypted_log_hashes_unordered_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_unencrypted_log_hashes(3);
    // Swap the items at index 0 and 2.
    swap_items(&mut builder.previous_kernel.unencrypted_logs_hashes, 0, 2);
    builder.output.append_unencrypted_log_hashes(3);
    builder.output.hash_unencrypted_log_hashes();

    builder.validate();
}

#[test(should_fail_with="mismatch unencrypted_logs_hashes")]
fn validate_accumulated_values_unencrypted_log_hashes_wrong_order_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_unencrypted_log_hashes(3);
    builder.output.append_unencrypted_log_hashes(3);
    // Swap the items in the output, making them out of order
    swap_items(&mut builder.output.unencrypted_logs_hashes, 0, 2);

    builder.validate();
}
