use dep::public_kernel_lib::PublicKernelSetupCircuitPrivateInputs;
use dep::types::PublicKernelCircuitPublicInputs;

#[recursive]
fn main(input: PublicKernelSetupCircuitPrivateInputs) -> pub PublicKernelCircuitPublicInputs {
    input.public_kernel_setup()
}
