use dep::public_kernel_lib::PublicKernelTailCircuitPrivateInputs;
use dep::types::KernelCircuitPublicInputs;

fn main(input: PublicKernelTailCircuitPrivateInputs) -> distinct pub KernelCircuitPublicInputs {
    input.public_kernel_tail()
}
