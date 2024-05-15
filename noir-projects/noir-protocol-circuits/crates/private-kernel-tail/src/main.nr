use dep::private_kernel_lib::PrivateKernelTailCircuitPrivateInputs;
use dep::types::KernelCircuitPublicInputs;

#[recursive]
fn main(input: PrivateKernelTailCircuitPrivateInputs) -> pub KernelCircuitPublicInputs {
    input.execute()
}
