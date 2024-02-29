mod private_kernel_init;
mod private_kernel_inner;
mod private_kernel_tail;

use private_kernel_init::PrivateKernelInitCircuitPrivateInputs;
use private_kernel_inner::PrivateKernelInnerCircuitPrivateInputs;
use private_kernel_tail::PrivateKernelTailCircuitPrivateInputs;

// TODO: rename to be precise as to what its common to.
mod common;
mod read_request_reset;
mod nullifier_read_request_reset;
