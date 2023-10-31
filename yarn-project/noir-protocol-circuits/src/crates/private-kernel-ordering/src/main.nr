use dep::private_kernel_lib::{PrivateKernelInputsOrdering, KernelCircuitPublicInputsFinal};

fn main(input : PrivateKernelInputsOrdering) -> distinct pub KernelCircuitPublicInputsFinal {
    input.native_private_kernel_circuit_ordering()
}
