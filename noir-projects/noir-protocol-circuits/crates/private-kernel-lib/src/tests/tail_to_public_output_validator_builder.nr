use crate::components::tail_to_public_output_validator::TailToPublicOutputValidator;
use dep::types::tests::fixture_builder::FixtureBuilder;

struct TailToPublicOutputValidatorBuilder {
    output: FixtureBuilder,
    previous_kernel: FixtureBuilder
}

impl TailToPublicOutputValidatorBuilder {
    pub fn new() -> Self {
        let mut output = FixtureBuilder::new();
        let mut previous_kernel = FixtureBuilder::new();
        output.set_first_nullifier();
        previous_kernel.set_first_nullifier();
        TailToPublicOutputValidatorBuilder { output, previous_kernel }
    }

    pub fn validate(self) {
        // TODO: Split the data using min_revertible_side_effect_counter in FixtureBuilder.
        let revertible = true;
        let output = self.output.to_public_kernel_circuit_public_inputs(revertible);
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        TailToPublicOutputValidator::new(output, previous_kernel).validate();
    }
}

// TODO: Add tests.