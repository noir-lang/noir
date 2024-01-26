use crate::common;
use dep::std::{
    cmp::Eq,
    option::Option,
    unsafe,
};
use dep::types::{
    abis::{
        call_request::CallRequest,
        nullifier_key_validation_request::NullifierKeyValidationRequestContext,
        previous_kernel_data::PreviousKernelData,
        kernel_circuit_public_inputs::{
            KernelCircuitPublicInputsBuilder, 
            KernelCircuitPublicInputsFinal,
        },
        side_effect::{SideEffect, SideEffectLinkedToNoteHash, Ordered},
    },
    constants::{
        MAX_NEW_COMMITMENTS_PER_TX,
        MAX_NEW_NULLIFIERS_PER_TX,
        MAX_READ_REQUESTS_PER_TX,
        MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    },
    grumpkin_private_key::GrumpkinPrivateKey,
    hash::{
        compute_commitment_nonce,
        compute_unique_siloed_commitment,
    },
    utils::{
        arrays::{array_length, array_eq},
        bounded_vec::BoundedVec,
    },
    traits::{Empty, is_empty}
};

struct PrivateKernelInputsOrdering {
    previous_kernel: PreviousKernelData,
    sorted_new_commitments: [SideEffect; MAX_NEW_COMMITMENTS_PER_TX],
    sorted_new_commitments_indexes: [u32; MAX_NEW_COMMITMENTS_PER_TX],
    read_commitment_hints: [Field; MAX_READ_REQUESTS_PER_TX],
    sorted_new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_new_nullifiers_indexes: [u32; MAX_NEW_NULLIFIERS_PER_TX],
    nullifier_commitment_hints: [Field; MAX_NEW_NULLIFIERS_PER_TX],
    master_nullifier_secret_keys: [GrumpkinPrivateKey; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
}

impl PrivateKernelInputsOrdering {
    fn validate_inputs(self) {
        assert_eq(array_length(self.previous_kernel.public_inputs.end.private_call_stack), 0, 
            "Private call stack must be empty when executing the ordering circuit");
    }

    fn validate_nullifier_keys(self, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        let requests = self.previous_kernel.public_inputs.end.nullifier_key_validation_requests;
        for i in 0..MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX {
            let request = requests[i];
            if !is_empty(request) {
                let master_secret_key = self.master_nullifier_secret_keys[i];
                let computed_public_key = master_secret_key.derive_public_key();
                assert(computed_public_key.eq(request.public_key), "Cannot derive nullifier public key from the master key.");

                let computed_secret_key = common::compute_siloed_nullifier_secret_key(master_secret_key, request.contract_address);
                assert(computed_secret_key.eq(request.secret_key), "Cannot derive siloed secret key from the master key.");
            }
        }

        // Empty out nullifier key validation requests after verifying them.
        public_inputs.end.nullifier_key_validation_requests = BoundedVec::new(NullifierKeyValidationRequestContext::empty());
    }

    fn match_reads_to_commitments(self, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        let new_commitments = public_inputs.end.new_commitments;
        let read_requests = public_inputs.end.read_requests;

        // match reads to commitments from the previous call(s)
        for rr_idx in 0..MAX_READ_REQUESTS_PER_TX {
            let read_request = read_requests.get_unchecked(rr_idx);
            let read_commitment_hint = self.read_commitment_hints[rr_idx] as u64;

            if (read_request.value != 0) {
                let commitment = new_commitments.get_unchecked(read_commitment_hint as Field);
                assert_eq(read_request.value, commitment.value, "Hinted commitment does not match read request");
                assert(read_request.counter > commitment.counter, "Read request counter must be greater than commitment counter");
            }
        }

        // Empty out read requests after matching them to commitments
        public_inputs.end.read_requests = BoundedVec::new(SideEffect::empty());
    }

    fn assert_sorted_counters<T, N>(original: [T; N], sorted: [T; N], indexes: [u32; N]) where T: Eq + Ordered + Empty {
        let mut prev_was_empty = false;

        for i in 0..N {
            let item = if prev_was_empty {
                sorted[i]
            } else {
                sorted[indexes[i]]
            };
            assert(item.eq(original[i]), "Sorted item is not equal");
            let is_empty = is_empty(item);

            if prev_was_empty {
                assert(is_empty, "Empty items must be at the end");
            } else if (i != 0) & !is_empty {
                assert(sorted[i].counter() > sorted[i - 1].counter(), "Not sorted");
            }

            prev_was_empty = is_empty;
        }
    }

