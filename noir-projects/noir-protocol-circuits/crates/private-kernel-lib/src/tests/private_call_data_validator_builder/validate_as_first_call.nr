use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;

#[test]
fn validate_as_first_call_regular_call_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();
    builder.validate_as_first_call();
}

#[test(should_fail_with="Users cannot make a static call")]
fn validate_as_first_call_static_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_static_call();
    builder.validate_as_first_call();
}

#[test(should_fail_with="Users cannot make a delegatecall")]
fn validate_as_first_call_delegate_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();
    builder.validate_as_first_call();
}
