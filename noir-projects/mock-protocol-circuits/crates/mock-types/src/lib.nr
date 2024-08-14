global MAX_COMMITMENTS_PER_CALL = 2;
global MAX_COMMITMENTS_PER_TX = 4;
global MAX_COMMITMENT_READ_REQUESTS_PER_CALL = 2;
global MAX_COMMITMENT_READ_REQUESTS_PER_TX = 4;

struct TxRequest {
    number_of_calls: u32,
}

struct AppPublicInputs {
    commitments: [Field; MAX_COMMITMENTS_PER_CALL],
    read_requests: [Field; MAX_COMMITMENT_READ_REQUESTS_PER_CALL],
}

impl Default for AppPublicInputs {
    fn default() -> Self {
        Self {
            commitments: [0; MAX_COMMITMENTS_PER_CALL],
            read_requests: [0; MAX_COMMITMENT_READ_REQUESTS_PER_CALL],
        }
    }
}

struct PrivateKernelPublicInputs {
    remaining_calls: u32,
    commitments: [Field; MAX_COMMITMENTS_PER_TX],
    read_requests: [Field; MAX_COMMITMENT_READ_REQUESTS_PER_TX],
}

impl Default for PrivateKernelPublicInputs {
    fn default() -> Self {
        Self {
            remaining_calls: 0,
            commitments: [0; MAX_COMMITMENTS_PER_TX],
            read_requests: [0; MAX_COMMITMENT_READ_REQUESTS_PER_TX],
        }
    }
}

struct PrivateKernelPublicInputsBuilder {
    remaining_calls: u32,
    commitments: BoundedVec<Field, MAX_COMMITMENTS_PER_TX>,
    read_requests: BoundedVec<Field, MAX_COMMITMENT_READ_REQUESTS_PER_TX>,
}

impl PrivateKernelPublicInputsBuilder {
    fn from_tx(tx: TxRequest) -> Self {
        Self { remaining_calls: tx.number_of_calls, commitments: BoundedVec::new(), read_requests: BoundedVec::new() }
    }

    fn from_previous_kernel(prev_kernel_public_inputs: PrivateKernelPublicInputs) -> Self {
        let mut builder = PrivateKernelPublicInputsBuilder {
            remaining_calls: prev_kernel_public_inputs.remaining_calls,
            commitments: BoundedVec::new(),
            read_requests: BoundedVec::new()
        };
        for i in 0..MAX_COMMITMENTS_PER_TX {
            if prev_kernel_public_inputs.commitments[i] != 0 {
                builder.commitments.push(prev_kernel_public_inputs.commitments[i]);
            }
        }
        for i in 0..MAX_COMMITMENT_READ_REQUESTS_PER_TX {
            if prev_kernel_public_inputs.read_requests[i] != 0 {
                builder.read_requests.push(prev_kernel_public_inputs.read_requests[i]);
            }
        }
        builder
    }

    fn ingest_app_inputs(&mut self, app_inputs: AppPublicInputs) {
        for i in 0..MAX_COMMITMENTS_PER_CALL {
            if app_inputs.commitments[i] != 0 {
                self.commitments.push(app_inputs.commitments[i]);
            }
        }

        for i in 0..MAX_COMMITMENT_READ_REQUESTS_PER_CALL {
            if app_inputs.read_requests[i] != 0 {
                self.read_requests.push(app_inputs.read_requests[i]);
            }
        }

        self.remaining_calls -= 1;
    }

    fn finish(self) -> PrivateKernelPublicInputs {
        PrivateKernelPublicInputs {
            remaining_calls: self.remaining_calls,
            commitments: self.commitments.storage,
            read_requests: self.read_requests.storage
        }
    }
}

struct KernelPublicInputs {
    commitments: [Field; MAX_COMMITMENTS_PER_TX],
}

impl Default for KernelPublicInputs {
    fn default() -> Self {
        Self {
            commitments: [0; MAX_COMMITMENTS_PER_TX],
        }
    }
}

