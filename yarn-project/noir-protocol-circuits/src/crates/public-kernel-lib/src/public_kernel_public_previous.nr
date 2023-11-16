use crate::abis::public_call_data::PublicCallData;
use dep::types::abis::previous_kernel_data::PreviousKernelData;
use dep::types::KernelCircuitPublicInputs;
use dep::types::abis::kernel_circuit_public_inputs::KernelCircuitPublicInputsBuilder;
use crate::common;
use dep::std::unsafe;


struct PublicKernelPublicPreviousInputs {
    previous_kernel: PreviousKernelData,
    public_call: PublicCallData,
}

impl PublicKernelPublicPreviousInputs {

// this is the only difference between the two PublicKernels' logic:
    fn validate_inputs(self) {
        let previous_call_is_private = self.previous_kernel.public_inputs.is_private;
        assert(previous_call_is_private == false,
            "Previous kernel must be public when in this public kernel version");
    }


    fn public_kernel_public_previous(self) -> KernelCircuitPublicInputs {
        // construct the circuit outputs
        let mut public_inputs: KernelCircuitPublicInputsBuilder = unsafe::zeroed();

        // initialise the end state with our provided previous kernel state
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        // validate the inputs common to all invocation circumstances
        common::validate_inputs(self.public_call);

        // validate the inputs unique to having a previous public kernel
        self.validate_inputs();

        // validate the kernel execution common to all invocation circumstances
        common::validate_kernel_execution(self.public_call);

        // validate our public call hash
        common::validate_this_public_call_hash(self.public_call, &mut public_inputs);

        common::update_public_end_values(self.public_call,&mut public_inputs);

        common::accumulate_unencrypted_logs(self.public_call, self.previous_kernel,&mut public_inputs);

        public_inputs.finish()
    }
}