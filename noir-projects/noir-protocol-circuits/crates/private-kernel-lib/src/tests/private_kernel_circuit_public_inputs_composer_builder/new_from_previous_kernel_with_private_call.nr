use crate::{tests::private_kernel_circuit_public_inputs_composer_builder::PrivateKernelCircuitPublicInputsComposerBuilder};
use dep::types::{
    abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputsArrayLengths, traits::is_empty,
    tests::utils::assert_array_eq
};

#[test]
fn new_from_previous_kernel_with_private_call_empty_data_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    let output = builder.compose_from_previous_kernel();

    let array_lengths = PrivateKernelCircuitPublicInputsArrayLengths::new(output);
    let expected_array_lengths = PrivateKernelCircuitPublicInputsArrayLengths::empty();
    assert_eq(array_lengths, expected_array_lengths);

    assert(output.validation_requests.for_rollup.max_block_number.is_none());
    assert(is_empty(output.public_teardown_call_request));
    assert(is_empty(output.fee_payer));
}

#[test]
fn new_from_previous_kernel_with_private_call_min_revertible_side_effect_counter_prev_empty_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 123;

    let output = builder.compose_from_previous_kernel();

    assert_eq(output.min_revertible_side_effect_counter, 123);
}

#[test]
fn new_from_previous_kernel_with_private_call_min_revertible_side_effect_counter_curr_empty_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.min_revertible_side_effect_counter = 123;

    let output = builder.compose_from_previous_kernel();

    assert_eq(output.min_revertible_side_effect_counter, 123);
}

#[test(should_fail_with="cannot overwrite non-zero min_revertible_side_effect_counter")]
fn new_from_previous_kernel_with_private_call_min_revertible_side_effect_counter_overwrite_fails() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.min_revertible_side_effect_counter = 123;
    builder.private_call.min_revertible_side_effect_counter = 123;

    let _ = builder.compose_from_previous_kernel();
}

#[test]
fn new_from_previous_kernel_with_private_call_max_block_number_prev_empty_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.set_max_block_number(123);

    let output = builder.compose_from_previous_kernel();

    assert_eq(output.validation_requests.for_rollup.max_block_number.unwrap(), 123);
}

#[test]
fn new_from_previous_kernel_with_private_call_max_block_number_curr_empty_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.set_max_block_number(123);

    let output = builder.compose_from_previous_kernel();

    assert_eq(output.validation_requests.for_rollup.max_block_number.unwrap(), 123);
}

#[test]
fn new_from_previous_kernel_with_private_call_max_block_number_pick_prev_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.set_max_block_number(123);
    builder.private_call.set_max_block_number(4567);

    let output = builder.compose_from_previous_kernel();

    assert_eq(output.validation_requests.for_rollup.max_block_number.unwrap(), 123);
}

#[test]
fn new_from_previous_kernel_with_private_call_max_block_number_pick_curr_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.set_max_block_number(4567);
    builder.private_call.set_max_block_number(123);

    let output = builder.compose_from_previous_kernel();

    assert_eq(output.validation_requests.for_rollup.max_block_number.unwrap(), 123);
}

#[test]
fn new_from_previous_kernel_with_private_call_note_hash_read_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_note_hash_read_requests(2);
    let prev = builder.previous_kernel.note_hash_read_requests.storage;
    builder.private_call.append_note_hash_read_requests(2);
    let curr = builder.private_call.note_hash_read_requests.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(
        output.validation_requests.note_hash_read_requests,
        [prev[0], prev[1], curr[0], curr[1]]
    );
}

#[test]
fn new_from_previous_kernel_with_private_call_nullifier_read_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_nullifier_read_requests(2);
    let prev = builder.previous_kernel.nullifier_read_requests.storage;
    builder.private_call.append_nullifier_read_requests(2);
    let curr = builder.private_call.nullifier_read_requests.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(
        output.validation_requests.nullifier_read_requests,
        [prev[0], prev[1], curr[0], curr[1]]
    );
}

#[test]
fn new_from_previous_kernel_with_private_call_key_validation_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_key_validation_requests(2);
    let prev = builder.previous_kernel.scoped_key_validation_requests_and_generators.storage;
    builder.private_call.append_key_validation_requests(2);
    let curr = builder.private_call.scoped_key_validation_requests_and_generators.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(
        output.validation_requests.scoped_key_validation_requests_and_generators,
        [prev[0], prev[1], curr[0], curr[1]]
    );
}