    fn sort_arrays(self, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        PrivateKernelInputsOrdering::assert_sorted_counters(public_inputs.end.new_commitments.storage, self.sorted_new_commitments, self.sorted_new_commitments_indexes);
        PrivateKernelInputsOrdering::assert_sorted_counters(public_inputs.end.new_nullifiers.storage, self.sorted_new_nullifiers, self.sorted_new_nullifiers_indexes);
        public_inputs.end.new_commitments.storage = self.sorted_new_commitments;
        public_inputs.end.new_nullifiers.storage = self.sorted_new_nullifiers;
    }

    fn match_nullifiers_to_commitments_and_squash(self, public_inputs: &mut KernelCircuitPublicInputsBuilder) {
        // Remark: The commitments in public_inputs.end have already been siloed by contract address!
        // Match nullifiers/nullified_commitments to commitments from the previous call(s)
        let mut new_commitments = public_inputs.end.new_commitments.storage;
        let mut new_nullifiers = public_inputs.end.new_nullifiers.storage;

        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            let nullifier = new_nullifiers[n_idx];
            // TODO - should not be able to squash the first nullifier.
            let nullified_commitment = nullifier.note_hash;
            let hint_pos = self.nullifier_commitment_hints[n_idx] as u64;

            // Nullified_commitment of value `0` implies non-transient (persistable)
            // nullifier in which case no attempt will be made to match it to a commitment.
            // Non-empty nullified_commitment implies transient nullifier which MUST be matched to a commitment below!
            // 0-valued nullified_commitment is empty and will be ignored
            if nullified_commitment != 0 {
                assert(hint_pos < MAX_NEW_COMMITMENTS_PER_TX as u64, "New nullifier is transient but hint is invalid");
                let commitment = new_commitments[hint_pos];
                assert_eq(nullified_commitment, commitment.value, "Hinted commitment does not match");
                assert(nullifier.counter > commitment.counter, "Nullifier counter must be greater than commitment counter");
                // match found!
                // squash both the nullifier and the commitment
                // (set to 0 here and then rearrange array after loop)
                new_commitments[hint_pos] = SideEffect::empty();
                new_nullifiers[n_idx as u64] = SideEffectLinkedToNoteHash::empty();
            }
            // non-transient (persistable) nullifiers are just kept in new_nullifiers array and forwarded
            // to public inputs (used later by base rollup circuit)
        }

        // Move all zero-ed (removed) entries of these arrays to the end and preserve ordering of other entries

        let mut new_commitments_vec = BoundedVec::new(SideEffect::empty());

        for c_idx in 0..MAX_NEW_COMMITMENTS_PER_TX {
            if new_commitments[c_idx].value != 0 {
                new_commitments_vec.push(new_commitments[c_idx]);
            }
        }

        public_inputs.end.new_commitments = new_commitments_vec;

