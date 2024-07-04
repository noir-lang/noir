use crate::tests::private_kernel_circuit_output_validator_builder::PrivateKernelCircuitOutputValidatorBuilder;

#[test]
fn validate_propagated_from_private_call_empty_data_succeeds() {
    let builder = PrivateKernelCircuitOutputValidatorBuilder::new();
    builder.validate_as_inner_call();
}

/**
 * note_hash_read_requests
 * With private call data only.
 */

#[test]
fn validate_propagated_from_private_call_note_hash_read_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="source item does not append to dest")]
fn validate_propagated_from_private_call_note_hash_read_requests_output_mismatch_value_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(1);
    // Tweak the value in the output.
    builder.output.note_hash_read_requests.storage[1].read_request.value += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="propagated contract address does not match")]
fn validate_propagated_from_private_call_note_hash_read_requests_mismatch_contract_address_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(2);
    // Tweak the contract address in the output.
    builder.output.note_hash_read_requests.storage[1].contract_address.inner += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="source item does not append to dest")]
fn validate_propagated_from_private_call_note_hash_read_requests_output_one_less_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_hash_read_requests(2);
    // Propagate 1 less item to the output.
    builder.output.append_note_hash_read_requests(1);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_note_hash_read_requests_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_hash_read_requests(2);
    // Propagate 1 more item to the output.
    builder.output.append_note_hash_read_requests(3);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_note_hash_read_requests_output_extra_non_empty_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(2);
    // Add a non-empty item to the end of the output.
    let len = builder.output.note_hash_read_requests.storage.len();
    builder.output.note_hash_read_requests.storage[len - 1] = builder.output.note_hash_read_requests.storage[0];

    builder.validate_as_inner_call();
}

/**
 * note_hash_read_requests
 * With previous kernel.
 */

fn append_note_hash_read_requests_to_previous_kernel(
    builder: &mut PrivateKernelCircuitOutputValidatorBuilder,
    num_requests: u32
) {
    builder.previous_kernel.append_note_hash_read_requests(num_requests);
    builder.output.append_note_hash_read_requests(num_requests);
    builder.offset_values(num_requests as Field);
}

#[test]
fn validate_propagated_from_private_call_note_hash_read_requests_with_previous_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    append_note_hash_read_requests_to_previous_kernel(&mut builder, 3);
    builder.private_call.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="source item does not append to dest")]
fn validate_propagated_from_private_call_note_hash_read_requests_with_previous_mismatch_value_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    append_note_hash_read_requests_to_previous_kernel(&mut builder, 3);
    builder.private_call.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(2);
    // Tweak the value in the output.
    builder.output.note_hash_read_requests.storage[3].read_request.value += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="propagated contract address does not match")]
fn validate_propagated_from_private_call_note_hash_read_requests_with_previous_mismatch_contract_address_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    append_note_hash_read_requests_to_previous_kernel(&mut builder, 3);
    builder.private_call.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(2);
    // Tweak the contract address in the output.
    builder.output.note_hash_read_requests.storage[3].contract_address.inner += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with="source item does not append to dest")]
fn validate_propagated_from_private_call_note_hash_read_requests_with_previous_output_one_less_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    append_note_hash_read_requests_to_previous_kernel(&mut builder, 3);
    builder.private_call.append_note_hash_read_requests(2);
    // Propagate 1 less item to the output.
    builder.output.append_note_hash_read_requests(1);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_note_hash_read_requests_with_previous_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    append_note_hash_read_requests_to_previous_kernel(&mut builder, 3);
    builder.private_call.append_note_hash_read_requests(2);
    // Propagate 1 more item to the output.
    builder.output.append_note_hash_read_requests(3);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_note_hash_read_requests_with_previous_output_extra_non_empty_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    append_note_hash_read_requests_to_previous_kernel(&mut builder, 3);
    builder.private_call.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(2);
    // Add a non-empty item to the end of the output.
    let len = builder.output.note_hash_read_requests.storage.len();
    builder.output.note_hash_read_requests.storage[len - 1] = builder.output.note_hash_read_requests.storage[0];

    builder.validate_as_inner_call();
}

#[test(should_fail_with="number of total items exceeds limit")]
fn validate_propagated_from_private_call_note_hash_read_requests_with_previous_output_exceeds_max_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    // Make the previous array to be full, therefore no more items can be added.
    let max_len = builder.output.note_hash_read_requests.max_len();
    append_note_hash_read_requests_to_previous_kernel(&mut builder, max_len);
    // Add 1 item to the current call.
    builder.private_call.append_note_hash_read_requests(1);

    builder.validate_as_inner_call();
}

// Create two tests (one succeeds and one fails) for the rest of the side effects to make sure that they are processed in the validator.

/**
 * nullifier_read_requests
 */

#[test]
fn validate_propagated_from_private_call_nullifier_read_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_nullifier_read_requests(2);
    builder.output.append_nullifier_read_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_nullifier_read_requests_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_nullifier_read_requests(2);
    // Propagate 1 more item to the output.
    builder.output.append_nullifier_read_requests(3);

    builder.validate_as_inner_call();
}

/**
 * key_validation_requests
 */

#[test]
fn validate_propagated_from_private_call_key_validation_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_key_validation_requests(2);
    builder.output.append_key_validation_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_key_validation_requests_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_key_validation_requests(2);
    // Propagate 1 more item to the output.
    builder.output.append_key_validation_requests(3);

    builder.validate_as_inner_call();
}

