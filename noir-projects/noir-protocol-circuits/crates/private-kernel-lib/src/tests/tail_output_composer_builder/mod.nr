mod meter_gas_used;

use crate::components::tail_output_composer::TailOutputComposer;
use dep::types::{abis::kernel_circuit_public_inputs::KernelCircuitPublicInputs, tests::fixture_builder::FixtureBuilder};

struct TailOutputComposerBuilder {
    previous_kernel: FixtureBuilder,
}

impl TailOutputComposerBuilder {
    pub fn new() -> Self {
        let mut previous_kernel = FixtureBuilder::new();
        previous_kernel.set_first_nullifier(); // Need the first nullifier to silo note hashes.
        TailOutputComposerBuilder { previous_kernel }
    }

    pub fn with_siloed_data_builder(self) -> (Self, FixtureBuilder) {
        let mut siloed_data_builder = FixtureBuilder::new();
        siloed_data_builder.set_first_nullifier();
        (self, siloed_data_builder)
    }

    pub fn finish(self) -> KernelCircuitPublicInputs {
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        let composer = TailOutputComposer::new(previous_kernel);
        composer.finish()
    }
}

// TODO: Add tests.