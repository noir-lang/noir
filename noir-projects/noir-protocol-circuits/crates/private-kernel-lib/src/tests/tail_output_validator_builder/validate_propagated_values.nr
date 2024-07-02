use crate::tests::tail_output_validator_builder::TailOutputValidatorBuilder;
use dep::types::address::AztecAddress;

#[test]
fn validate_propagated_values_constants_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.historical_header.total_fees = 123;
    builder.output.historical_header.total_fees = 123;

    builder.validate();
}

#[test(should_fail_with="mismatch constants")]
fn validate_propagated_values_constants_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.historical_header.total_fees = 123;
    // Tweak the value in the output.
    builder.output.historical_header.total_fees = 45;

    builder.validate();
}

#[test]
fn validate_propagated_values_max_block_number_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(123);
    builder.output.set_max_block_number(123);

    builder.validate();
}

#[test(should_fail_with="mismatch rollup_validation_requests")]
fn validate_propagated_values_max_block_number_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.set_max_block_number(123);
    // Tweak the value in the output.
    builder.output.set_max_block_number(45);

    builder.validate();
}

#[test]
fn validate_propagated_values_fee_payer_succeeds() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.set_fee_payer(AztecAddress::from_field(123));
    builder.output.set_fee_payer(AztecAddress::from_field(123));

    builder.validate();
}

#[test(should_fail_with="mismatch fee_payer")]
fn validate_propagated_values_fee_payer_mismatch_fails() {
    let mut builder = TailOutputValidatorBuilder::new();

    builder.previous_kernel.set_fee_payer(AztecAddress::from_field(123));
    // Tweak the value in the output.
    builder.output.set_fee_payer(AztecAddress::from_field(45));

    builder.validate();
}
