use dep::aztec::constants_gen::{
    EMPTY_NULLIFIED_COMMITMENT,
    MAX_NEW_COMMITMENTS_PER_TX,
    MAX_NEW_NULLIFIERS_PER_TX,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_READ_REQUESTS_PER_TX,
};
use crate::common;
use dep::types::mocked::{Proof, AggregationObject, verify_previous_kernel_state};
use dep::types::transaction::request::TxRequest;
use dep::types::abis::{
    combined_constant_data::CombinedConstantData,
    previous_kernel_data::PreviousKernelData,
    private_kernel::private_call_data::PrivateCallData,
    kernel_circuit_public_inputs::{
        KernelCircuitPublicInputs, 
        KernelCircuitPublicInputsBuilder, 
        KernelCircuitPublicInputsFinal,
    },
};
use dep::types::utils::{
    arrays,
    bounded_vec::BoundedVec,
};
use dep::types::hash::{
    compute_commitment_nonce,
    compute_unique_commitment,
};
use dep::std::unsafe;

struct PrivateKernelInputsOrdering {
    previous_kernel : PreviousKernelData,

    read_commitment_hints : [Field; MAX_READ_REQUESTS_PER_TX],
    nullifier_commitment_hints : [Field; MAX_NEW_NULLIFIERS_PER_TX],
}

impl PrivateKernelInputsOrdering {
    fn match_reads_to_commitments(public_inputs : &mut KernelCircuitPublicInputsBuilder, read_commitment_hints :[Field; MAX_READ_REQUESTS_PER_TX]) {
        let new_commitments = public_inputs.end.new_commitments;
        let read_requests = public_inputs.end.read_requests;

        // match reads to commitments from the previous call(s)
        for rr_idx in 0..MAX_READ_REQUESTS_PER_TX {
            let read_request = read_requests.get_unchecked(rr_idx);
            let read_commitment_hint = read_commitment_hints[rr_idx];
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

    fn match_nullifiers_to_commitments_and_squash(public_inputs : &mut KernelCircuitPublicInputsBuilder, nullifier_commitment_hints : [Field; MAX_NEW_NULLIFIERS_PER_TX]) {
        // Remark: The commitments in public_inputs.end have already been siloed by contract address!
        // Match nullifiers/nullified_commitments to commitments from the previous call(s)
        let nullified_commitments = public_inputs.end.nullified_commitments.storage;
        let mut new_commitments = public_inputs.end.new_commitments.storage;
        let mut new_nullifiers = public_inputs.end.new_nullifiers.storage;

        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            // TODO - should not be able to squash the first nullifier.
            let nullified_commitment = nullified_commitments[n_idx];
            let nullifier_commitment_hint = nullifier_commitment_hints[n_idx];
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

    fn apply_commitment_nonces(public_inputs : &mut KernelCircuitPublicInputsBuilder) {
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
                let unique_commitment = compute_unique_commitment(nonce, commitment);
                unique_commitments[c_idx] = unique_commitment;
            }
        }

        public_inputs.end.new_commitments.storage = unique_commitments;
    }

    pub fn native_private_kernel_circuit_ordering(self) -> KernelCircuitPublicInputsFinal {
        let mut public_inputs : KernelCircuitPublicInputsBuilder = unsafe::zeroed();
        public_inputs.is_private = true;

        common::validate_previous_kernel_values(self.previous_kernel.public_inputs.end);
        
        // Do this before any functions can modify the inputs.
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1486): validate that `len(new_nullifiers) ==
        // len(nullified_commitments)`

        PrivateKernelInputsOrdering::match_reads_to_commitments(&mut public_inputs, self.read_commitment_hints);

        PrivateKernelInputsOrdering::match_nullifiers_to_commitments_and_squash(&mut public_inputs, self.nullifier_commitment_hints);        

        PrivateKernelInputsOrdering::apply_commitment_nonces(&mut public_inputs); 

        public_inputs.to_final()
    }
}

mod tests {
    use dep::types::abis::{
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        read_request_membership_witness::ReadRequestMembershipWitness,
    };
    use dep::types::address::Address;
    use dep::types::hash::{
        compute_commitment_nonce,
        compute_unique_commitment,
    };
    use crate::private_kernel_ordering::PrivateKernelInputsOrdering;
    use dep::types::tests::{
        apps::constructor::constructor_app,
        testing_harness::{
            create_previous_kernel_data,
            non_zero_items,
            PrivateAppInputs,
        },
    };
    use dep::aztec::constants_gen::{
        MAX_READ_REQUESTS_PER_TX,
        MAX_NEW_COMMITMENTS_PER_TX,
        MAX_NEW_NULLIFIERS_PER_TX,
        EMPTY_NULLIFIED_COMMITMENT,
        MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
        MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    };
    use dep::types::utils::{
        bounded_vec::BoundedVec,
    };

