mod kernel_circuit_public_inputs_composer;
mod private_kernel_init;
mod private_kernel_inner;
mod private_kernel_tail;
mod private_kernel_tail_to_public;

use private_kernel_init::PrivateKernelInitCircuitPrivateInputs;
use private_kernel_inner::PrivateKernelInnerCircuitPrivateInputs;
use private_kernel_tail::PrivateKernelTailCircuitPrivateInputs;
use private_kernel_tail_to_public::PrivateKernelTailToPublicCircuitPrivateInputs;

// TODO: rename to be precise as to what its common to.
mod common;
