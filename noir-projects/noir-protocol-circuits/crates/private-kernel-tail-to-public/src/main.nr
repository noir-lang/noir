use dep::private_kernel_lib::PrivateKernelTailToPublicCircuitPrivateInputs;
use dep::types::PublicKernelCircuitPublicInputs;

fn main(input: PrivateKernelTailToPublicCircuitPrivateInputs) -> distinct pub PublicKernelCircuitPublicInputs {
    input.execute()
}
