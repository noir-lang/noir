use dep::private_kernel_lib::PrivateKernelResetCircuitPrivateInputs;
use dep::types::PrivateKernelCircuitPublicInputs;

#[recursive]
fn main(input: PrivateKernelResetCircuitPrivateInputs) -> pub PrivateKernelCircuitPublicInputs {
    input.execute()
}
