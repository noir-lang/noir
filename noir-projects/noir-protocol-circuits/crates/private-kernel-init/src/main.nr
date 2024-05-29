use dep::private_kernel_lib::PrivateKernelInitCircuitPrivateInputs;
use dep::types::PrivateKernelCircuitPublicInputs;

#[recursive]
fn main(input: PrivateKernelInitCircuitPrivateInputs) -> pub PrivateKernelCircuitPublicInputs {
    input.execute()
}