    fn build_inputs<T>(
        function: fn (PrivateAppInputs, T) -> PrivateCircuitPublicInputs,
        app_params: T,
    ) -> PrivateKernelInputsOrdering {
        let msg_sender = Address::from_field(27);
        let previous_kernel = create_previous_kernel_data(
            false,
            function,
            app_params,
            msg_sender,
        );

        let read_commitment_hints = [0; MAX_READ_REQUESTS_PER_TX];
        let nullifier_commitment_hints = [0; MAX_NEW_NULLIFIERS_PER_TX];

        PrivateKernelInputsOrdering {
            previous_kernel,
            read_commitment_hints,
            nullifier_commitment_hints,
        }
    }

    fn generate_unique_siloed_commitments<N>(
        private_inputs: PrivateKernelInputsOrdering,
        siloed_commitments: [Field; MAX_NEW_COMMITMENTS_PER_TX],
        squashed_indices: [Field; N],
    ) -> [Field; MAX_NEW_COMMITMENTS_PER_TX] {
        let mut valid_indices = [true; MAX_NEW_COMMITMENTS_PER_TX];
        for i in 0..N {
            valid_indices[squashed_indices[i]] = false;
        }

        let mut unique_siloed_commitments = BoundedVec::new(0);
        let first_nullifier = private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0];
        for i in 0..MAX_NEW_COMMITMENTS_PER_TX {
            let siloed_commitment = siloed_commitments[i];
            if (siloed_commitment != 0) & valid_indices[i] {
                let idx = unique_siloed_commitments.len();
                let nonce = compute_commitment_nonce(first_nullifier, idx);
                let unique_siloed_commitment = compute_unique_commitment(nonce, siloed_commitment);
                unique_siloed_commitments.push(unique_siloed_commitment);
            }
        }

