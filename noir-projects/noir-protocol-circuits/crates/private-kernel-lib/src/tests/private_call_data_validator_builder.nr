use crate::private_call_data_validator::PrivateCallDataValidator;
use dep::types::{
    abis::{
    call_request::CallRequest, caller_context::CallerContext, note_hash::NoteHash, nullifier::Nullifier,
    nullifier_key_validation_request::NullifierKeyValidationRequest, read_request::ReadRequest,
    side_effect::SideEffect
},
    address::{AztecAddress, EthAddress}, grumpkin_point::GrumpkinPoint,
    messaging::l2_to_l1_message::L2ToL1Message,
    tests::private_call_data_builder::PrivateCallDataBuilder, transaction::tx_request::TxRequest
};

struct PrivateCallDataValidatorBuilder {
    private_call: PrivateCallDataBuilder,
}

impl PrivateCallDataValidatorBuilder {
    pub fn new() -> Self {
        let private_call = PrivateCallDataBuilder::new();
        PrivateCallDataValidatorBuilder { private_call }
    }

    pub fn is_delegate_call(&mut self) -> Self {
        let _ = self.private_call.is_delegate_call();
        *self
    }

    pub fn is_static_call(&mut self) -> Self {
        let _ = self.private_call.is_static_call();
        *self
    }

    pub fn validate(self) {
        let private_call = self.private_call.finish();
        PrivateCallDataValidator::new(private_call).validate();
    }

    pub fn validate_against_tx_request(self, request: TxRequest) {
        let private_call = self.private_call.finish();
        PrivateCallDataValidator::new(private_call).validate_against_tx_request(request);
    }

    pub fn validate_against_call_request(self, request: CallRequest) {
        let private_call = self.private_call.finish();
        PrivateCallDataValidator::new(private_call).validate_against_call_request(request);
    }
}

/**
 * validate_arrays
 */

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_note_hash_read_requests_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.note_hash_read_requests.extend_from_array(
        [
        ReadRequest::empty(),
        ReadRequest { value: 9123, counter: 1 }
    ]
    );

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_nullifier_read_requests_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.nullifier_read_requests.extend_from_array(
        [
        ReadRequest::empty(),
        ReadRequest { value: 9123, counter: 1 }
    ]
    );

    builder.validate();
}

// Enable this test if MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL is greater than 1.
// #[test(should_fail_with="invalid array")]
// fn validate_arrays_malformed_nullifier_key_validation_requests_fails() {
//     let mut builder = PrivateCallDataValidatorBuilder::new();

//     builder.private_call.public_inputs.nullifier_key_validation_requests.extend_from_array(
//         [
//         NullifierKeyValidationRequest::empty(),
//         NullifierKeyValidationRequest { master_nullifier_public_key: GrumpkinPoint { x: 12, y: 34 }, app_nullifier_secret_key: 5 }
//     ]
//     );

//     builder.validate();
// }

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_note_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.new_note_hashes.extend_from_array(
        [
        NoteHash::empty(),
        NoteHash { value: 9123, counter: 1 }
    ]
    );

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_nullifiers_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.new_nullifiers.extend_from_array(
        [
        Nullifier::empty(),
        Nullifier { value: 9123, note_hash: 0, counter: 1 }
    ]
    );

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_l2_to_l1_msgs_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.new_l2_to_l1_msgs.extend_from_array(
        [
        L2ToL1Message::empty(),
        L2ToL1Message { recipient: EthAddress::from_field(6), content: 9123, counter: 0 }
    ]
    );

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_private_call_stack_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.private_call_stack_hashes.extend_from_array([0, 9123]);

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_public_call_stack_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.public_call_stack_hashes.extend_from_array([0, 9123]);

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_encrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.encrypted_logs_hashes.extend_from_array(
        [
        SideEffect { value: 0, counter: 0 },
        SideEffect { value: 9123, counter: 1 }
    ]
    );

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_unencrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.unencrypted_logs_hashes.extend_from_array(
        [
        SideEffect { value: 0, counter: 0 },
        SideEffect { value: 9123, counter: 1 }
    ]
    );

    builder.validate();
}

/**
 * validate_contract_address
 */

#[test(should_fail_with="contract address cannot be zero")]
fn validate_contract_address_zero_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Set (storage) contract_address to 0
    builder.private_call.contract_address = AztecAddress::zero();
    builder.private_call.public_inputs.call_context.storage_contract_address = AztecAddress::zero();

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_function_leaf_index_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Set the leaf index of the function leaf to a wrong value (the correct value + 1).
    let leaf_index = builder.private_call.function_leaf_membership_witness.leaf_index;
    builder.private_call.function_leaf_membership_witness.leaf_index = leaf_index + 1;

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_function_leaf_sibling_path_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Set the first value of the sibling path to a wrong value (the correct value + 1).
    let sibling_path_0 = builder.private_call.function_leaf_membership_witness.sibling_path[0];
    builder.private_call.function_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_contract_class_preimage_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.contract_class_artifact_hash = builder.private_call.contract_class_artifact_hash + 1;

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_partial_address_preimage_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.salted_initialization_hash.inner = builder.private_call.salted_initialization_hash.inner + 1;

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_address_preimage_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_keys_hash.inner = builder.private_call.public_keys_hash.inner + 1;

    builder.validate();
}

