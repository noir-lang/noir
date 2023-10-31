use dep::private_kernel_lib::{ PrivateKernelInputsInit, KernelCircuitPublicInputs };

unconstrained fn main(input: PrivateKernelInputsInit) -> distinct pub KernelCircuitPublicInputs {
    input.native_private_kernel_circuit_initial()
}
