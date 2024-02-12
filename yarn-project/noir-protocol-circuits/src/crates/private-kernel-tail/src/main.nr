use dep::private_kernel_lib::PrivateKernelTailCircuitPrivateInputs;
use dep::types::PrivateKernelTailCircuitPublicInputs;

fn main(input: PrivateKernelTailCircuitPrivateInputs) -> distinct pub PrivateKernelTailCircuitPublicInputs {
    input.native_private_kernel_circuit_ordering()
}
