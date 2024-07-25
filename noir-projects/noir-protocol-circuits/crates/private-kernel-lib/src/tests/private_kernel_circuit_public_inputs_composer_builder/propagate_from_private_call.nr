use crate::{
    components::private_kernel_circuit_public_inputs_composer::create_first_nullifier,
    tests::private_kernel_circuit_public_inputs_composer_builder::PrivateKernelCircuitPublicInputsComposerBuilder
};
use dep::types::{
    abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputsArrayLengths, traits::is_empty,
    tests::utils::assert_array_eq
};

#[test]
fn propagate_from_private_call_empty_data_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    let tx_request = builder.tx_request;
    let first_nullifier = create_first_nullifier(tx_request);

    let output = builder.compose_from_tx_request();

    assert_eq(output.end.nullifiers[0], first_nullifier);

    let array_lengths = PrivateKernelCircuitPublicInputsArrayLengths::new(output);
    let mut expected_array_lengths = PrivateKernelCircuitPublicInputsArrayLengths::empty();
    expected_array_lengths.nullifiers = 1;
    assert_eq(array_lengths, expected_array_lengths);

    assert_eq(output.min_revertible_side_effect_counter, 0);
    assert(is_empty(output.validation_requests));
    assert(output.validation_requests.for_rollup.max_block_number.is_none());
    assert(is_empty(output.public_teardown_call_request));
    assert(is_empty(output.fee_payer));
}

#[test]
fn propagate_from_private_call_min_revertible_side_effect_counter_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.min_revertible_side_effect_counter = 123;

    let output = builder.compose_from_tx_request();

    assert_eq(output.min_revertible_side_effect_counter, 123);
}

#[test]
fn propagate_from_private_call_max_block_number_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.set_max_block_number(123);

    let output = builder.compose_from_tx_request();

    assert_eq(output.validation_requests.for_rollup.max_block_number.unwrap(), 123);
}

#[test]
fn propagate_from_private_call_note_hash_read_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_note_hash_read_requests(2);
    let res = builder.private_call.note_hash_read_requests.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(
        output.validation_requests.note_hash_read_requests,
        [res[0], res[1]]
    );
}

#[test]
fn propagate_from_private_call_nullifier_read_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_nullifier_read_requests(2);
    let res = builder.private_call.nullifier_read_requests.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(
        output.validation_requests.nullifier_read_requests,
        [res[0], res[1]]
    );
}

#[test]
fn propagate_from_private_call_key_validation_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_key_validation_requests(2);
    let res = builder.private_call.scoped_key_validation_requests_and_generators.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(
        output.validation_requests.scoped_key_validation_requests_and_generators,
        [res[0], res[1]]
    );
}

#[test]
fn propagate_from_private_call_note_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_note_hashes(2);
    let res = builder.private_call.note_hashes.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(output.end.note_hashes, [res[0], res[1]]);
}

#[test]
fn propagate_from_private_call_nullifiers_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_nullifiers(2);
    let res = builder.private_call.nullifiers.storage;

    let tx_request = builder.tx_request;
    let first_nullifier = create_first_nullifier(tx_request);

    let output = builder.compose_from_tx_request();

    assert_array_eq(output.end.nullifiers, [first_nullifier, res[0], res[1]]);
}

#[test]
fn propagate_from_private_call_l2_to_l1_msgs_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_l2_to_l1_msgs(2);
    let res = builder.private_call.l2_to_l1_msgs.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(output.end.l2_to_l1_msgs, [res[0], res[1]]);
}

#[test]
fn propagate_from_private_call_note_encrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_note_encrypted_log_hashes(2);
    let res = builder.private_call.note_encrypted_logs_hashes.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(output.end.note_encrypted_logs_hashes, [res[0], res[1]]);
}

#[test]
fn propagate_from_private_call_encrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_encrypted_log_hashes(2);
    let res = builder.private_call.encrypted_logs_hashes.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(output.end.encrypted_logs_hashes, [res[0], res[1]]);
}

#[test]
fn propagate_from_private_call_unencrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_unencrypted_log_hashes(2);
    let res = builder.private_call.unencrypted_logs_hashes.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(output.end.unencrypted_logs_hashes, [res[0], res[1]]);
}

#[test]
fn propagate_from_private_call_private_call_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_private_call_requests(2);
    let res = builder.private_call.private_call_requests.storage;

    let output = builder.compose_from_tx_request();

    // Call requests will be propagated in reversed order.
    assert_array_eq(output.end.private_call_stack, [res[1], res[0]]);
}

#[test]
fn propagate_from_private_call_public_call_requests_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_public_call_requests(2);
    let res = builder.private_call.public_call_requests.storage;

    let output = builder.compose_from_tx_request();

    assert_array_eq(output.end.public_call_requests, [res[0], res[1]]);
}

#[test]
fn propagate_from_private_call_public_teardown_call_request_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    builder.private_call.append_public_teardown_call_request();
    let res = builder.private_call.public_teardown_call_stack.storage[0];

    let output = builder.compose_from_tx_request();

    assert_eq(output.public_teardown_call_request, res);
}

#[test]
fn propagate_from_private_call_fee_payer_succeeds() {
    let mut builder = PrivateKernelCircuitPublicInputsComposerBuilder::new();

    let fee_payer = builder.private_call.make_fee_payer();

    let output = builder.compose_from_tx_request();

    assert_eq(output.fee_payer, fee_payer);
}