        let mut new_nullifiers_vec = BoundedVec::new(SideEffectLinkedToNoteHash::empty());

        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            if new_nullifiers[n_idx].value != 0 {
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
            let nonce = compute_commitment_nonce(first_nullifier.value, c_idx);
            let commitment = unique_commitments[c_idx];
            if commitment.value != 0 {
                let unique_commitment = compute_unique_siloed_commitment(nonce, commitment.value);
                unique_commitments[c_idx] = SideEffect{
                    value: unique_commitment,
                    counter: commitment.counter
                };
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

        self.validate_nullifier_keys(&mut public_inputs);

        self.sort_arrays(&mut public_inputs);

        self.match_reads_to_commitments(&mut public_inputs);

        self.match_nullifiers_to_commitments_and_squash(&mut public_inputs);        

        PrivateKernelInputsOrdering::apply_commitment_nonces(&mut public_inputs); 

        public_inputs.to_final()
    }
}

mod tests {
    use dep::std::cmp::Eq;
    use crate::private_kernel_ordering::PrivateKernelInputsOrdering;
    use dep::types::constants::{
        MAX_READ_REQUESTS_PER_TX,
        MAX_NEW_COMMITMENTS_PER_TX,
        MAX_NEW_NULLIFIERS_PER_TX,
    };
    use dep::types::{
        abis::{
            kernel_circuit_public_inputs::KernelCircuitPublicInputsFinal,
            side_effect::{SideEffect, SideEffectLinkedToNoteHash, Ordered},
        },
        hash::compute_unique_siloed_commitments,
        tests::previous_kernel_data_builder::PreviousKernelDataBuilder,
        utils::{
            arrays::{array_eq, array_length},
            bounded_vec::BoundedVec,
        },
        traits::{Empty, is_empty, is_empty_array}
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

        pub fn get_new_commitments(self) -> [SideEffect; MAX_NEW_COMMITMENTS_PER_TX] {
            self.previous_kernel.end.new_commitments.storage
        }

        pub fn get_new_nullifiers(self) -> [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX] {
            self.previous_kernel.end.new_nullifiers.storage
        }

        pub fn get_unique_siloed_commitments(self) -> [SideEffect; MAX_NEW_COMMITMENTS_PER_TX] {
            self.compute_unique_siloed_commitments(self.previous_kernel.end.new_commitments.storage)
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed 
        // commitments for the given commitments.
        pub fn compute_unique_siloed_commitments<N>(self, commitments: [SideEffect; N]) -> [SideEffect; N] {
            let first_nullifier = self.previous_kernel.end.new_nullifiers.get_unchecked(0);
            compute_unique_siloed_commitments(first_nullifier.value, commitments)
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
            self.previous_kernel.end.new_nullifiers.storage[nullifier_index].note_hash = self.previous_kernel.end.new_commitments.get(commitment_index).value;
            self.nullifier_commitment_hints[nullifier_index] = commitment_index;
        }

        fn sort_sideffects<T, N>(original: [T; N]) -> ([T; N],[u32; N]) where T: Ordered + Eq + Empty {
            let mut indexes = [0; N];
            for i in 0..N {
                indexes[i] = i as u32;
            }
            let sorted_indexes = indexes.sort_via(|a_index: u32, b_index: u32| {
                let a = original[a_index];
                let b = original[b_index];
                if is_empty(b) {
                    true
                } else if is_empty(a) {
                    false
                } else {
                    a.counter() < b.counter()
                }
            });
            let sorted_sideffects = sorted_indexes.map(|i: u32| original[i]);
            let mut reverse_map = [0; N];
            for i in 0..N {
                reverse_map[sorted_indexes[i]] = i;
            }

            (sorted_sideffects, reverse_map)
        }

        pub fn execute(self) -> KernelCircuitPublicInputsFinal {
            let (sorted_new_commitments, sorted_new_commitments_indexes) = PrivateKernelOrderingInputsBuilder::sort_sideffects(self.get_new_commitments());
            let mut sorted_read_commitment_hints = [0; MAX_READ_REQUESTS_PER_TX];
            for i in 0..sorted_read_commitment_hints.len() {
                sorted_read_commitment_hints[i] = sorted_new_commitments_indexes[self.read_commitment_hints[i]] as Field;
            }
            let (sorted_new_nullifiers, sorted_new_nullifiers_indexes) = PrivateKernelOrderingInputsBuilder::sort_sideffects(self.get_new_nullifiers());
            let mut sorted_nullifier_commitment_hints = [0; MAX_NEW_NULLIFIERS_PER_TX];
            for i in 0..sorted_nullifier_commitment_hints.len() {
                sorted_nullifier_commitment_hints[i] = sorted_new_nullifiers_indexes[self.nullifier_commitment_hints[i]] as Field;
            }
            let kernel = PrivateKernelInputsOrdering {
                previous_kernel: self.previous_kernel.finish(),
                sorted_new_commitments,
                sorted_new_commitments_indexes,
                read_commitment_hints: sorted_read_commitment_hints,
                sorted_new_nullifiers,
                sorted_new_nullifiers_indexes,
                nullifier_commitment_hints: sorted_nullifier_commitment_hints,
                master_nullifier_secret_keys: dep::std::unsafe::zeroed(),
            };
            kernel.native_private_kernel_circuit_ordering()
        }

        pub fn failed(self) {
            let _ = self.execute();
        }
    }

    #[test]
    unconstrained fn native_matching_one_read_request_to_commitment_works() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        builder.append_transient_commitments(1);
        builder.add_transient_read(0);

        let unique_siloed_commitments = builder.get_unique_siloed_commitments();

        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_commitments) == 1);
        assert(public_inputs.end.new_commitments[0].eq(unique_siloed_commitments[0]));
    }

