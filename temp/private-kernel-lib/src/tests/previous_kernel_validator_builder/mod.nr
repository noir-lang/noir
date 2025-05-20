mod validate_common;
mod validate_no_transient_data;

use crate::components::previous_kernel_validator::PreviousKernelValidator;
use dep::types::{address::AztecAddress, tests::fixture_builder::FixtureBuilder, traits::FromField};

pub struct PreviousKernelValidatorBuilder {
    previous_kernel: FixtureBuilder,
}

impl PreviousKernelValidatorBuilder {
    pub fn new() -> Self {
        let mut previous_kernel = FixtureBuilder::new();
        previous_kernel.set_protocol_nullifier();
        previous_kernel.set_fee_payer(AztecAddress::from_field(345345));
        PreviousKernelValidatorBuilder { previous_kernel }
    }

    pub fn new_tail() -> Self {
        let mut builder = PreviousKernelValidatorBuilder::new();
        builder.previous_kernel.is_private_only = true;
        builder
    }

    pub fn new_tail_to_public() -> Self {
        let mut builder = PreviousKernelValidatorBuilder::new();
        builder.previous_kernel.append_public_call_requests(1);
        builder.previous_kernel.min_revertible_side_effect_counter = 11;
        builder
    }

    pub fn validate_for_private_tail(self) {
        let previous_kernel = self.previous_kernel.to_private_kernel_data();
        PreviousKernelValidator::new(previous_kernel).validate_for_private_tail();
    }

    pub fn validate_for_private_tail_to_public(self) {
        let previous_kernel = self.previous_kernel.to_private_kernel_data();
        PreviousKernelValidator::new(previous_kernel).validate_for_private_tail_to_public();
    }
}

// TODO: Add tests.
