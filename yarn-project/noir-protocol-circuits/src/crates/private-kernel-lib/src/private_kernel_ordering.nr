use crate::common;
use dep::aztec::constants_gen::{
    EMPTY_NULLIFIED_COMMITMENT,
    MAX_NEW_COMMITMENTS_PER_TX,
    MAX_NEW_NULLIFIERS_PER_TX,
    MAX_READ_REQUESTS_PER_TX,
};
use dep::std::unsafe;
use dep::types::{
    abis::{
        call_request::CallRequest,
        previous_kernel_data::PreviousKernelData,
        kernel_circuit_public_inputs::{
            KernelCircuitPublicInputsBuilder, 
            KernelCircuitPublicInputsFinal,
        },
    },
    hash::{
        compute_commitment_nonce,
        compute_unique_siloed_commitment,
    },
    utils::{
        arrays::{array_length, struct_array_length},
        bounded_vec::BoundedVec,
    },
};

struct PrivateKernelInputsOrdering {
    previous_kernel: PreviousKernelData,
    read_commitment_hints: [Field; MAX_READ_REQUESTS_PER_TX],
    nullifier_commitment_hints: [Field; MAX_NEW_NULLIFIERS_PER_TX],
}

impl PrivateKernelInputsOrdering {
    fn validate_inputs(self) {
        assert_eq(struct_array_length(self.previous_kernel.public_inputs.end.private_call_stack, |c: CallRequest| c.is_empty()), 0, 
            "Private call stack must be empty when executing the ordering circuit");
    }

    fn match_reads_to_commitments(self, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        let new_commitments = public_inputs.end.new_commitments;
        let read_requests = public_inputs.end.read_requests;

        // match reads to commitments from the previous call(s)
        for rr_idx in 0..MAX_READ_REQUESTS_PER_TX {
            let read_request = read_requests.get_unchecked(rr_idx);
            let read_commitment_hint = self.read_commitment_hints[rr_idx];
            //TODO(David): Shouldn't this just be a uint64?
            let hint_pos = read_commitment_hint as u64;

            if (read_request != 0) {
                let mut match_pos = MAX_NEW_COMMITMENTS_PER_TX as u64;
                if (hint_pos < MAX_NEW_COMMITMENTS_PER_TX as u64) {
                    match_pos = if read_request == new_commitments.get_unchecked(hint_pos as Field) { hint_pos } else { match_pos };
                }
            
                assert(match_pos != MAX_NEW_COMMITMENTS_PER_TX as u64, "read request is transient but does not match any commitment");
            }
        }

        // Empty out read requests after matching them to commitments
        public_inputs.end.read_requests = BoundedVec::new(0);
    }

    fn match_nullifiers_to_commitments_and_squash(self, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        // Remark: The commitments in public_inputs.end have already been siloed by contract address!
        // Match nullifiers/nullified_commitments to commitments from the previous call(s)
        let nullified_commitments = public_inputs.end.nullified_commitments.storage;
        let mut new_commitments = public_inputs.end.new_commitments.storage;
        let mut new_nullifiers = public_inputs.end.new_nullifiers.storage;

        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            // TODO - should not be able to squash the first nullifier.
            let nullified_commitment = nullified_commitments[n_idx];
            let nullifier_commitment_hint = self.nullifier_commitment_hints[n_idx];
            let hint_pos = nullifier_commitment_hint as u64;

            // Nullified_commitment of value `EMPTY_NULLIFIED_COMMITMENT` implies non-transient (persistable)
            // nullifier in which case no attempt will be made to match it to a commitment.
            // Non-empty nullified_commitment implies transient nullifier which MUST be matched to a commitment below!
            // 0-valued nullified_commitment is empty and will be ignored
            if (nullified_commitments[n_idx] != 0) & (nullified_commitments[n_idx] != EMPTY_NULLIFIED_COMMITMENT) {
                assert(hint_pos < MAX_NEW_COMMITMENTS_PER_TX as u64, "New nullifier is transient but hint is invalid");
                assert_eq(nullified_commitment, new_commitments[hint_pos], "Hinted commitment does not match");
                // match found!
                // squash both the nullifier and the commitment
                // (set to 0 here and then rearrange array after loop)
                new_commitments[hint_pos] = 0;
                new_nullifiers[n_idx as u64] = 0;
            }
            // non-transient (persistable) nullifiers are just kept in new_nullifiers array and forwarded
            // to public inputs (used later by base rollup circuit)
        }
        // Move all zero-ed (removed) entries of these arrays to the end and preserve ordering of other entries

