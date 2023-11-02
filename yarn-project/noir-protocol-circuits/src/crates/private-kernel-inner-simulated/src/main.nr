use dep::private_kernel_lib::PrivateKernelInputsInner;
use dep::types::KernelCircuitPublicInputs;

unconstrained fn main(input: PrivateKernelInputsInner) -> distinct pub KernelCircuitPublicInputs {
    input.native_private_kernel_circuit_inner()
}
