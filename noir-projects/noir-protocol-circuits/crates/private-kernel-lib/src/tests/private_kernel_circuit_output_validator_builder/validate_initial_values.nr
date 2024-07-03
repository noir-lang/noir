use crate::tests::private_kernel_circuit_output_validator_builder::PrivateKernelCircuitOutputValidatorBuilder;
use dep::types::address::AztecAddress;

#[test]
fn validate_initial_values_empty_data_succeeds() {
    let builder = PrivateKernelCircuitOutputValidatorBuilder::new();
    builder.validate_as_first_call();
}

/**
 * constants
 */

#[test(should_fail_with="mismatch tx_context")]
fn validate_initial_values_constants_mismatch_chain_id_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.tx_context.chain_id += 1;

    builder.validate_as_first_call();
}

#[test(should_fail_with="mismatch historical_header")]
fn validate_initial_values_constants_mismatch_txs_effects_hash_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.historical_header.content_commitment.txs_effects_hash += 1;

    builder.validate_as_first_call();
}

#[test(should_fail_with="constants.global_variables must be empty")]
fn validate_initial_values_constants_non_empty_global_variables_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.global_variables.block_number += 1;

    builder.validate_as_first_call();
}

/**
 * First nullifier.
 */

#[test(should_fail_with="first nullifier must be the tx request nullifier")]
fn validate_initial_values_constants_incorrect_first_nullifier_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.nullifiers.storage[0].nullifier.value += 1;

    builder.validate_as_first_call();
}

#[test(should_fail_with="first nullifier must be the tx request nullifier")]
fn validate_initial_values_constants_empty_first_nullifier_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.nullifiers = BoundedVec::new();

    builder.validate_as_first_call();
}

/**
 * min_revertible_side_effect_counter
 */

#[test]
fn validate_initial_values_min_revertible_side_effect_counter_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 8989;
    builder.output.min_revertible_side_effect_counter = 8989;

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial min_revertible_side_effect_counter")]
fn validate_initial_values_min_revertible_side_effect_counter_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 8989;
    builder.output.min_revertible_side_effect_counter = 50;

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial min_revertible_side_effect_counter")]
fn validate_initial_values_min_revertible_side_effect_counter_empty_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 8989;

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial min_revertible_side_effect_counter")]
fn validate_initial_values_min_revertible_side_effect_counter_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.min_revertible_side_effect_counter = 8989;

    builder.validate_as_first_call();
}

/**
 * max_block_number
 */

#[test]
fn validate_initial_values_max_block_number_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_max_block_number(123);
    builder.output.set_max_block_number(123);

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial max_block_number")]
fn validate_initial_values_max_block_number_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_max_block_number(4567);
    builder.output.set_max_block_number(123);

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial max_block_number")]
fn validate_initial_values_max_block_number_empty_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_max_block_number(4567);

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial max_block_number")]
fn validate_initial_values_max_block_number_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.set_max_block_number(123);

    builder.validate_as_first_call();
}

/**
 * public_teardown_call_request
 */

#[test]
fn validate_initial_values_public_teardown_call_request_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request();
    builder.output.append_public_teardown_call_request();

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial public_teardown_call_request")]
fn validate_initial_values_public_teardown_call_request_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request();
    builder.output.append_public_teardown_call_request();
    // Tweak the output.
    builder.output.public_teardown_call_stack.storage[0].hash += 1;

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial public_teardown_call_request")]
fn validate_initial_values_public_teardown_call_request_empty_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request();

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial public_teardown_call_request")]
fn validate_initial_values_public_teardown_call_request_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.append_public_teardown_call_request();

    builder.validate_as_first_call();
}

/**
 * fee_payer
 */

#[test]
fn validate_initial_values_fee_payer_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.private_call.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial fee_payer")]
fn validate_initial_values_fee_payer_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.private_call.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);
    // Tweak the output.
    builder.output.fee_payer.inner += 1;

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial fee_payer")]
fn validate_initial_values_fee_payer_empty_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let _ = builder.private_call.make_fee_payer();

    builder.validate_as_first_call();
}

#[test(should_fail_with="incorrect initial fee_payer")]
fn validate_initial_values_fee_payer_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.set_fee_payer(AztecAddress::from_field(123));

    builder.validate_as_first_call();
}