/**
 * validate_call
 */

#[test]
fn validate_call_is_regular_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();
    builder.validate();
}

#[test(should_fail_with="call stack storage address does not match expected contract address")]
fn validate_call_is_regular_mismatch_storage_contract_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Change the storage contract address to be a different value.
    builder.private_call.public_inputs.call_context.storage_contract_address.inner += 1;

    builder.validate();
}

#[test]
fn validate_call_is_delegate_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();
    builder.validate();
}

#[test(should_fail_with="current contract address must not match storage contract address for delegate calls")]
fn validate_call_is_delegate_call_from_same_contract_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    // Change the caller's storage contract address to be the same as the contract address.
    builder.private_call.public_inputs.call_context.storage_contract_address = builder.private_call.contract_address;

    builder.validate();
}

#[test]
fn validate_call_is_static_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();
    builder.validate();
}

#[test(should_fail_with="call stack storage address does not match expected contract address")]
fn validate_call_is_static_mismatch_storage_contract_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    // Change the storage contract address to be a different value.
    builder.private_call.public_inputs.call_context.storage_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="new_note_hashes must be empty for static calls")]
fn validate_call_is_static_creating_note_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.new_note_hashes.push(NoteHash { value: 1, counter: 0 });

    builder.validate();
}

#[test(should_fail_with="new_nullifiers must be empty for static calls")]
fn validate_call_is_static_creating_nullifiers_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.new_nullifiers.push(Nullifier { value: 1, counter: 0, note_hash: 0 });

    builder.validate();
}

#[test(should_fail_with="new_l2_to_l1_msgs must be empty for static calls")]
fn validate_call_is_static_creating_l2_to_l1_msgs_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.new_l2_to_l1_msgs.push(L2ToL1Message { recipient: EthAddress::from_field(6), content: 9123, counter: 0 });

    builder.validate();
}

#[test(should_fail_with="encrypted_logs_hashes must be empty for static calls")]
fn validate_call_is_static_creating_encrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.encrypted_logs_hashes.push(SideEffect { value: 9123, counter: 1 });

    builder.validate();
}

#[test(should_fail_with="unencrypted_logs_hashes must be empty for static calls")]
fn validate_call_is_static_creating_unencrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.unencrypted_logs_hashes.push(SideEffect { value: 9123, counter: 1 });

    builder.validate();
}

/**
 * validate_private_call_requests
 */

#[test]
fn validate_private_call_requests_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2, false);

    builder.validate();
}

#[test]
fn validate_private_call_requests_delegate_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2, true);

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_private_call_requests_incorrect_hash_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2, false);
    let mut call_request = builder.private_call.private_call_stack.pop();
    // Change the hash to be a different value.
    call_request.hash += 1;
    builder.private_call.private_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_private_call_requests_incorrect_caller_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1, false);
    let mut call_request = builder.private_call.private_call_stack.pop();
    // Change the caller contract address to be a different value.
    call_request.caller_contract_address.inner += 1;
    builder.private_call.private_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_private_call_requests_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1, true);
    let mut call_request = builder.private_call.private_call_stack.pop();
    // Change the storage contract to be a different value.
    call_request.caller_context.storage_contract_address.inner += 1;
    builder.private_call.private_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_private_call_requests_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1, true);
    let mut call_request = builder.private_call.private_call_stack.pop();
    // Change the msg_sender to be a different value.
    call_request.caller_context.msg_sender.inner += 1;
    builder.private_call.private_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="call requests length does not match the expected length")]
fn validate_private_call_requests_fewer_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2, false);
    // Remove one call stack item hash.
    let _ = builder.private_call.public_inputs.private_call_stack_hashes.pop();

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_private_call_requests_more_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2, false);
    // Add one random call stack item hash.
    builder.private_call.public_inputs.private_call_stack_hashes.push(9123);

    builder.validate();
}

/**
 * validate_public_call_requests
 */

#[test]
fn validate_public_call_requests_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, false);

    builder.validate();
}

#[test]
fn validate_public_call_requests_delegate_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, true);

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_public_call_requests_incorrect_hash_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, false);
    let mut call_request = builder.private_call.public_call_stack.pop();
    // Change the hash to be a different value.
    call_request.hash += 1;
    builder.private_call.public_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_public_call_requests_incorrect_caller_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1, false);
    let mut call_request = builder.private_call.public_call_stack.pop();
    // Change the caller contract address to be a different value.
    call_request.caller_contract_address.inner += 1;
    builder.private_call.public_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_public_call_requests_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1, true);
    let mut call_request = builder.private_call.public_call_stack.pop();
    // Change the storage contract to be a different value.
    call_request.caller_context.storage_contract_address.inner += 1;
    builder.private_call.public_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_public_call_requests_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1, true);
    let mut call_request = builder.private_call.public_call_stack.pop();
    // Change the msg_sender to be a different value.
    call_request.caller_context.msg_sender.inner += 1;
    builder.private_call.public_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="call requests length does not match the expected length")]
