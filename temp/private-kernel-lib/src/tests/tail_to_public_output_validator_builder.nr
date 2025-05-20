use crate::components::tail_to_public_output_validator::TailToPublicOutputValidator;
use dep::types::tests::fixture_builder::FixtureBuilder;

pub(crate) struct TailToPublicOutputValidatorBuilder {
    pub(crate) output: FixtureBuilder,
    pub(crate) previous_kernel: FixtureBuilder,
}

impl TailToPublicOutputValidatorBuilder {
    pub(crate) fn new() -> Self {
        let mut output = FixtureBuilder::new();
        let mut previous_kernel = FixtureBuilder::new();
        output.set_protocol_nullifier();
        previous_kernel.set_protocol_nullifier();
        TailToPublicOutputValidatorBuilder { output, previous_kernel }
    }

    pub(crate) fn validate(self) {
        // TODO: Split the data using min_revertible_side_effect_counter in FixtureBuilder.
        let revertible = true;
        let output = self.output.to_private_to_public_kernel_circuit_public_inputs(revertible);
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        TailToPublicOutputValidator::new(output, previous_kernel).validate();
    }
}

// TODO: Add tests.
