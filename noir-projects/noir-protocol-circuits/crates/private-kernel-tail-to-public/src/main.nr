use dep::private_kernel_lib::PrivateKernelTailToPublicCircuitPrivateInputs;
use dep::types::PublicKernelCircuitPublicInputs;

#[recursive]
fn main(input: PrivateKernelTailToPublicCircuitPrivateInputs) -> pub PublicKernelCircuitPublicInputs {
    input.execute()
}
