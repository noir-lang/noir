use dep::private_kernel_lib::PrivateKernelInnerCircuitPrivateInputs;
use dep::types::PrivateKernelCircuitPublicInputs;

fn main(input: PrivateKernelInnerCircuitPrivateInputs) -> distinct pub PrivateKernelCircuitPublicInputs {
    input.native_private_kernel_circuit_inner()
}
