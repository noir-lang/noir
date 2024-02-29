use dep::private_kernel_lib::PrivateKernelTailCircuitPrivateInputs;
use dep::types::PrivateKernelTailCircuitPublicInputs;

unconstrained fn main(input: PrivateKernelTailCircuitPrivateInputs) -> distinct pub PrivateKernelTailCircuitPublicInputs {
    input.native_private_kernel_circuit_tail()
}
