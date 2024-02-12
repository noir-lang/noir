use dep::private_kernel_lib::PrivateKernelInnerCircuitPrivateInputs;
use dep::types::PrivateKernelInnerCircuitPublicInputs;

fn main(input: PrivateKernelInnerCircuitPrivateInputs) -> distinct pub PrivateKernelInnerCircuitPublicInputs {
    input.native_private_kernel_circuit_inner()
}
