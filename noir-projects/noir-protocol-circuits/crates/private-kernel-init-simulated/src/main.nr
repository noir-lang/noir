use dep::private_kernel_lib::PrivateKernelInitCircuitPrivateInputs;
use dep::types::PrivateKernelCircuitPublicInputs;

unconstrained fn main(input: PrivateKernelInitCircuitPrivateInputs) -> distinct pub PrivateKernelCircuitPublicInputs {
    input.native_private_kernel_circuit_initial()
}
