use dep::private_kernel_lib::PrivateKernelTailCircuitPrivateInputs;
use dep::types::KernelCircuitPublicInputs;

unconstrained fn main(input: PrivateKernelTailCircuitPrivateInputs) -> distinct pub KernelCircuitPublicInputs {
    input.native_private_kernel_circuit_tail()
}
