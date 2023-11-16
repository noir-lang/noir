use dep::public_kernel_lib::{PublicKernelPrivatePreviousInputs};
use dep::types::{KernelCircuitPublicInputs};

unconstrained fn main(input: PublicKernelPrivatePreviousInputs) -> distinct pub KernelCircuitPublicInputs {
    input.public_kernel_private_previous()
}
