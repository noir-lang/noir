use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;

#[test]
fn validate_call_is_regular_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();
    builder.validate();
}

#[test]
fn validate_call_is_static_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();
    builder.validate();
}

#[test(should_fail_with = "note_hashes must be empty for static calls")]
fn validate_call_is_static_creating_note_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_note_hashes(1);

    builder.validate();
}

#[test(should_fail_with = "nullifiers must be empty for static calls")]
fn validate_call_is_static_creating_nullifiers_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_nullifiers(1);

    builder.validate();
}

#[test(should_fail_with = "l2_to_l1_msgs must be empty for static calls")]
fn validate_call_is_static_creating_l2_to_l1_msgs_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_l2_to_l1_msgs(1);

    builder.validate();
}

#[test(should_fail_with = "private_logs must be empty for static calls")]
fn validate_call_is_static_creating_private_logs_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_private_logs(1);

    builder.validate();
}

#[test(should_fail_with = "contract_class_logs_hashes must be empty for static calls")]
fn validate_call_is_static_creating_contract_class_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.add_contract_class_log_hash(1, 2);

    builder.validate();
}

#[test(should_fail_with = "only the class registerer may emit contract class logs")]
fn validate_call_is_from_class_registerer_fails() {
    // the default bulder address != REGISTERER_CONTRACT_ADDRESS
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_contract_class_log_hash(1, 2);

    builder.validate();
}
