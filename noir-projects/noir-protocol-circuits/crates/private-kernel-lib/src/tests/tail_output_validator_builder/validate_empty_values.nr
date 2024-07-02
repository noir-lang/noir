use crate::tests::tail_output_validator_builder::TailOutputValidatorBuilder;

#[test]
fn validate_empty_values_succeeds() {
    let builder = TailOutputValidatorBuilder::new();
    builder.validate();
}

#[test(should_fail_with="start_state must be empty")]
fn validate_empty_values_non_empty_start_state_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.output.start_state.public_data_tree.root = 123;

    builder.validate();
}

#[test(should_fail_with="revert_code must be empty")]
fn validate_empty_values_non_empty_revert_code_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.output.revert_code = 1;

    builder.validate();
}

