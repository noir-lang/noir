use dep::private_kernel_lib::{PrivateKernelInputsInner, KernelCircuitPublicInputs};

fn main(input: PrivateKernelInputsInner) -> distinct pub KernelCircuitPublicInputs {
    input.native_private_kernel_circuit_inner()
}
