mod validate_gas_used;
mod validate_propagated_sorted_values;
mod validate_propagated_values;

use crate::components::{
    tail_output_composer::meter_gas_used,
    tail_output_validator::{
        tail_output_hints::{generate_tail_output_hints, TailOutputHints},
        TailOutputValidator,
    },
};
use dep::types::{
    abis::{
        gas::Gas, gas_fees::GasFees, gas_settings::GasSettings,
        kernel_circuit_public_inputs::PrivateToRollupKernelCircuitPublicInputs,
    },
    constants::{DEFAULT_GAS_LIMIT, DEFAULT_TEARDOWN_GAS_LIMIT},
    tests::fixture_builder::FixtureBuilder,
};

pub struct TailOutputValidatorBuilder {
    output: FixtureBuilder,
    previous_kernel: FixtureBuilder,
}

impl TailOutputValidatorBuilder {
    pub fn new() -> Self {
        let gas_settings = GasSettings::new(
            Gas::new(DEFAULT_GAS_LIMIT, DEFAULT_GAS_LIMIT),
            Gas::new(DEFAULT_TEARDOWN_GAS_LIMIT, DEFAULT_TEARDOWN_GAS_LIMIT),
            GasFees::new(10, 10),
            GasFees::new(3, 3),
        );

        let mut output = FixtureBuilder::new();
        let mut previous_kernel = FixtureBuilder::new();
        output.tx_context.gas_settings = gas_settings;
        previous_kernel.tx_context.gas_settings = gas_settings;
        output.set_protocol_nullifier();
        previous_kernel.set_protocol_nullifier();
        TailOutputValidatorBuilder { output, previous_kernel }
    }

    pub fn get_hints(self) -> TailOutputHints {
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        // Safety: This is only used in tests.
        unsafe {
            generate_tail_output_hints(previous_kernel)
        }
    }

    pub fn export_output(self) -> PrivateToRollupKernelCircuitPublicInputs {
        let mut output = self.output.to_private_to_rollup_kernel_circuit_public_inputs();
        output.gas_used = meter_gas_used(output.end);
        output
    }

    pub fn validate(self) {
        let output = self.export_output();
        self.validate_with_output(output);
    }

    pub fn validate_with_output(self, output: PrivateToRollupKernelCircuitPublicInputs) {
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        TailOutputValidator::new(output, previous_kernel).validate();
    }

    pub fn validate_with_hints(self, hints: TailOutputHints) {
        let output = self.export_output();
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        TailOutputValidator::new_with_hints(output, previous_kernel, hints).validate();
    }
}