/**
 * note_hashes
 */

#[test]
fn validate_propagated_from_private_call_note_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_hashes(2);
    builder.output.append_note_hashes(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_note_hashes_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_hashes(2);
    // Propagate 1 more item to the output.
    builder.output.append_note_hashes(3);

    builder.validate_as_inner_call();
}

/**
 * nullifiers
 */

#[test]
fn validate_propagated_from_private_call_nullifiers_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.offset_values(1); // Offset the first nullifier.
    builder.private_call.append_nullifiers(2);
    builder.output.append_nullifiers(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_nullifiers_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.offset_values(1); // Offset the first nullifier.
    builder.private_call.append_nullifiers(2);
    // Propagate 1 more item to the output.
    builder.output.append_nullifiers(3);

    builder.validate_as_inner_call();
}

/**
 * l2_to_l1_msgs
 */

#[test]
fn validate_propagated_from_private_call_l2_to_l1_msgs_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_l2_to_l1_msgs(2);
    builder.output.append_l2_to_l1_msgs(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_l2_to_l1_msgs_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_l2_to_l1_msgs(1);
    // Propagate 1 more item to the output.
    builder.output.append_l2_to_l1_msgs(2);

    builder.validate_as_inner_call();
}

/**
 * note_encrypted_log_hashes
 */

#[test]
fn validate_propagated_from_private_call_note_encrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_encrypted_log_hashes(2);
    builder.output.append_note_encrypted_log_hashes(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_note_encrypted_log_hashes_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_note_encrypted_log_hashes(2);
    // Propagate 1 more item to the output.
    builder.output.append_note_encrypted_log_hashes(3);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="number of total items exceeds limit")]
fn validate_propagated_from_private_call_note_encrypted_log_hashes_with_previous_output_exceeds_max_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    // Make the previous array to be full, therefore no more items can be added.
    let max_len = builder.previous_kernel.note_encrypted_logs_hashes.max_len();
    builder.previous_kernel.append_note_encrypted_log_hashes(max_len);
    builder.output.append_note_encrypted_log_hashes(max_len);
    // Add 1 item to the current call.
    builder.private_call.append_note_encrypted_log_hashes(1);

    builder.validate_as_inner_call();
}

/**
 * encrypted_log_hashes
 */

#[test]
fn validate_propagated_from_private_call_encrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_encrypted_log_hashes(2);
    builder.output.append_encrypted_log_hashes(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_encrypted_log_hashes_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_encrypted_log_hashes(2);
    // Propagate 1 more item to the output.
    builder.output.append_encrypted_log_hashes(3);

    builder.validate_as_inner_call();
}

/**
 * unencrypted_log_hashes
 */

#[test]
fn validate_propagated_from_private_call_unencrypted_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_unencrypted_log_hashes(2);
    builder.output.append_unencrypted_log_hashes(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_unencrypted_log_hashes_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_unencrypted_log_hashes(2);
    // Propagate 1 more item to the output.
    builder.output.append_unencrypted_log_hashes(3);

    builder.validate_as_inner_call();
}

/**
 * private_call_requests
 */

#[test]
fn validate_propagated_from_private_call_private_call_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2);
    builder.output.append_private_call_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="source item does not reversed append to dest")]
fn validate_propagated_from_private_call_private_call_requests_not_reversed_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2);
    builder.output.append_private_call_requests(2);
    // Swap the call requests.
    let first_call_request = builder.output.private_call_requests.storage[0];
    builder.output.private_call_requests.storage[0] = builder.output.private_call_requests.storage[1];
    builder.output.private_call_requests.storage[1] = first_call_request;

    builder.validate_as_inner_call();
}

#[test]
fn validate_propagated_from_private_call_private_call_requests_with_previous_output_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.append_private_call_requests(5);
    let requests = builder.output.private_call_requests.storage;
    // The private_call_stack in the output will be: [requests[4], requests[3], requests[2], requests[1], requests[0]].

    // First 2 are propagated from the previous kernel.
    builder.previous_kernel.private_call_requests.push(requests[3]);
    builder.previous_kernel.private_call_requests.push(requests[4]);

    // Next 3 are propagated in reversed order from the private call.
    builder.private_call.private_call_requests.push(requests[0]);
    builder.private_call.private_call_requests.push(requests[1]);
    builder.private_call.private_call_requests.push(requests[2]);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="number of total items exceeds limit")]
fn validate_propagated_from_private_call_private_call_requests_with_previous_output_exceeds_max_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    // Make the previous array to be full, therefore no more items can be added.
    // Minus one because an extra request for the current call will be added in validate_as_inner_call().
    let max_len = builder.previous_kernel.private_call_requests.max_len() - 1;
    builder.previous_kernel.append_private_call_requests(max_len);
    builder.output.append_private_call_requests(max_len);
    // Add 2 item to the current call.
    builder.private_call.append_private_call_requests(2);

    builder.validate_as_inner_call();
}

/**
 * public_call_requests
 */

#[test]
fn validate_propagated_from_private_call_public_call_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2);
    builder.output.append_public_call_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with="output should be appended with empty items")]
fn validate_propagated_from_private_call_public_call_requests_output_one_more_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2);
    // Propagate 1 more item to the output.
    builder.output.append_public_call_requests(3);

    builder.validate_as_inner_call();
}
