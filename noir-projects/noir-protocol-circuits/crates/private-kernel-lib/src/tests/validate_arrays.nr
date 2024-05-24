use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::{
    abis::{
    validation_requests::{KeyValidationRequest, KeyValidationRequestAndGenerator},
    caller_context::CallerContext, note_hash::NoteHash, nullifier::Nullifier,
    private_call_request::PrivateCallRequest, read_request::ReadRequest,
    log_hash::{LogHash, NoteLogHash, EncryptedLogHash}
},
    address::EthAddress, constants::GENERATOR_INDEX__IVSK_M, grumpkin_point::GrumpkinPoint,
    messaging::l2_to_l1_message::L2ToL1Message
};

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

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_key_validation_requests_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.key_validation_requests_and_generators.extend_from_array(
        [
        KeyValidationRequestAndGenerator::empty(),
        KeyValidationRequestAndGenerator {
            request: KeyValidationRequest { pk_m: GrumpkinPoint { x: 12, y: 34 }, sk_app: 5 },
            sk_app_generator: GENERATOR_INDEX__IVSK_M
        }
    ]
    );

    builder.validate();
}

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
fn validate_arrays_malformed_private_call_requests_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.private_call_requests.extend_from_array(
        [
        PrivateCallRequest::empty(), PrivateCallRequest {
            hash: 9123,
            start_side_effect_counter: 1,
            end_side_effect_counter: 2,
            caller_context: CallerContext::empty()
        }
    ]
    );

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
        EncryptedLogHash { value: 0, counter: 0, length: 0, randomness: 0 },
        EncryptedLogHash { value: 9123, counter: 1, length: 2, randomness: 4 }
    ]
    );

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_unencrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.unencrypted_logs_hashes.extend_from_array(
        [
        LogHash { value: 0, counter: 0, length: 0 },
        LogHash { value: 9123, counter: 1, length: 2 }
    ]
    );

    builder.validate();
}

#[test(should_fail_with="invalid array")]
fn validate_arrays_malformed_note_encrypted_logs_hashes() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.note_encrypted_logs_hashes.extend_from_array(
        [
        NoteLogHash { value: 0, counter: 0, length: 0, note_hash_counter: 0 },
        NoteLogHash { value: 9123, counter: 2, length: 2, note_hash_counter: 1 }
    ]
    );

    builder.validate();
}
