mod meter_gas_used;
mod split_to_public;

use crate::components::private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer;
use dep::types::abis::kernel_circuit_public_inputs::{
    PrivateKernelCircuitPublicInputs, PrivateToPublicKernelCircuitPublicInputs,
};
pub use meter_gas_used::meter_gas_used;
pub use split_to_public::split_to_public;

pub struct TailToPublicOutputComposer {
    output_composer: PrivateKernelCircuitPublicInputsComposer,
}

impl TailToPublicOutputComposer {
    pub unconstrained fn new(previous_kernel: PrivateKernelCircuitPublicInputs) -> Self {
        let mut output_composer =
            PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(previous_kernel);
        output_composer.sort_ordered_values();

        TailToPublicOutputComposer { output_composer }
    }

    pub unconstrained fn finish(self) -> PrivateToPublicKernelCircuitPublicInputs {
        let source = self.output_composer.public_inputs;

        let (non_revertible_accumulated_data, revertible_accumulated_data) =
            split_to_public(source.end, source.min_revertible_side_effect_counter);

        let gas_used = meter_gas_used(
            non_revertible_accumulated_data,
            revertible_accumulated_data,
            source.public_teardown_call_request,
            source.constants.tx_context.gas_settings.teardown_gas_limits,
        );

        let mut output = PrivateToPublicKernelCircuitPublicInputs {
            constants: source.constants,
            rollup_validation_requests: source.validation_requests.for_rollup(),
            non_revertible_accumulated_data,
            revertible_accumulated_data,
            public_teardown_call_request: source.public_teardown_call_request,
            gas_used,
            fee_payer: source.fee_payer,
        };

        output
    }
}
