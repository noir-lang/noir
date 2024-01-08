use dep::private_kernel_lib::PrivateKernelInputsOrdering;
use dep::types::KernelCircuitPublicInputsFinal;

unconstrained fn main(input: PrivateKernelInputsOrdering) -> distinct pub KernelCircuitPublicInputsFinal {
    input.native_private_kernel_circuit_ordering()
}