    #[test]
    unconstrained fn native_matching_some_read_requests_to_commitments_works() {
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
            assert(public_inputs.end.new_commitments[i].eq(unique_siloed_commitments[i]));
        }
    }

    #[test(should_fail_with="Hinted commitment does not match read request")]
    unconstrained fn native_read_request_unknown_fails() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();
        builder.append_transient_commitments(1);
        builder.add_transient_read(0);
        // Tweak the read request so that it does not match the commitment at index 0;
        let read_request = builder.previous_kernel.end.read_requests.pop();
        builder.previous_kernel.end.read_requests.push(SideEffect { value: read_request.value + 1, counter: 0 });
        builder.failed();
    }

    #[test]
    unconstrained fn native_squash_one_of_one_transient_matches_works() {
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
    unconstrained fn native_squash_one_of_two_transient_matches_works() {
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
        assert(
            array_eq(
                public_inputs.end.new_commitments,
                [unique_siloed_commitments[0]]
            )
        );
        // The nullifier at index 1 is chopped.
        let expected_new_nullifiers = [new_nullifiers[0], new_nullifiers[2]];
        assert(array_eq(public_inputs.end.new_nullifiers, expected_new_nullifiers));
    }

    #[test]
    unconstrained fn native_squash_two_of_two_transient_matches_works() {
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
    unconstrained fn ordering_of_commitments_and_nullifiers() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();

        let mut sorted_new_commitments = [SideEffect::empty(); 10];
        let mut sorted_new_nullifiers = [SideEffectLinkedToNoteHash::empty(); 10];

        for i in 0..10 {
            sorted_new_commitments[i] = SideEffect { value: (i + 1) as Field, counter: i + 1 };
            sorted_new_nullifiers[i] = SideEffectLinkedToNoteHash { value: (i + 11) as Field, counter: i + 11, note_hash: 0 };
        }

        for i in 0..10 {
            builder.previous_kernel.end.new_commitments.push(sorted_new_commitments[9 - i]);
            builder.previous_kernel.end.new_nullifiers.push(sorted_new_nullifiers[9 - i]);
        }

        let public_inputs = builder.execute();

        let sorted_unique_commitments = compute_unique_siloed_commitments(
            public_inputs.end.new_nullifiers[0].value,
            sorted_new_commitments
        );

        for i in 0..10 {
            assert(public_inputs.end.new_commitments[i].eq(sorted_unique_commitments[i]));
            // +1 due to the 0th nullifier being the tx hash
            assert(public_inputs.end.new_nullifiers[i + 1].eq(sorted_new_nullifiers[i]));
        }
    }

    #[test]
    unconstrained fn native_empty_nullified_commitment_means_persistent_nullifier_0() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();
        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);
        let public_inputs = builder.execute();
        assert_eq(array_length(public_inputs.end.new_commitments), 2);
        assert_eq(array_length(public_inputs.end.new_nullifiers), 3);
    }
    // same as previous test, but this time there are 0 commitments!
    // (Do we really need this test?)

    #[test]
    unconstrained fn native_empty_nullified_commitment_means_persistent_nullifier_1() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();
        builder.append_nullifiers(2);
        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_commitments) == 0);
        assert(array_length(public_inputs.end.new_nullifiers) == 3);
    }

    #[test(should_fail)]
    unconstrained fn invalid_nullifier_commitment_hint_fails() {
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
    unconstrained fn wrong_nullifier_commitment_hint_fails() {
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
    unconstrained fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();
        builder.previous_kernel.push_private_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="The 0th nullifier in the accumulated nullifier array is zero")]
    unconstrained fn zero_0th_nullifier_fails() {
        let mut builder = PrivateKernelOrderingInputsBuilder::new();
        builder.previous_kernel.end.new_nullifiers = BoundedVec::new(SideEffectLinkedToNoteHash::empty());
        builder.failed();
    }
}
