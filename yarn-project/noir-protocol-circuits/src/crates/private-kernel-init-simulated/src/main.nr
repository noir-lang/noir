use dep::private_kernel_lib::PrivateKernelInitCircuitPrivateInputs;
use dep::types::PrivateKernelInnerCircuitPublicInputs;

unconstrained fn main(input: PrivateKernelInitCircuitPrivateInputs) -> distinct pub PrivateKernelInnerCircuitPublicInputs {
    input.native_private_kernel_circuit_initial()
}
