use crate::tests::private_kernel_circuit_output_validator_builder::PrivateKernelCircuitOutputValidatorBuilder;
use dep::types::{address::AztecAddress, traits::FromField};

#[test]
fn validate_initial_values_empty_data_succeeds() {
    let builder = PrivateKernelCircuitOutputValidatorBuilder::new();
    builder.validate_as_first_call(false);
}

/**
 * constants
 */
#[test(should_fail_with = "mismatch tx_context")]
fn validate_initial_values_constants_mismatch_chain_id_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.tx_context.chain_id += 1;

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "mismatch is_private_only")]
fn validate_initial_values_is_private_only_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.is_private_only = true;

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "mismatch historical_header")]
fn validate_initial_values_constants_mismatch_out_hash_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.historical_header.content_commitment.out_hash += 1;

    builder.validate_as_first_call(false);
}

/**
 * First nullifier.
 */
#[test(should_fail_with = "protocol nullifier must be the tx request nullifier")]
fn validate_initial_values_constants_incorrect_protocol_nullifier_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();
    builder.with_protocol_nullifier();

    let mut nullifier = builder.output.nullifiers.get(0);
    nullifier.nullifier.value += 1;
    builder.output.nullifiers.set(0, nullifier);

    builder.validate_as_first_call(false);
}

/**
 * min_revertible_side_effect_counter
 */
#[test]
fn validate_initial_values_min_revertible_side_effect_counter_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 8989;
    builder.output.min_revertible_side_effect_counter = 8989;

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial min_revertible_side_effect_counter")]
fn validate_initial_values_min_revertible_side_effect_counter_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 8989;
    builder.output.min_revertible_side_effect_counter = 50;

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial min_revertible_side_effect_counter")]
fn validate_initial_values_min_revertible_side_effect_counter_empty_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 8989;

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial min_revertible_side_effect_counter")]
fn validate_initial_values_min_revertible_side_effect_counter_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.min_revertible_side_effect_counter = 8989;

    builder.validate_as_first_call(false);
}

/**
 * max_block_number
 */
#[test]
fn validate_initial_values_max_block_number_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_max_block_number(3);
    builder.output.set_max_block_number(3);

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial max_block_number")]
fn validate_initial_values_max_block_number_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_max_block_number(4);
    builder.output.set_max_block_number(3);

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial max_block_number")]
fn validate_initial_values_max_block_number_empty_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_max_block_number(4);

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial max_block_number")]
fn validate_initial_values_max_block_number_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.set_max_block_number(3);

    builder.validate_as_first_call(false);
}

/**
 * public_teardown_call_request
 */
#[test]
fn validate_initial_values_public_teardown_call_request_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_public_teardown_call_request();
    builder.output.set_public_teardown_call_request();

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial public_teardown_call_request")]
fn validate_initial_values_public_teardown_call_request_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_public_teardown_call_request();
    builder.output.set_public_teardown_call_request();
    // Tweak the output.
    builder.output.public_teardown_call_request.calldata_hash += 1;

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial public_teardown_call_request")]
fn validate_initial_values_public_teardown_call_request_empty_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_public_teardown_call_request();

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial public_teardown_call_request")]
fn validate_initial_values_public_teardown_call_request_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.set_public_teardown_call_request();

    builder.validate_as_first_call(false);
}

/**
 * fee_payer
 */
#[test]
fn validate_initial_values_fee_payer_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.private_call.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial fee_payer")]
fn validate_initial_values_fee_payer_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.private_call.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);
    // Tweak the output.
    builder.output.fee_payer.inner += 1;

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial fee_payer")]
fn validate_initial_values_fee_payer_empty_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let _ = builder.private_call.make_fee_payer();

    builder.validate_as_first_call(false);
}

#[test(should_fail_with = "incorrect initial fee_payer")]
fn validate_initial_values_fee_payer_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.set_fee_payer(AztecAddress::from_field(3));

    builder.validate_as_first_call(false);
}
