use crate::tests::previous_kernel_validator_builder::PreviousKernelValidatorBuilder;
use types::address::AztecAddress;

#[test]
fn validate_common_for_private_tail_succeeds() {
    let builder = PreviousKernelValidatorBuilder::new_tail();
    builder.validate_for_private_tail();
}

#[test(should_fail_with = "Fee payer can't be empty")]
fn empty_fee_payer_for_private_tail_fails() {
    let mut builder = PreviousKernelValidatorBuilder::new_tail();

    builder.previous_kernel.set_fee_payer(AztecAddress::zero());

    builder.validate_for_private_tail();
}

#[test]
fn validate_common_for_private_tail_to_public_succeeds() {
    let builder = PreviousKernelValidatorBuilder::new_tail_to_public();
    builder.validate_for_private_tail_to_public();
}

#[test(should_fail_with = "Fee payer can't be empty")]
fn empty_fee_payer_for_private_to_public_tail_fails() {
    let mut builder = PreviousKernelValidatorBuilder::new_tail_to_public();

    builder.previous_kernel.set_fee_payer(AztecAddress::zero());

    builder.validate_for_private_tail_to_public();
}
