use crate::tests::tail_output_validator_builder::TailOutputValidatorBuilder;
use dep::types::address::AztecAddress;

/**
 * constants
 */

#[test]
fn validate_propagated_values_constants_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.historical_header.total_fees = 123;
    builder.output.historical_header.total_fees = 123;

    builder.validate();
}

#[test(should_fail_with="mismatch constants")]
fn validate_propagated_values_constants_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.historical_header.total_fees = 123;
    // Tweak the value in the output.
    builder.output.historical_header.total_fees = 45;

    builder.validate();
}

/**
 * max_block_number
 */

#[test]
fn validate_propagated_values_max_block_number_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(123);
    builder.output.set_max_block_number(123);

    builder.validate();
}

#[test(should_fail_with="mismatch rollup_validation_requests")]
fn validate_propagated_values_max_block_number_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(123);
    // Tweak the value in the output.
    builder.output.set_max_block_number(45);

    builder.validate();
}

/**
 * fee_payer
 */

#[test]
fn validate_propagated_values_fee_payer_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.set_fee_payer(AztecAddress::from_field(123));
    builder.output.set_fee_payer(AztecAddress::from_field(123));

    builder.validate();
}

#[test(should_fail_with="mismatch fee_payer")]
fn validate_propagated_values_fee_payer_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.set_fee_payer(AztecAddress::from_field(123));
    // Tweak the value in the output.
    builder.output.set_fee_payer(AztecAddress::from_field(45));

    builder.validate();
}

/**
 * note_hashes
 */

#[test]
fn validate_propagated_values_note_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(3);
    builder.output.append_note_hashes(3);

    builder.validate();
}

#[test(should_fail_with="mismatch note_hashes")]
fn validate_propagated_values_note_hashes_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(3);
    builder.output.append_note_hashes(3);

    // Tweak the value at index 1.
    builder.output.note_hashes.storage[1].note_hash.value += 1;

    builder.validate();
}

#[test(should_fail_with="mismatch note_hashes")]
fn validate_propagated_values_note_hashes_extra_item_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(3);
    // Append 1 more item.
    builder.output.append_note_hashes(4);

    builder.validate();
}

/**
 * nullifiers
 */

#[test]
fn validate_propagated_values_nullifiers_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(3);
    builder.output.append_nullifiers(3);

    builder.validate();
}

#[test(should_fail_with="mismatch nullifiers")]
fn validate_propagated_values_nullifiers_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(3);
    builder.output.append_nullifiers(3);

    // Tweak the value at index 1.
    builder.output.nullifiers.storage[1].nullifier.value += 1;

    builder.validate();
}

#[test(should_fail_with="mismatch nullifiers")]
fn validate_propagated_values_nullifiers_extra_item_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(3);
    // Append 1 more item.
    builder.output.append_nullifiers(4);

    builder.validate();
}

/**
 * note_encrypted_log_hashes
 */

#[test]
fn validate_propagated_values_note_encrypted_log_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_encrypted_log_hashes(3);
    builder.output.append_note_encrypted_log_hashes(3);

    builder.validate();
}

#[test(should_fail_with="mismatch note_encrypted_logs_hashes")]
fn validate_propagated_values_note_encrypted_log_hashes_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_encrypted_log_hashes(3);
    builder.output.append_note_encrypted_log_hashes(3);

    // Tweak the value at index 1.
    builder.output.note_encrypted_logs_hashes.storage[1].value += 1;

    builder.validate();
}

#[test(should_fail_with="mismatch note_encrypted_logs_hashes")]
fn validate_propagated_values_note_encrypted_log_hashes_non_zero_counter_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_encrypted_log_hashes(3);
    builder.output.append_note_encrypted_log_hashes(3);

    let mut output = builder.export_output();
    // Set the counter at index 1.
    output.end.note_encrypted_logs_hashes[1].counter = builder.previous_kernel.note_encrypted_logs_hashes.storage[1].counter;

    builder.validate_with_output(output);
}

/**
 * encrypted_log_hashes
 */

#[test]
fn validate_propagated_values_encrypted_log_hashes_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_encrypted_log_hashes(3);
    builder.output.append_encrypted_log_hashes(3);

    builder.validate();
}

#[test(should_fail_with="mismatch encrypted_logs_hashes")]
fn validate_propagated_values_encrypted_logs_hashes_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_encrypted_log_hashes(3);
    builder.output.append_encrypted_log_hashes(3);

    // Tweak the value at index 1.
    builder.output.encrypted_logs_hashes.storage[1].log_hash.value += 1;

    builder.validate();
}

#[test(should_fail_with="mismatch encrypted_logs_hashes")]
fn validate_propagated_values_encrypted_logs_hashes_non_zero_counter_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_encrypted_log_hashes(3);
    builder.output.append_encrypted_log_hashes(3);

    let mut output = builder.export_output();
    // Set the counter at index 1.
    output.end.encrypted_logs_hashes[1].log_hash.counter = builder.previous_kernel.encrypted_logs_hashes.storage[1].log_hash.counter;

    builder.validate_with_output(output);
}
