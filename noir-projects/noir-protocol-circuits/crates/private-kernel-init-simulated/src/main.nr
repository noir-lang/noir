use dep::private_kernel_lib::PrivateKernelInitCircuitPrivateInputs;
use dep::types::PrivateKernelCircuitPublicInputs;

unconstrained fn main(input: PrivateKernelInitCircuitPrivateInputs) -> pub PrivateKernelCircuitPublicInputs {
    input.execute()
}
