use dep::public_kernel_lib::PublicKernelTailCircuitPrivateInputs;
use dep::types::PublicKernelCircuitPublicInputs;

unconstrained fn main(input: PublicKernelTailCircuitPrivateInputs) -> distinct pub PublicKernelCircuitPublicInputs {
    input.public_kernel_tail()
}
