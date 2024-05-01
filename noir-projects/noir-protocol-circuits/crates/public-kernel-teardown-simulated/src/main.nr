use dep::public_kernel_lib::PublicKernelTeardownCircuitPrivateInputs;
use dep::types::PublicKernelCircuitPublicInputs;

unconstrained fn main(input: PublicKernelTeardownCircuitPrivateInputs) ->  pub PublicKernelCircuitPublicInputs {
    input.public_kernel_teardown()
}
