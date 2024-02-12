use dep::public_kernel_lib::{PublicKernelAppLogicCircuitPrivateInputs};
use dep::types::{PublicKernelCircuitPublicInputs};

fn main(input: PublicKernelAppLogicCircuitPrivateInputs) -> distinct pub PublicKernelCircuitPublicInputs {
    input.public_kernel_app_logic()
}
