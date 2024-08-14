use dep::mock_types::{TxRequest, PrivateKernelPublicInputs, PrivateKernelPublicInputsBuilder, AppPublicInputs};

fn main(tx: TxRequest, app_inputs: AppPublicInputs) -> pub PrivateKernelPublicInputs {
    let mut private_kernel_inputs = PrivateKernelPublicInputsBuilder::from_tx(tx);
    private_kernel_inputs.ingest_app_inputs(app_inputs);
    private_kernel_inputs.finish()
}

