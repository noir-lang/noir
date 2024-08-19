use dep::public_kernel_lib::PublicKernelAppLogicCircuitPrivateInputs;
use dep::types::PublicKernelCircuitPublicInputs;

unconstrained fn main(input: PublicKernelAppLogicCircuitPrivateInputs) -> pub PublicKernelCircuitPublicInputs {
    input.execute()
}
