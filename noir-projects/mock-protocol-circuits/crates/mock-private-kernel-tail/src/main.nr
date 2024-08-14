use dep::mock_types::{
    PrivateKernelPublicInputs, PrivateKernelPublicInputsBuilder, KernelPublicInputs,
    MAX_COMMITMENT_READ_REQUESTS_PER_TX
};

// The tail kernel finishes the client IVC chain exposing the final public inputs with no remaining calls or unfulfilled read requests.
fn main(prev_kernel_public_inputs: PrivateKernelPublicInputs) -> pub KernelPublicInputs {
    assert_eq(prev_kernel_public_inputs.remaining_calls, 0);
    for i in 0..MAX_COMMITMENT_READ_REQUESTS_PER_TX {
        assert_eq(prev_kernel_public_inputs.read_requests[i], 0);
    }

    KernelPublicInputs { commitments: prev_kernel_public_inputs.commitments }
}