fn validate_public_call_requests_fewer_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, false);
    // Remove one call stack item hash.
    let _ = builder.private_call.public_inputs.public_call_stack_hashes.pop();

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_public_call_requests_more_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, false);
    // Add one random call stack item hash.
    builder.private_call.public_inputs.public_call_stack_hashes.push(9123);

    builder.validate();
}

/**
 * validate_teardown_call_request
 */

#[test]
fn validate_teardown_call_request_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teaddown_call_request(false);

    builder.validate();
}

#[test]
fn validate_teardown_call_request_delegate_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teaddown_call_request(true);

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_teardown_call_request_incorrect_hash_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teaddown_call_request(true);
    // Change the hash to be a different value.
    builder.private_call.public_teardown_call_request.hash += 1;

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_teardown_call_request_incorrect_caller_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teaddown_call_request(true);
    // Change the caller contract address to be a different value.
    builder.private_call.public_teardown_call_request.caller_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_teardown_call_request_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teaddown_call_request(true);
    // Change the storage contract to be a different value.
    builder.private_call.public_teardown_call_request.caller_context.storage_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="invalid caller")]
fn validate_teardown_call_request_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teaddown_call_request(true);
    // Change the msg_sender to be a different value.
    builder.private_call.public_teardown_call_request.caller_context.msg_sender.inner += 1;

    builder.validate();
}

#[test(should_fail_with="call requests length does not match the expected length")]
fn validate_teardown_call_request_fewer_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teaddown_call_request(true);
    // Remove the call stack item hash.
    builder.private_call.public_inputs.public_teardown_function_hash = 0;

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_teardown_call_request_more_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teaddown_call_request(true);
    // Remove the call request.
    builder.private_call.public_teardown_call_request = CallRequest::empty();

    builder.validate();
}

/**
 * validate_against_tx_request
 */

#[test]
fn validate_against_tx_request_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let request = builder.private_call.build_tx_request();

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="origin address does not match call stack items contract address")]
fn validate_against_tx_request_mismatch_contract_address_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the origin to be a different value.
    request.origin.inner += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="tx_request function_data must match call_stack_item function_data")]
fn validate_against_tx_request_mismatch_function_data_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the function selector to be a different value.
    request.function_data.selector.inner += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="noir function args passed to tx_request must match args in the call_stack_item")]
fn validate_against_tx_request_mismatch_args_hash_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the args hash to be a different value.
    request.args_hash += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="tx_context in tx_request must match tx_context in call_stack_item")]
fn validate_against_tx_request_mismatch_chain_id_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the chain id to be a different value.
    request.tx_context.chain_id += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="tx_context in tx_request must match tx_context in call_stack_item")]
fn validate_against_tx_request_mismatch_version_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the version to be a different value.
    request.tx_context.version += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="Users cannot make a static call")]
fn validate_against_tx_request_static_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_static_call();
    let request = builder.private_call.build_tx_request();
    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="Users cannot make a delegatecall")]
fn validate_against_tx_request_delegate_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();
    let request = builder.private_call.build_tx_request();
    builder.validate_against_tx_request(request);
}

/**
 * validate_against_call_request
 */

#[test]
fn validate_against_call_request_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let request = builder.private_call.build_call_request();

    builder.validate_against_call_request(request);
}

#[test]
fn validate_against_call_request_delegate_call_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let request = builder.private_call.build_call_request();

    builder.validate_against_call_request(request);
}

#[test]
fn validate_against_call_request_static_call_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    let request = builder.private_call.build_call_request();

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="calculated private_call_hash does not match provided private_call_hash at the top of the call stack")]
fn validate_against_call_request_mismatch_hash_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_call_request();
    // Tweak the hash to be a different value.
    request.hash += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="caller context cannot be empty for delegate calls")]
fn validate_against_call_request_empty_caller_context_for_delegate_calls_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let mut request = builder.private_call.build_call_request();
    request.caller_context = CallerContext::empty();

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="call stack msg_sender does not match expected msg_sender for delegate calls")]
fn validate_against_call_request_incorrect_msg_sender_for_delegate_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let mut request = builder.private_call.build_call_request();
    // Tweak the msg_sender to be a different value.
    request.caller_context.msg_sender.inner += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="call stack storage address does not match expected contract address for delegate calls")]
fn validate_against_call_request_incorrect_storage_contract_address_for_delegate_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let mut request = builder.private_call.build_call_request();
    // Tweak the storage contract address to be a different value.
    request.caller_context.storage_contract_address.inner += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="call stack msg_sender does not match caller contract address")]
fn validate_against_call_request_incorrect_msg_sender_for_regular_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_call_request();
    // Tweak the caller's contract address to be a different value.
    request.caller_contract_address.inner += 1;

    builder.validate_against_call_request(request);
}
