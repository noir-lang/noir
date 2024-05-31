use dep::private_kernel_lib::PrivateKernelEmptyPrivateInputs;
use dep::types::KernelCircuitPublicInputs;

#[recursive]
fn main(input: PrivateKernelEmptyPrivateInputs) -> pub KernelCircuitPublicInputs {
    input.execute()
}