        let mut new_commitments_vec = BoundedVec::new(0);

        for c_idx in 0..MAX_NEW_COMMITMENTS_PER_TX {
            if new_commitments[c_idx] != 0 {
                new_commitments_vec.push(new_commitments[c_idx]);
            }
        }

        public_inputs.end.new_commitments = new_commitments_vec;

        let mut new_nullifiers_vec = BoundedVec::new(0);

        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            if new_nullifiers[n_idx] != 0 {
                new_nullifiers_vec.push(new_nullifiers[n_idx]);
            }
        }

        public_inputs.end.new_nullifiers = new_nullifiers_vec;
    }

    fn apply_commitment_nonces(public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        // Remark: The commitments in public_inputs.end have already been siloed by contract address!
        // tx hash
        let first_nullifier = public_inputs.end.new_nullifiers.get(0);
        let mut unique_commitments = public_inputs.end.new_commitments.storage;

        for c_idx in 0..MAX_NEW_COMMITMENTS_PER_TX {
            // Apply nonce to all non-zero/non-empty commitments
            // Nonce is the hash of the first (0th) nullifier and the commitment's index into new_commitments array
            let nonce = compute_commitment_nonce(first_nullifier, c_idx);
            let commitment = unique_commitments[c_idx];
            if commitment != 0 {
                let unique_commitment = compute_unique_siloed_commitment(nonce, commitment);
                unique_commitments[c_idx] = unique_commitment;
            }
        }

        public_inputs.end.new_commitments.storage = unique_commitments;
    }

    pub fn native_private_kernel_circuit_ordering(self) -> KernelCircuitPublicInputsFinal {
        let mut public_inputs : KernelCircuitPublicInputsBuilder = unsafe::zeroed();
        public_inputs.is_private = true;

        self.validate_inputs();

        common::validate_previous_kernel_values(self.previous_kernel.public_inputs.end);
        
        // Do this before any functions can modify the inputs.
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1486): validate that `len(new_nullifiers) ==
        // len(nullified_commitments)`

        self.match_reads_to_commitments(&mut public_inputs);

        self.match_nullifiers_to_commitments_and_squash(&mut public_inputs);        

        PrivateKernelInputsOrdering::apply_commitment_nonces(&mut public_inputs); 

        public_inputs.to_final()
    }
}

mod tests {
    use crate::private_kernel_ordering::PrivateKernelInputsOrdering;
    use dep::aztec::constants_gen::{
        MAX_READ_REQUESTS_PER_TX,
        MAX_NEW_COMMITMENTS_PER_TX,
        MAX_NEW_NULLIFIERS_PER_TX,
    };
    use dep::types::{
        abis::kernel_circuit_public_inputs::KernelCircuitPublicInputsFinal,
        hash::compute_unique_siloed_commitments,
        tests::previous_kernel_data_builder::PreviousKernelDataBuilder,
        utils::{
            arrays::{array_eq, array_length, is_empty_array},
            bounded_vec::BoundedVec,
        },
    };

    struct PrivateKernelOrderingInputsBuilder {
        previous_kernel: PreviousKernelDataBuilder,
        read_commitment_hints: [Field; MAX_READ_REQUESTS_PER_TX],
        nullifier_commitment_hints: [Field; MAX_NEW_NULLIFIERS_PER_TX],
    }

