use crate::{
    reset::{
    non_existent_read_request::reset_non_existent_read_requests,
    mutable_data_read_request::reset_mutable_data_read_requests, read_request::reset_read_requests
},
    nullifier_read_request_reset::NullifierReadRequestHints,
    nullifier_non_existent_read_request_reset::NullifierNonExistentReadRequestHints,
    public_data_read_request_reset::PublicDataReadRequestHints
};
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, nullifier::Nullifier,
    public_data_update_request::PublicDataUpdateRequest, validation_requests::ValidationRequests
},
    data::public_data_hint::PublicDataHint,
    constants::{MAX_NULLIFIERS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX},
    hash::compute_siloed_nullifier, traits::is_empty,
    utils::arrays::{array_merge, array_to_bounded_vec, assert_sorted_array}
};

struct PublicValidationRequestProcessor<N> {
    validation_requests: ValidationRequests,
    pending_nullifiers: [Nullifier; MAX_NULLIFIERS_PER_TX],
    pending_public_data_writes: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    nullifier_read_request_hints: NullifierReadRequestHints<MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifier_non_existent_read_request_hints: NullifierNonExistentReadRequestHints,
    nullifier_tree_root: Field,
    public_data_read_request_hints: PublicDataReadRequestHints,
    public_data_hints: [PublicDataHint; N],
}

impl<N> PublicValidationRequestProcessor<N> {
    pub fn new(
        public_inputs: PublicKernelCircuitPublicInputs,
        nullifier_read_request_hints: NullifierReadRequestHints<MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
        nullifier_non_existent_read_request_hints: NullifierNonExistentReadRequestHints,
        nullifier_tree_root: Field,
        public_data_read_request_hints: PublicDataReadRequestHints,
        public_data_hints: [PublicDataHint; N]
    ) -> Self {
        let end_non_revertible = public_inputs.end_non_revertible;
        let end = public_inputs.end;

        let pending_nullifiers = array_merge(end_non_revertible.nullifiers, end.nullifiers);

        let pending_public_data_writes = array_merge(
            end_non_revertible.public_data_update_requests,
            end.public_data_update_requests
        );

        PublicValidationRequestProcessor {
            validation_requests: public_inputs.validation_requests,
            pending_nullifiers,
            pending_public_data_writes,
            nullifier_read_request_hints,
            nullifier_non_existent_read_request_hints,
            nullifier_tree_root,
            public_data_read_request_hints,
            public_data_hints
        }
    }

    pub fn validate(self) {
        self.validate_nullifier_read_requests();
        self.validate_nullifier_non_existent_read_requests();
        self.validate_public_data_read_requests();
    }

    fn validate_nullifier_read_requests(self) {
        let requests = self.validation_requests.nullifier_read_requests;
        let hints = self.nullifier_read_request_hints;
        let unverified_nullifier_read_requests = reset_read_requests(
            requests,
            self.pending_nullifiers,
            hints.read_request_statuses,
            hints.pending_read_hints,
            hints.settled_read_hints,
            self.nullifier_tree_root
        );
        assert(
            unverified_nullifier_read_requests.len() == 0, "All nullifier read requests must be verified"
        );
    }

    fn validate_nullifier_non_existent_read_requests(self) {
        // The values of the read requests here need to be siloed.
        // Notice that it's not the case for regular read requests, which can be run between two kernel iterations, and will to be verified against unsiloed pending values.
        let mut read_requests = self.validation_requests.nullifier_non_existent_read_requests;
        for i in 0..read_requests.len() {
            let read_request = read_requests[i];
            if !is_empty(read_request) {
                read_requests[i].read_request.value = compute_siloed_nullifier(read_request.contract_address, read_request.value());
            }
        }

        let hints = self.nullifier_non_existent_read_request_hints;

        assert_sorted_array(
            self.pending_nullifiers,
            hints.sorted_pending_values,
            hints.sorted_pending_value_index_hints,
            |a: Nullifier, b: Nullifier| a.value.lt(b.value)
        );
        let sorted_pending_nullifiers = array_to_bounded_vec(hints.sorted_pending_values);

        reset_non_existent_read_requests(
            read_requests,
            hints.non_membership_hints,
            self.nullifier_tree_root,
            sorted_pending_nullifiers,
            hints.next_pending_value_indices
        );
    }

    fn validate_public_data_read_requests(self) {
        let hints = self.public_data_read_request_hints;

        reset_mutable_data_read_requests(
            self.validation_requests.public_data_reads,
            hints.read_request_statuses,
            self.pending_public_data_writes,
            self.public_data_hints,
            hints.pending_read_hints,
            hints.leaf_data_read_hints
        );
    }
}
