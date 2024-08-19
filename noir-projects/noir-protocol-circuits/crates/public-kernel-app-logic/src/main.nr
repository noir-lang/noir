use dep::public_kernel_lib::PublicKernelAppLogicCircuitPrivateInputs;
use dep::types::PublicKernelCircuitPublicInputs;

#[recursive]
fn main(input: PublicKernelAppLogicCircuitPrivateInputs) -> pub PublicKernelCircuitPublicInputs {
    input.execute()
}
