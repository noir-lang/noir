mod validate_accumulated_values;
mod validate_empty_values;
mod validate_propagated_sorted_siloed_values;
mod validate_propagated_values;

use crate::components::tail_output_validator::TailOutputValidator;
use dep::types::tests::fixture_builder::FixtureBuilder;

struct TailOutputValidatorBuilder {
    output: FixtureBuilder,
    previous_kernel: FixtureBuilder
}

impl TailOutputValidatorBuilder {
    pub fn new() -> Self {
        let mut output = FixtureBuilder::new();
        let mut previous_kernel = FixtureBuilder::new();
        output.set_first_nullifier();
        previous_kernel.set_first_nullifier();
        TailOutputValidatorBuilder { output, previous_kernel }
    }

    pub fn validate(self) {
        let output = self.output.to_kernel_circuit_public_inputs();
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        TailOutputValidator::new(output, previous_kernel).validate();
    }
}
