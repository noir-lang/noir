use crate::tests::tail_output_validator_builder::TailOutputValidatorBuilder;
use dep::types::{address::AztecAddress, traits::FromField};

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

#[test(should_fail_with = "mismatch historical_header")]
fn validate_propagated_values_total_fees_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.historical_header.total_fees = 123;
    // Tweak the value in the output.
    builder.output.historical_header.total_fees = 45;

    builder.validate();
}

#[test(should_fail_with = "mismatch tx_context")]
fn validate_propagated_values_chain_id_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.tx_context.chain_id = 123;
    // Tweak the value in the output.
    builder.output.tx_context.chain_id = 45;

    builder.validate();
}

#[test(should_fail_with = "mismatch vk_tree_root")]
fn validate_propagated_values_vk_tree_root_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.vk_tree_root = 123;
    // Tweak the value in the output.
    builder.output.vk_tree_root = 45;

    builder.validate();
}

#[test(should_fail_with = "mismatch protocol_contract_tree_root")]
fn validate_propagated_values_protocol_contract_tree_root_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.protocol_contract_tree_root = 123;
    // Tweak the value in the output.
    builder.output.protocol_contract_tree_root = 45;

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

#[test(should_fail_with = "mismatch rollup_validation_requests")]
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

#[test(should_fail_with = "mismatch fee_payer")]
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

#[test(should_fail_with = "mismatch note_hashes")]
fn validate_propagated_values_note_hashes_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(3);
    builder.output.append_note_hashes(3);

    // Tweak the value at index 1.
    let mut note_hash = builder.output.note_hashes.get(1);
    note_hash.note_hash.value += 1;
    builder.output.note_hashes.set(1, note_hash);

    builder.validate();
}

#[test(should_fail_with = "mismatch note_hashes")]
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

#[test(should_fail_with = "mismatch nullifiers")]
fn validate_propagated_values_nullifiers_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(3);
    builder.output.append_nullifiers(3);

    // Tweak the value at index 1.
    let mut nullifier = builder.output.nullifiers.get(1);
    nullifier.nullifier.value += 1;
    builder.output.nullifiers.set(1, nullifier);

    builder.validate();
}

#[test(should_fail_with = "mismatch nullifiers")]
fn validate_propagated_values_nullifiers_extra_item_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(3);
    // Append 1 more item.
    builder.output.append_nullifiers(4);

    builder.validate();
}

/**
 * private_logs
 */
#[test]
fn validate_propagated_values_private_logs_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_private_logs(3);
    builder.output.append_private_logs(3);

    builder.validate();
}

#[test(should_fail_with = "mismatch private_logs")]
fn validate_propagated_values_private_logs_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.append_private_logs(3);
    builder.output.append_private_logs(3);

    // Tweak the value at index 1.
    let mut private_log = builder.output.private_logs.get(1);
    private_log.inner.log.fields[0] += 1;
    builder.output.private_logs.set(1, private_log);

    builder.validate();
}
