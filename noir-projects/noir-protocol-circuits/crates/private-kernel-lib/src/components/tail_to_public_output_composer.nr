mod meter_gas_used;
mod split_to_public;

use crate::components::{
    private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer,
    tail_to_public_output_composer::{
    meter_gas_used::{meter_gas_used_non_revertible, meter_gas_used_revertible},
    split_to_public::split_to_public
}
};
use dep::types::abis::{kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputs, PublicKernelCircuitPublicInputs}};

struct TailToPublicOutputComposer {
    output_composer: PrivateKernelCircuitPublicInputsComposer,
}

impl TailToPublicOutputComposer {
    pub fn new(previous_kernel: PrivateKernelCircuitPublicInputs) -> Self {
        let mut output_composer = PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(previous_kernel);
        output_composer.sort_and_silo();

        TailToPublicOutputComposer { output_composer }
    }

    pub fn finish(self) -> PublicKernelCircuitPublicInputs {
        let source = self.output_composer.public_inputs;
        let mut output = PublicKernelCircuitPublicInputs::empty();
        output.validation_requests = source.validation_requests.finish();
        output.constants = source.constants;
        output.public_teardown_call_stack[0] = source.public_teardown_call_request.expose_to_public();
        output.fee_payer = source.fee_payer;

        let mut (end_non_revertible, end) = split_to_public(source.end, source.min_revertible_side_effect_counter);
        end_non_revertible.gas_used = meter_gas_used_non_revertible(end_non_revertible);
        let teardown_gas = source.constants.tx_context.gas_settings.teardown_gas_limits;
        end.gas_used = meter_gas_used_revertible(end, teardown_gas);
        output.end_non_revertible = end_non_revertible;
        output.end = end;

        output
    }
}
