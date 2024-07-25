use crate::tests::private_kernel_circuit_output_validator_builder::PrivateKernelCircuitOutputValidatorBuilder;

#[test]
fn validate_aggregated_values_empty_data_succeeds() {
    let builder = PrivateKernelCircuitOutputValidatorBuilder::new();
    builder.validate_as_inner_call();
}

/**
 * min_revertible_side_effect_counter
 */

#[test]
fn validate_aggregated_values_min_revertible_side_effect_counter_from_previous_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.min_revertible_side_effect_counter = 123;
    builder.output.min_revertible_side_effect_counter = 123;

    builder.validate_as_inner_call();
}

#[test]
fn validate_aggregated_values_min_revertible_side_effect_counter_from_private_call_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 123;
    builder.output.min_revertible_side_effect_counter = 123;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="cannot overwrite min_revertible_side_effect_counter")]
fn validate_aggregated_values_min_revertible_side_effect_counter_overwrite_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.min_revertible_side_effect_counter = 123;
    builder.private_call.min_revertible_side_effect_counter = 4567;
    builder.output.min_revertible_side_effect_counter = 123;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output min_revertible_side_effect_counter")]
fn validate_aggregated_values_min_revertible_side_effect_counter_from_previous_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.min_revertible_side_effect_counter = 123;
    builder.output.min_revertible_side_effect_counter = 4567;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output min_revertible_side_effect_counter")]
fn validate_aggregated_values_min_revertible_side_effect_counter_from_private_call_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 123;
    builder.output.min_revertible_side_effect_counter = 4567;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output min_revertible_side_effect_counter")]
fn validate_aggregated_values_min_revertible_side_effect_counter_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.min_revertible_side_effect_counter = 123;

    builder.validate_as_inner_call();
}

/**
 * max_block_number
 */

#[test]
fn validate_aggregated_values_max_block_number_from_previous_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(123);
    builder.output.set_max_block_number(123);

    builder.validate_as_inner_call();
}

#[test]
fn validate_aggregated_values_max_block_number_from_private_call_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.set_max_block_number(123);
    builder.output.set_max_block_number(123);

    builder.validate_as_inner_call();
}

#[test]
fn validate_aggregated_values_max_block_number_from_both_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(123);
    builder.private_call.set_max_block_number(123);
    builder.output.set_max_block_number(123);

    builder.validate_as_inner_call();
}

#[test]
fn validate_aggregated_values_max_block_number_from_both_pick_previous_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(123);
    builder.private_call.set_max_block_number(4567);
    builder.output.set_max_block_number(123);

    builder.validate_as_inner_call();
}

#[test]
fn validate_aggregated_values_max_block_number_from_both_pick_private_call_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(4567);
    builder.private_call.set_max_block_number(123);
    builder.output.set_max_block_number(123);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output max_block_number")]
fn validate_aggregated_values_max_block_number_from_both_pick_larger_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(4567);
    builder.private_call.set_max_block_number(123);
    builder.output.set_max_block_number(4567);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output max_block_number")]
fn validate_aggregated_values_max_block_number_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.set_max_block_number(123);

    builder.validate_as_inner_call();
}

/**
 * public_teardown_call_request
 */

#[test]
fn validate_aggregated_values_public_teardown_call_request_from_previous_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_public_teardown_call_request();
    builder.output.append_public_teardown_call_request();

    builder.validate_as_inner_call();
}

#[test]
fn validate_aggregated_values_public_teardown_call_request_from_private_call_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request();
    builder.output.append_public_teardown_call_request();

    builder.validate_as_inner_call();
}

#[test(should_fail_with="cannot overwrite public_teardown_call_request")]
fn validate_aggregated_values_public_teardown_call_request_from_both_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_public_teardown_call_request();
    builder.private_call.append_public_teardown_call_request();
    builder.output.append_public_teardown_call_request();

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output public_teardown_call_request")]
fn validate_aggregated_values_public_teardown_call_request_from_previous_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_public_teardown_call_request();
    builder.output.append_public_teardown_call_request();
    // Tweak the output.
    builder.output.public_teardown_call_stack.storage[0].item.args_hash += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output public_teardown_call_request")]
fn validate_aggregated_values_public_teardown_call_request_from_private_call_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request();
    builder.output.append_public_teardown_call_request();
    // Tweak the output.
    builder.output.public_teardown_call_stack.storage[0].item.args_hash += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output public_teardown_call_request")]
fn validate_aggregated_values_public_teardown_call_request_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.append_public_teardown_call_request();

    builder.validate_as_inner_call();
}

/**
 * fee_payer
 */

#[test]
fn validate_aggregated_values_fee_payer_from_previous_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.previous_kernel.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);

    builder.validate_as_inner_call();
}

#[test]
fn validate_aggregated_values_fee_payer_from_private_call_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.private_call.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="cannot overwrite fee_payer")]
fn validate_aggregated_values_fee_payer_from_both_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.previous_kernel.make_fee_payer();
    let fee_payer = builder.private_call.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output fee_payer")]
fn validate_aggregated_values_fee_payer_from_previous_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.previous_kernel.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);
    // Tweak the output.
    builder.output.fee_payer.inner += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output fee_payer")]
fn validate_aggregated_values_fee_payer_from_private_call_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    let fee_payer = builder.private_call.make_fee_payer();
    builder.output.set_fee_payer(fee_payer);
    // Tweak the output.
    builder.output.fee_payer.inner += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="incorrect output fee_payer")]
fn validate_aggregated_values_fee_payer_random_output_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.set_fee_payer(builder.private_call.storage_contract_address);

    builder.validate_as_inner_call();
}
