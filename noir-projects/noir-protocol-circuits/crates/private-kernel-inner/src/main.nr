use dep::private_kernel_lib::PrivateKernelInnerCircuitPrivateInputs;
use dep::types::PrivateKernelCircuitPublicInputs;

#[recursive]
fn main(input: PrivateKernelInnerCircuitPrivateInputs) -> pub PrivateKernelCircuitPublicInputs {
    input.native_private_kernel_circuit_inner()
}
