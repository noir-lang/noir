use crate::tests::private_kernel_circuit_output_validator_builder::PrivateKernelCircuitOutputValidatorBuilder;

#[test]
fn validate_propagated_from_previous_kernel_empty_data_succeeds() {
    let builder = PrivateKernelCircuitOutputValidatorBuilder::new();
    builder.validate_as_inner_call();
}

/**
 * constants
 */
#[test(should_fail_with = "mismatch constants")]
fn validate_propagated_from_previous_kernel_constants_mismatch_chain_id_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.tx_context.chain_id += 1;

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "mismatch constants")]
fn validate_propagated_from_previous_kernel_constants_mismatch_protocol_contract_tree_root_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.output.protocol_contract_tree_root += 1;

    builder.validate_as_inner_call();
}

/**
 * validation_requests
 */
#[test]
fn validate_propagated_from_previous_kernel_validation_requests_split_counter_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.validation_requests_split_counter = Option::some(123);
    builder.output.validation_requests_split_counter = Option::some(123);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "mismatch validation requests split counter")]
fn validate_propagated_from_previous_kernel_validation_requests_split_counter_mismatch_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.validation_requests_split_counter = Option::some(123);
    builder.output.validation_requests_split_counter = Option::some(4567);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "mismatch validation requests split counter")]
fn validate_propagated_from_previous_kernel_validation_requests_split_counter_unexpected_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.validation_requests_split_counter = Option::none();
    builder.output.validation_requests_split_counter = Option::some(123);

    builder.validate_as_inner_call();
}

/**
 * note_hash_read_requests
 */
#[test]
fn validate_propagated_from_previous_kernel_note_hash_read_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hash_read_requests(2);
    builder.output.append_note_hash_read_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_note_hash_read_requests_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hash_read_requests(2);
    // Propagate 1 less item to the output.
    builder.output.append_note_hash_read_requests(1);

    builder.validate_as_inner_call();
}

/**
 * nullifier_read_requests
 */
#[test]
fn validate_propagated_from_previous_kernel_nullifier_read_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifier_read_requests(2);
    builder.output.append_nullifier_read_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_nullifier_read_requests_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifier_read_requests(2);
    // Propagate 1 less item to the output.
    builder.output.append_nullifier_read_requests(1);

    builder.validate_as_inner_call();
}

/**
 * key_validation_requests
 */
#[test]
fn validate_propagated_from_previous_kernel_key_validation_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_key_validation_requests(2);
    builder.output.append_key_validation_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_key_validation_requests_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_key_validation_requests(2);
    // Propagate 1 less item to the output.
    builder.output.append_key_validation_requests(1);

    builder.validate_as_inner_call();
}

/**
 * note_hashes
 */
#[test]
fn validate_propagated_from_previous_kernel_note_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(2);
    builder.output.append_note_hashes(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_note_hashes_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_note_hashes(2);
    // Propagate 1 less item to the output.
    builder.output.append_note_hashes(1);

    builder.validate_as_inner_call();
}

/**
 * nullifiers
 */
#[test]
fn validate_propagated_from_previous_kernel_nullifiers_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(2);
    builder.output.append_nullifiers(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_nullifiers_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_nullifiers(2);
    // Propagate 1 less item to the output.
    builder.output.append_nullifiers(1);

    builder.validate_as_inner_call();
}

/**
 * l2_to_l1_msgs
 */
#[test]
fn validate_propagated_from_previous_kernel_l2_to_l1_msgs_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    builder.output.append_l2_to_l1_msgs(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_l2_to_l1_msgs_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_l2_to_l1_msgs(2);
    // Propagate 1 less item to the output.
    builder.output.append_l2_to_l1_msgs(1);

    builder.validate_as_inner_call();
}

/**
 * private_logs
 */
#[test]
fn validate_propagated_from_previous_kernel_private_logs_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_private_logs(2);
    builder.output.append_private_logs(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_private_logs_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_private_logs(2);
    // Propagate 1 less item to the output.
    builder.output.append_private_logs(1);

    builder.validate_as_inner_call();
}

/**
 * contract_class_log_hashes
 */
#[test]
fn validate_propagated_from_previous_kernel_contract_class_log_hashes_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.add_contract_class_log_hash(2, 1);
    builder.output.add_contract_class_log_hash(2, 1);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "output should be appended with empty items")]
fn validate_propagated_from_previous_kernel_contract_class_log_hashes_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    // Propagate 1 less item to the output.
    builder.output.add_contract_class_log_hash(2, 1);

    builder.validate_as_inner_call();
}

/**
 * private_call_requests
 */
#[test]
fn validate_propagated_from_previous_kernel_private_call_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_private_call_requests(2);
    builder.output.append_private_call_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_private_call_requests_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_private_call_requests(2);
    // Propagate 1 less item to the output.
    builder.output.append_private_call_requests(1);

    builder.validate_as_inner_call();
}

/**
 * public_call_requests
 */
#[test]
fn validate_propagated_from_previous_kernel_public_call_requests_succeeds() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_public_call_requests(2);
    builder.output.append_public_call_requests(2);

    builder.validate_as_inner_call();
}

#[test(should_fail_with = "source item does not prepend to dest")]
fn validate_propagated_from_previous_kernel_public_call_requests_less_than_fails() {
    let mut builder = PrivateKernelCircuitOutputValidatorBuilder::new();

    builder.previous_kernel.append_public_call_requests(2);
    // Propagate 1 less item to the output.
    builder.output.append_public_call_requests(1);

    builder.validate_as_inner_call();
}
