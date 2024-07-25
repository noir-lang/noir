use dep::types::{
    abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs, utils::arrays::array_length,
    traits::is_empty
};

struct PreviousKernelValidator {
    previous_kernel: PrivateKernelCircuitPublicInputs
}

impl PreviousKernelValidator {
    pub fn new(previous_kernel: PrivateKernelCircuitPublicInputs) -> Self {
        PreviousKernelValidator { previous_kernel }
    }

    pub fn validate_for_private_tail(self) {
        self.validate_common();
        self.validate_empty_public_call_requests();
    }

    pub fn validate_for_private_tail_to_public(self) {
        self.validate_common();
        self.validate_non_empty_public_call_requests();
    }

    fn validate_common(self) {
        self.validate_empty_private_call_stack();
        self.verify_empty_validation_requests();
        self.verify_no_transient_data();
    }

    fn validate_empty_private_call_stack(self) {
        assert_eq(
            array_length(self.previous_kernel.end.private_call_stack), 0, "Private call stack must be empty when executing the tail circuit"
        );
    }

    fn validate_empty_public_call_requests(self) {
        assert_eq(
            array_length(self.previous_kernel.end.public_call_requests), 0, "Public call stack must be empty when executing the tail circuit"
        );
        assert(
            is_empty(self.previous_kernel.public_teardown_call_request), "Public teardown call request must be empty when executing the tail circuit"
        );
    }

    fn validate_non_empty_public_call_requests(self) {
        let len = array_length(self.previous_kernel.end.public_call_requests);
        assert(
            (len != 0) | !is_empty(self.previous_kernel.public_teardown_call_request), "Must have public calls when exporting public kernel data from the tail circuit"
        );
    }

    fn verify_empty_validation_requests(self) {
        assert_eq(
            array_length(self.previous_kernel.validation_requests.note_hash_read_requests), 0, "Non empty note hash read requests"
        );
        assert_eq(
            array_length(self.previous_kernel.validation_requests.nullifier_read_requests), 0, "Non empty nullifier read requests"
        );
        assert_eq(
            array_length(self.previous_kernel.validation_requests.scoped_key_validation_requests_and_generators), 0, "Non empty key validation requests"
        );
    }

    fn verify_no_transient_data(self) {
        // Currently all the transient nullifiers must be cleared in the reset circuits.
        // When a transient nullifier is removed, the linked note hash and all the associated note logs will also be removed.
        for nullifier in self.previous_kernel.end.nullifiers {
            assert_eq(nullifier.nullified_note_hash(), 0, "Unresolved transient nullifier");
        }
    }
}
