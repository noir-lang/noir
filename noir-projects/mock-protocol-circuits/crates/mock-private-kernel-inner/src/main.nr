use dep::mock_types::{PrivateKernelPublicInputs, PrivateKernelPublicInputsBuilder, AppPublicInputs};

fn main(
    prev_kernel_public_inputs: PrivateKernelPublicInputs,
    app_inputs: AppPublicInputs
) -> pub PrivateKernelPublicInputs {
    let mut private_kernel_inputs = PrivateKernelPublicInputsBuilder::from_previous_kernel(prev_kernel_public_inputs);
    private_kernel_inputs.ingest_app_inputs(app_inputs);
    private_kernel_inputs.finish()
}

