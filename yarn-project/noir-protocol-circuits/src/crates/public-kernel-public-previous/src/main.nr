use dep::public_kernel_lib::{PublicKernelPublicPreviousInputs};
use dep::types::{KernelCircuitPublicInputs};

fn main(input: PublicKernelPublicPreviousInputs) -> distinct pub KernelCircuitPublicInputs {
    input.public_kernel_public_previous()
}