        unique_siloed_commitments.storage
    }

    fn mock_new_commitments(private_inputs: &mut PrivateKernelInputsOrdering, num_new_commitments: Field) -> (
        [Field; MAX_NEW_COMMITMENTS_PER_TX],
        [Field; MAX_NEW_COMMITMENTS_PER_TX],
    ) {
        let mut new_commitments = [0; MAX_NEW_COMMITMENTS_PER_TX];
        for i in 0..MAX_NEW_COMMITMENTS_PER_TX {
            if i as u64 < num_new_commitments as u64 {
                let siloed_commitment = i + 623;
                new_commitments[i] = siloed_commitment;
            }
        }

        private_inputs.previous_kernel.public_inputs.end.new_commitments = new_commitments;

        let unique_siloed_commitments = generate_unique_siloed_commitments(*private_inputs, new_commitments, []);

        (new_commitments, unique_siloed_commitments)
    }

    fn mock_new_nullifiers(private_inputs: &mut PrivateKernelInputsOrdering, num_extra_nullifier: Field) -> [Field; MAX_NEW_NULLIFIERS_PER_TX] {
        let mut new_nullifiers = [0; MAX_NEW_NULLIFIERS_PER_TX];
        let first_nullifier = private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0];
        new_nullifiers[0] = first_nullifier;

        for i in 1..MAX_NEW_NULLIFIERS_PER_TX {
            if i as u64 <= num_extra_nullifier as u64 {
                // Set a random value that's different to the first nullifier.
                new_nullifiers[i] = first_nullifier + i;
            }
        }

        private_inputs.previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;

        new_nullifiers
    }

    #[test]
    fn native_matching_one_read_request_to_commitment_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let (new_commitments, unique_siloed_commitments) = mock_new_commitments(&mut private_inputs, 1);
        private_inputs.read_commitment_hints[0] = 0;
        private_inputs.previous_kernel.public_inputs.end.read_requests[0] = new_commitments[0];

        let public_inputs = private_inputs.native_private_kernel_circuit_ordering();
        assert(non_zero_items(public_inputs.end.new_commitments) == 1);
        assert(public_inputs.end.new_commitments[0] == unique_siloed_commitments[0]);
    }

    #[test]
    fn native_matching_some_read_requests_to_commitments_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let (new_commitments, unique_siloed_commitments) = mock_new_commitments(&mut private_inputs, MAX_NEW_COMMITMENTS_PER_TX);
        // Read the commitment at index 1;
        private_inputs.read_commitment_hints[0] = 1;
        private_inputs.previous_kernel.public_inputs.end.read_requests[0] = new_commitments[1];
        // Read the commitment at index 3;
        private_inputs.read_commitment_hints[1] = 3;
        private_inputs.previous_kernel.public_inputs.end.read_requests[1] = new_commitments[3];

        let public_inputs = private_inputs.native_private_kernel_circuit_ordering();
        assert(non_zero_items(public_inputs.end.new_commitments) == MAX_NEW_COMMITMENTS_PER_TX as u64);
        for i in 0..MAX_NEW_COMMITMENTS_PER_TX {
            assert(public_inputs.end.new_commitments[i] == unique_siloed_commitments[i]);
        }
    }

    #[test(should_fail_with="read request is transient but does not match any commitment")]
    fn native_read_request_unknown_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let (new_commitments, _) = mock_new_commitments(&mut private_inputs, 1);
        private_inputs.read_commitment_hints[0] = 0;
        // The read request does not match the commitment at index 0;
        private_inputs.previous_kernel.public_inputs.end.read_requests[0] = new_commitments[0] + 1;

        private_inputs.native_private_kernel_circuit_ordering();
    }

    #[test]
    fn native_squash_one_of_one_transient_matches_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let (new_commitments, _) = mock_new_commitments(&mut private_inputs, 1);
        let new_nullifiers = mock_new_nullifiers(&mut private_inputs, 2);
        // The nullifier at index 1 is nullifying the commitment at index 0;
        let transient_nullifier_index = 1;
        let nullified_commitment_index = 0;
        private_inputs.previous_kernel.public_inputs.end.nullified_commitments[transient_nullifier_index] = new_commitments[nullified_commitment_index];
        private_inputs.nullifier_commitment_hints[transient_nullifier_index] = nullified_commitment_index;

        let public_inputs = private_inputs.native_private_kernel_circuit_ordering();
        assert(non_zero_items(public_inputs.end.new_commitments) == 0);
        assert(non_zero_items(public_inputs.end.new_nullifiers) == 2);
        assert(public_inputs.end.new_nullifiers[0] == new_nullifiers[0]);
        assert(public_inputs.end.new_nullifiers[1] == new_nullifiers[2]);
    }

    #[test]
    fn native_squash_one_of_two_transient_matches_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let (new_commitments, _) = mock_new_commitments(&mut private_inputs, 2);
        let new_nullifiers = mock_new_nullifiers(&mut private_inputs, 2);
        // The nullifier at index 1 is nullifying the commitment at index 0;
        let transient_nullifier_index = 1;
        let nullified_commitment_index = 0;
        private_inputs.previous_kernel.public_inputs.end.nullified_commitments[transient_nullifier_index] = new_commitments[nullified_commitment_index];
        private_inputs.nullifier_commitment_hints[transient_nullifier_index] = nullified_commitment_index;

        // The 0th commitment is chopped.
        let unique_siloed_commitments = generate_unique_siloed_commitments(private_inputs, new_commitments, [0]);

        let public_inputs = private_inputs.native_private_kernel_circuit_ordering();
        assert(non_zero_items(public_inputs.end.new_commitments) == 1);
        assert(public_inputs.end.new_commitments[0] == unique_siloed_commitments[0]);
        assert(non_zero_items(public_inputs.end.new_nullifiers) == 2);
        assert(public_inputs.end.new_nullifiers[0] == new_nullifiers[0]);
        assert(public_inputs.end.new_nullifiers[1] == new_nullifiers[2]);
    }

    #[test]
    fn native_squash_two_of_two_transient_matches_works() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let (new_commitments, _) = mock_new_commitments(&mut private_inputs, 2);
        let new_nullifiers = mock_new_nullifiers(&mut private_inputs, 2);
        // The nullifier at index 1 is nullifying the commitment at index 1;
        private_inputs.previous_kernel.public_inputs.end.nullified_commitments[1] = new_commitments[1];
        private_inputs.nullifier_commitment_hints[1] = 1;
        // The nullifier at index 2 is nullifying the commitment at index 0;
        private_inputs.previous_kernel.public_inputs.end.nullified_commitments[2] = new_commitments[0];
        private_inputs.nullifier_commitment_hints[2] = 0;

        let public_inputs = private_inputs.native_private_kernel_circuit_ordering();
        assert(non_zero_items(public_inputs.end.new_commitments) == 0);
        assert(non_zero_items(public_inputs.end.new_nullifiers) == 1);
        assert(public_inputs.end.new_nullifiers[0] == new_nullifiers[0]);
    }

    #[test]
    fn native_empty_nullified_commitment_means_persistent_nullifier_0() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        mock_new_commitments(&mut private_inputs, 2);
        mock_new_nullifiers(&mut private_inputs, 2);

        let public_inputs = private_inputs.native_private_kernel_circuit_ordering();
        assert(non_zero_items(public_inputs.end.new_commitments) == 2);
        assert(non_zero_items(public_inputs.end.new_nullifiers) == 3);
    }

    // same as previous test, but this time there are 0 commitments!
    // (Do we really need this test?)
    #[test]
    fn native_empty_nullified_commitment_means_persistent_nullifier_1() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        mock_new_nullifiers(&mut private_inputs, 2);

        let public_inputs = private_inputs.native_private_kernel_circuit_ordering();
        assert(non_zero_items(public_inputs.end.new_commitments) == 0);
        assert(non_zero_items(public_inputs.end.new_nullifiers) == 3);
    }

    #[test(should_fail_with="The 0th nullifier in the accumulated nullifier array is zero")]
    fn zero_0th_nullifier_fails() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0] = 0;

        private_inputs.native_private_kernel_circuit_ordering();
    }

    #[test(should_fail_with="invalid array")]
    fn input_validation_malformed_end_arrays_read_requests() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_read_requests = [0; MAX_READ_REQUESTS_PER_TX];
        malformed_read_requests[1] = 9123;
        malformed_read_requests[3] = 12;
        private_inputs.previous_kernel.public_inputs.end.read_requests = malformed_read_requests;

        private_inputs.native_private_kernel_circuit_ordering();
    }

    #[test(should_fail_with="invalid array")]
    fn input_validation_malformed_end_arrays_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_commitments = [0; MAX_NEW_COMMITMENTS_PER_TX];
        malformed_commitments[1] = 9123;
        private_inputs.previous_kernel.public_inputs.end.new_commitments = malformed_commitments;

        private_inputs.native_private_kernel_circuit_ordering();
    }

    #[test(should_fail_with="invalid array")]
    fn input_validation_malformed_end_arrays_nullifiers() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_nullifiers = [0; MAX_NEW_NULLIFIERS_PER_TX];
        malformed_nullifiers[MAX_NEW_NULLIFIERS_PER_TX - 1] = 12;
        private_inputs.previous_kernel.public_inputs.end.new_nullifiers = malformed_nullifiers;

        private_inputs.native_private_kernel_circuit_ordering();
    }

    #[test(should_fail_with="invalid array")]
    fn input_validation_malformed_end_arrays_nullified_commitments() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_nullified_commitments = [0; MAX_NEW_NULLIFIERS_PER_TX];
        malformed_nullified_commitments[2] = EMPTY_NULLIFIED_COMMITMENT;
        private_inputs.previous_kernel.public_inputs.end.nullified_commitments = malformed_nullified_commitments;

        private_inputs.native_private_kernel_circuit_ordering();
    }

    #[test(should_fail_with="invalid array")]
    fn input_validation_malformed_end_arrays_private_call_stack() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_private_call_stack = [0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX];
        malformed_private_call_stack[1] = 888;
        private_inputs.previous_kernel.public_inputs.end.private_call_stack = malformed_private_call_stack;

        private_inputs.native_private_kernel_circuit_ordering();
    }

    #[test(should_fail_with="invalid array")]
    fn input_validation_malformed_end_arrays_public_call_stack() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_public_call_stack = [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX];
        malformed_public_call_stack[1] = 888;
        private_inputs.previous_kernel.public_inputs.end.public_call_stack = malformed_public_call_stack;

        private_inputs.native_private_kernel_circuit_ordering();
    }

    #[test(should_fail_with="invalid array")]
    fn input_validation_malformed_end_arrays_l2_to_l1_msgs() {
        let params = dep::std::unsafe::zeroed();
        let mut private_inputs = build_inputs(constructor_app, params);

        let mut malformed_l2_to_l1_msgs = [0; MAX_NEW_L2_TO_L1_MSGS_PER_TX];
        malformed_l2_to_l1_msgs[MAX_NEW_L2_TO_L1_MSGS_PER_TX - 1] = 1;
        private_inputs.previous_kernel.public_inputs.end.new_l2_to_l1_msgs = malformed_l2_to_l1_msgs;

        private_inputs.native_private_kernel_circuit_ordering();
    }
}