#[test]
fn new_from_previous_kernel_with_private_call_note_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_note_hashes(2);
    let prev = builder.previous_kernel.note_hashes.storage;
    builder.private_call.append_note_hashes(2);
    let curr = builder.private_call.note_hashes.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(output.end.note_hashes, [prev[0], prev[1], curr[0], curr[1]]);
}

#[test]
fn new_from_previous_kernel_with_private_call_nullifiers_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_nullifiers(2);
    let prev = builder.previous_kernel.nullifiers.storage;
    builder.private_call.append_nullifiers(2);
    let curr = builder.private_call.nullifiers.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(output.end.nullifiers, [prev[0], prev[1], curr[0], curr[1]]);
}

#[test]
fn new_from_previous_kernel_with_private_call_l2_to_l1_msgs_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(1);
    let prev = builder.previous_kernel.l2_to_l1_msgs.storage;
    builder.private_call.append_l2_to_l1_msgs(1);
    let curr = builder.private_call.l2_to_l1_msgs.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(output.end.l2_to_l1_msgs, [prev[0], curr[0]]);
}

#[test]
fn new_from_previous_kernel_with_private_call_note_encrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_note_encrypted_log_hashes(2);
    let prev = builder.previous_kernel.note_encrypted_logs_hashes.storage;
    builder.private_call.append_note_encrypted_log_hashes(2);
    let curr = builder.private_call.note_encrypted_logs_hashes.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(
        output.end.note_encrypted_logs_hashes,
        [prev[0], prev[1], curr[0], curr[1]]
    );
}

#[test]
fn new_from_previous_kernel_with_private_call_encrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_encrypted_log_hashes(2);
    let prev = builder.previous_kernel.encrypted_logs_hashes.storage;
    builder.private_call.append_encrypted_log_hashes(2);
    let curr = builder.private_call.encrypted_logs_hashes.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(
        output.end.encrypted_logs_hashes,
        [prev[0], prev[1], curr[0], curr[1]]
    );
}

#[test]
fn new_from_previous_kernel_with_private_call_unencrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_unencrypted_log_hashes(2);
    let prev = builder.previous_kernel.unencrypted_logs_hashes.storage;
    builder.private_call.append_unencrypted_log_hashes(2);
    let curr = builder.private_call.unencrypted_logs_hashes.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(
        output.end.unencrypted_logs_hashes,
        [prev[0], prev[1], curr[0], curr[1]]
    );
}

#[test]
fn new_from_previous_kernel_with_private_call_private_call_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_private_call_requests(2);
    let prev = builder.previous_kernel.private_call_requests.storage;
    builder.private_call.append_private_call_requests(2);
    let curr = builder.private_call.private_call_requests.storage;

    let output = builder.compose_from_previous_kernel();

    // Call requests from private call will be propagated in reversed order.
    assert_array_eq(
        output.end.private_call_stack,
        [prev[1], prev[0], curr[1], curr[0]]
    );
}

#[test]
fn new_from_previous_kernel_with_private_call_public_call_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_public_call_requests(2);
    let prev = builder.previous_kernel.public_call_requests.storage;
    builder.private_call.append_public_call_requests(2);
    let curr = builder.private_call.public_call_requests.storage;

    let output = builder.compose_from_previous_kernel();

    assert_array_eq(
        output.end.public_call_requests,
        [prev[0], prev[1], curr[0], curr[1]]
    );
}

#[test]
fn new_from_previous_kernel_with_private_call_public_teardown_call_request_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_public_teardown_call_request();
    let request = builder.private_call.public_teardown_call_stack.storage[0];

    let output = builder.compose_from_previous_kernel();

    assert_eq(output.public_teardown_call_request, request);
}

#[test(should_fail_with="Public teardown call request already set")]
fn new_from_previous_kernel_with_private_call_public_teardown_call_request_overwrite_fails() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.previous_kernel.append_public_teardown_call_request();
    builder.private_call.append_public_teardown_call_request();

    let _ = builder.compose_from_previous_kernel();
}

#[test]
fn new_from_previous_kernel_with_private_call_fee_payer_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    let fee_payer = builder.private_call.make_fee_payer();

    let output = builder.compose_from_previous_kernel();

    assert_eq(output.fee_payer, fee_payer);
}

#[test(should_fail_with="Cannot overwrite non-empty fee_payer")]
fn new_from_previous_kernel_with_private_call_fee_payer_overwrite_fails() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    let _ = builder.previous_kernel.make_fee_payer();
    let _ = builder.private_call.make_fee_payer();

    let _ = builder.compose_from_previous_kernel();
}
