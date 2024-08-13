mod meter_gas_used;
mod split_to_public;
mod tail_to_public_output_composer;

use crate::components::tail_to_public_output_composer::TailToPublicOutputComposer;
use dep::types::{
    abis::kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs,
    tests::fixture_builder::FixtureBuilder
};

struct TailToPublicOutputComposerBuilder {
    previous_kernel: FixtureBuilder,
}

impl TailToPublicOutputComposerBuilder {
    pub fn new() -> Self {
        let mut previous_kernel = FixtureBuilder::new();
        previous_kernel.set_first_nullifier(); // Need the first nullifier to silo note hashes.
        TailToPublicOutputComposerBuilder { previous_kernel }
    }

    pub fn finish(self) -> PublicKernelCircuitPublicInputs {
        let previous_kernel = self.previous_kernel.to_private_kernel_circuit_public_inputs();
        let composer = TailToPublicOutputComposer::new(previous_kernel);
        composer.finish()
    }
}