    impl PrivateKernelOrderingInputsBuilder {
        pub fn new() -> Self {
            PrivateKernelOrderingInputsBuilder {
                previous_kernel: PreviousKernelDataBuilder::new(),
                read_commitment_hints: [0; MAX_READ_REQUESTS_PER_TX],
                nullifier_commitment_hints: [0; MAX_NEW_NULLIFIERS_PER_TX],
            }
        }

        pub fn get_new_commitments(self) -> [Field; MAX_NEW_COMMITMENTS_PER_TX] {
            self.previous_kernel.end.new_commitments.storage
        }
        
        pub fn get_new_nullifiers(self) -> [Field; MAX_NEW_NULLIFIERS_PER_TX] {
            self.previous_kernel.end.new_nullifiers.storage
        }

        pub fn get_unique_siloed_commitments(self) -> [Field; MAX_NEW_COMMITMENTS_PER_TX] {
            self.compute_unique_siloed_commitments(self.previous_kernel.end.new_commitments.storage)
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed 
        // commitments for the given commitments.
        pub fn compute_unique_siloed_commitments<N>(self, commitments: [Field; N]) -> [Field; N] {
            let first_nullifier = self.previous_kernel.end.new_nullifiers.get_unchecked(0);
            compute_unique_siloed_commitments(first_nullifier, commitments)
        }

        pub fn append_transient_commitments(&mut self, num_commitments: Field) {
            // All new commitments aggregated in the previous kernel are transient commitments.
            self.previous_kernel.append_new_commitments(num_commitments);
        }

        pub fn add_transient_read(&mut self, commitment_index: Field) {
            let read_request_index = self.previous_kernel.add_read_request_for_transient_commitment(commitment_index);
            self.read_commitment_hints[read_request_index] = commitment_index;
        }

        pub fn append_nullifiers(&mut self, num_nullifiers: Field) {
            self.previous_kernel.append_new_nullifiers(num_nullifiers);
        }
        
        pub fn nullify_transient_commitment(&mut self, nullifier_index: Field, commitment_index: Field) {
            self.previous_kernel.end.nullified_commitments.storage[nullifier_index] = self.previous_kernel.end.new_commitments.get(commitment_index);
            self.nullifier_commitment_hints[nullifier_index] = commitment_index;
        }

        pub fn execute(self) -> KernelCircuitPublicInputsFinal {
            let kernel = PrivateKernelInputsOrdering {
                previous_kernel: self.previous_kernel.finish(),
                read_commitment_hints: self.read_commitment_hints,
                nullifier_commitment_hints: self.nullifier_commitment_hints,
            };
            kernel.native_private_kernel_circuit_ordering()
        }

        pub fn failed(self) {
            let _ = self.execute();
        }
    }

    #[test]
    fn native_matching_one_read_request_to_commitment_works() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(1);
        builder.add_transient_read(0);

        let unique_siloed_commitments = builder.get_unique_siloed_commitments();

        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_commitments) == 1);
        assert(public_inputs.end.new_commitments[0] == unique_siloed_commitments[0]);
    }

    #[test]
    fn native_matching_some_read_requests_to_commitments_works() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(MAX_NEW_COMMITMENTS_PER_TX);
        // Read the commitment at index 1;
        builder.add_transient_read(1);
        // Read the commitment at index 3;
        builder.add_transient_read(3);

        let unique_siloed_commitments = builder.get_unique_siloed_commitments();

        let public_inputs = builder.execute();
        assert_eq(array_length(public_inputs.end.new_commitments), MAX_NEW_COMMITMENTS_PER_TX);
        for i in 0..MAX_NEW_COMMITMENTS_PER_TX {
            assert(public_inputs.end.new_commitments[i] == unique_siloed_commitments[i]);
        }
    }

    #[test(should_fail_with="read request is transient but does not match any commitment")]
    fn native_read_request_unknown_fails() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(1);
        builder.add_transient_read(0);
        // Tweak the read request so that it does not match the commitment at index 0;
        let read_request = builder.previous_kernel.end.read_requests.pop();
        builder.previous_kernel.end.read_requests.push(read_request + 1);

        builder.failed();
    }

    #[test]
    fn native_squash_one_of_one_transient_matches_works() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(1);
        builder.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the commitment at index 0;
        builder.nullify_transient_commitment(1, 0);
        let new_nullifiers = builder.get_new_nullifiers();

        let public_inputs = builder.execute();
        assert(is_empty_array(public_inputs.end.new_commitments));

        // The nullifier at index 1 is chopped.
        let expected_new_nullifiers = [new_nullifiers[0], new_nullifiers[2]];
        assert(array_eq(public_inputs.end.new_nullifiers, expected_new_nullifiers));
    }

    #[test]
    fn native_squash_one_of_two_transient_matches_works() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the commitment at index 0;
        builder.nullify_transient_commitment(1, 0);

        let new_commitments = builder.get_new_commitments();
        // The 0th commitment will be chopped.
        let unique_siloed_commitments = builder.compute_unique_siloed_commitments([new_commitments[1]]);
        let new_nullifiers = builder.get_new_nullifiers();

        let public_inputs = builder.execute();
        assert(array_eq(public_inputs.end.new_commitments, unique_siloed_commitments));

        // The nullifier at index 1 is chopped.
        let expected_new_nullifiers = [new_nullifiers[0], new_nullifiers[2]];
        assert(array_eq(public_inputs.end.new_nullifiers, expected_new_nullifiers));
    }

    #[test]
    fn native_squash_two_of_two_transient_matches_works() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the commitment at index 1;
        builder.nullify_transient_commitment(1, 1);
        // The nullifier at index 2 is nullifying the commitment at index 0;
        builder.nullify_transient_commitment(2, 0);

        let new_nullifiers = builder.get_new_nullifiers();

        let public_inputs = builder.execute();
        assert(is_empty_array(public_inputs.end.new_commitments));
        assert(array_eq(public_inputs.end.new_nullifiers, [new_nullifiers[0]]));
    }

    #[test]
    fn native_empty_nullified_commitment_means_persistent_nullifier_0() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);

        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_commitments) == 2);
        assert(array_length(public_inputs.end.new_nullifiers) == 3);
    }

    // same as previous test, but this time there are 0 commitments!
    // (Do we really need this test?)
    #[test]
    fn native_empty_nullified_commitment_means_persistent_nullifier_1() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_nullifiers(2);

        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_commitments) == 0);
        assert(array_length(public_inputs.end.new_nullifiers) == 3);
    }

    #[test(should_fail_with="New nullifier is transient but hint is invalid")]
    fn invalid_nullifier_commitment_hint_fails() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(1);
        builder.append_nullifiers(1);
        // The nullifier at index 1 is nullifying the commitment at index 0;
        builder.nullify_transient_commitment(1, 0);
        // Change the hint to be out of bounds.
        builder.nullifier_commitment_hints[1] = MAX_NEW_COMMITMENTS_PER_TX;

        builder.failed();
    }

    #[test(should_fail_with="Hinted commitment does not match")]
    fn wrong_nullifier_commitment_hint_fails() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the commitment at index 1;
        builder.nullify_transient_commitment(1, 1);
        // The nullifier at index 2 is nullifying the commitment at index 0;
        builder.nullify_transient_commitment(2, 0);
        // Tweak the hint to be for the commitment at index 1.
        builder.nullifier_commitment_hints[2] = 1;

        builder.failed();
    }

    #[test(should_fail_with="Private call stack must be empty when executing the ordering circuit")]
    fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.previous_kernel.push_private_call_request(1, false);

        builder.failed();
    }

    #[test(should_fail_with="The 0th nullifier in the accumulated nullifier array is zero")]
    fn zero_0th_nullifier_fails() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.previous_kernel.end.new_nullifiers = BoundedVec::new(0);

        builder.failed();
    }
}
