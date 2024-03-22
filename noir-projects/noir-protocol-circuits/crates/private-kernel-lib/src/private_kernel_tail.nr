use crate::common;
use dep::std::{cmp::Eq, option::Option, unsafe};
use dep::reset_kernel_lib::{NullifierReadRequestHints, reset_read_requests};
use dep::types::{
    abis::{
    call_request::CallRequest, nullifier_key_validation_request::NullifierKeyValidationRequestContext,
    kernel_data::PrivateKernelInnerData,
    kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputsBuilder, PrivateKernelTailCircuitPublicInputs},
    side_effect::{SideEffect, SideEffectLinkedToNoteHash, Ordered}
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX
},
    grumpkin_private_key::GrumpkinPrivateKey,
    hash::{compute_note_hash_nonce, compute_unique_siloed_note_hash},
    utils::{arrays::{array_length, array_eq}}, traits::{Empty, is_empty}
};

struct PrivateKernelTailCircuitPrivateInputs {
    previous_kernel: PrivateKernelInnerData,
    sorted_new_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX],
    sorted_new_note_hashes_indexes: [u64; MAX_NEW_NOTE_HASHES_PER_TX],
    read_commitment_hints: [u64; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    sorted_new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],
    sorted_new_nullifiers_indexes: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    nullifier_read_request_hints: NullifierReadRequestHints,
    nullifier_commitment_hints: [u64; MAX_NEW_NULLIFIERS_PER_TX],
    master_nullifier_secret_keys: [GrumpkinPrivateKey; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
}

impl PrivateKernelTailCircuitPrivateInputs {
    fn validate_inputs(self) {
        assert_eq(
            array_length(self.previous_kernel.public_inputs.end.private_call_stack), 0, "Private call stack must be empty when executing the tail circuit"
        );
    }

    fn validate_nullifier_read_requests(self, public_inputs: &mut PrivateKernelCircuitPublicInputsBuilder) {
        let requests = self.previous_kernel.public_inputs.validation_requests.nullifier_read_requests;

        let pending_nullifiers = self.previous_kernel.public_inputs.end.new_nullifiers;

        let hints = self.nullifier_read_request_hints;

        let nullifier_tree_root = public_inputs.constants.historical_header.state.partial.nullifier_tree.root;

        public_inputs.validation_requests.nullifier_read_requests = reset_read_requests(
            requests,
            pending_nullifiers,
            hints.read_request_statuses,
            hints.pending_read_hints,
            hints.settled_read_hints,
            nullifier_tree_root,
        );
        // When we have a separate reset circuit, we can allow unverified requests and process them later after the 
        // corresponding values are added to public inputs in nested executions.
        // But right now, all the request must be cleared in one go.
        assert(
            public_inputs.validation_requests.nullifier_read_requests.len() == 0, "All nullifier read requests must be verified"
        );
    }

    fn validate_nullifier_keys(self, public_inputs: &mut PrivateKernelCircuitPublicInputsBuilder) {
        let requests = self.previous_kernel.public_inputs.validation_requests.nullifier_key_validation_requests;
        for i in 0..MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX {
            let request = requests[i];
            if !is_empty(request) {
                let master_secret_key = self.master_nullifier_secret_keys[i];
                let computed_public_key = master_secret_key.derive_public_key();
                assert(
                    computed_public_key.eq(request.public_key), "Cannot derive nullifier public key from the master key."
                );

                let computed_secret_key = common::compute_siloed_nullifier_secret_key(master_secret_key, request.contract_address);
                assert(
                    computed_secret_key.eq(request.secret_key), "Cannot derive siloed secret key from the master key."
                );
            }
        }

        // Empty out nullifier key validation requests after verifying them.
        public_inputs.validation_requests.nullifier_key_validation_requests = BoundedVec::new();
    }

    fn match_reads_to_commitments(self, public_inputs: &mut PrivateKernelCircuitPublicInputsBuilder) {
        let new_note_hashes = public_inputs.end.new_note_hashes;
        let read_requests = public_inputs.validation_requests.note_hash_read_requests;

        // match reads to commitments from the previous call(s)
        for rr_idx in 0..MAX_NOTE_HASH_READ_REQUESTS_PER_TX {
            let read_request = read_requests.get_unchecked(rr_idx);
            let read_commitment_hint = self.read_commitment_hints[rr_idx];

            if (read_request.value != 0) {
                let hash = new_note_hashes.get_unchecked(read_commitment_hint);
                assert_eq(read_request.value, hash.value, "Hinted hash does not match read request");
                assert(
                    read_request.counter > hash.counter, "Read request counter must be greater than hash counter"
                );
            }
        }

        // Empty out read requests after matching them to commitments
        public_inputs.validation_requests.note_hash_read_requests = BoundedVec::new();
    }

    fn assert_sorted_counters<T, N>(original: [T; N], sorted: [T; N], indexes: [u64; N]) where T: Eq + Ordered + Empty {
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

    fn sort_arrays(self, public_inputs: &mut PrivateKernelCircuitPublicInputsBuilder) {
        PrivateKernelTailCircuitPrivateInputs::assert_sorted_counters(
            public_inputs.end.new_note_hashes.storage,
            self.sorted_new_note_hashes,
            self.sorted_new_note_hashes_indexes
        );
        PrivateKernelTailCircuitPrivateInputs::assert_sorted_counters(
            public_inputs.end.new_nullifiers.storage,
            self.sorted_new_nullifiers,
            self.sorted_new_nullifiers_indexes
        );
        public_inputs.end.new_note_hashes.storage = self.sorted_new_note_hashes;
        public_inputs.end.new_nullifiers.storage = self.sorted_new_nullifiers;
    }

    fn match_nullifiers_to_note_hashes_and_squash(self, public_inputs: &mut PrivateKernelCircuitPublicInputsBuilder) {
        // Remark: The commitments in public_inputs.end have already been siloed by contract address!
        // Match nullifiers/nullified_commitments to commitments from the previous call(s)
        let mut new_note_hashes = public_inputs.end.new_note_hashes.storage;
        let mut new_nullifiers = public_inputs.end.new_nullifiers.storage;

        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            let nullifier = new_nullifiers[n_idx];
            // TODO - should not be able to squash the first nullifier.
            let nullified_note_hash = nullifier.note_hash;
            let hint_pos = self.nullifier_commitment_hints[n_idx];

            // Nullified_commitment of value `0` implies non-transient (persistable)
            // nullifier in which case no attempt will be made to match it to a hash.
            // Non-empty nullified_note_hash implies transient nullifier which MUST be matched to a hash below!
            // 0-valued nullified_note_hash is empty and will be ignored
            if nullified_note_hash != 0 {
                assert(
                    hint_pos < MAX_NEW_NOTE_HASHES_PER_TX, "New nullifier is transient but hint is invalid"
                );
                let hash = new_note_hashes[hint_pos];
                assert_eq(nullified_note_hash, hash.value, "Hinted hash does not match");
                assert(
                    nullifier.counter > hash.counter, "Nullifier counter must be greater than hash counter"
                );
                // match found!
                // squash both the nullifier and the hash
                // (set to 0 here and then rearrange array after loop)
                new_note_hashes[hint_pos] = SideEffect::empty();
                new_nullifiers[n_idx] = SideEffectLinkedToNoteHash::empty();
            }
            // non-transient (persistable) nullifiers are just kept in new_nullifiers array and forwarded
            // to public inputs (used later by base rollup circuit)
        }

        // Move all zero-ed (removed) entries of these arrays to the end and preserve ordering of other entries

        let mut new_note_hashes_vec = BoundedVec::new();

        for c_idx in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            if new_note_hashes[c_idx].value != 0 {
                new_note_hashes_vec.push(new_note_hashes[c_idx]);
            }
        }

        public_inputs.end.new_note_hashes = new_note_hashes_vec;

        let mut new_nullifiers_vec = BoundedVec::new();

        for n_idx in 0..MAX_NEW_NULLIFIERS_PER_TX {
            if new_nullifiers[n_idx].value != 0 {
                new_nullifiers_vec.push(new_nullifiers[n_idx]);
            }
        }

        public_inputs.end.new_nullifiers = new_nullifiers_vec;
    }

    fn apply_note_hash_nonces(public_inputs: &mut PrivateKernelCircuitPublicInputsBuilder) {
        // Remark: The commitments in public_inputs.end have already been siloed by contract address!
        // tx hash
        let first_nullifier = public_inputs.end.new_nullifiers.get(0);
        let mut siloed_note_hashes = public_inputs.end.new_note_hashes.storage;

        for c_idx in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            // Apply nonce to all non-zero/non-empty note hashes
            // Nonce is the hash of the first (0th) nullifier and the note hash's index into new_note_hashes array
            let nonce = compute_note_hash_nonce(first_nullifier.value, c_idx);
            let hash = siloed_note_hashes[c_idx];
            if hash.value != 0 {
                let unique_note_hash = compute_unique_siloed_note_hash(nonce, hash.value);
                siloed_note_hashes[c_idx] = SideEffect{
                    value: unique_note_hash,
                    counter: hash.counter
                };
            }
        }

        public_inputs.end.new_note_hashes.storage = siloed_note_hashes;
    }

    pub fn native_private_kernel_circuit_tail(self) -> PrivateKernelTailCircuitPublicInputs {
        let mut public_inputs : PrivateKernelCircuitPublicInputsBuilder = unsafe::zeroed();
        public_inputs.is_private = true;

        self.validate_inputs();

        common::validate_previous_kernel_values(self.previous_kernel.public_inputs.end);

        // Do this before any functions can modify the inputs.
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        self.validate_nullifier_read_requests(&mut public_inputs);

        self.validate_nullifier_keys(&mut public_inputs);

        self.sort_arrays(&mut public_inputs);

        self.match_reads_to_commitments(&mut public_inputs);

        self.match_nullifiers_to_note_hashes_and_squash(&mut public_inputs);

        PrivateKernelTailCircuitPrivateInputs::apply_note_hash_nonces(&mut public_inputs);

        public_inputs.to_tail()
    }
}

mod tests {
    use dep::std::{cmp::Eq, unsafe};
    use crate::{private_kernel_tail::PrivateKernelTailCircuitPrivateInputs};
    use dep::reset_kernel_lib::{
        tests::nullifier_read_request_hints_builder::NullifierReadRequestHintsBuilder,
        read_request_reset::{PendingReadHint, ReadRequestState, ReadRequestStatus}
    };
    use dep::types::constants::{
        MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX,
        MAX_REVERTIBLE_NOTE_HASHES_PER_TX
    };
    use dep::types::{
        abis::{
        kernel_circuit_public_inputs::PrivateKernelTailCircuitPublicInputs,
        max_block_number::MaxBlockNumber, side_effect::{SideEffect, SideEffectLinkedToNoteHash, Ordered}
    },
        hash::compute_unique_siloed_note_hashes, tests::kernel_data_builder::PreviousKernelDataBuilder,
        utils::{arrays::{array_eq, array_length}}, traits::{Empty, is_empty, is_empty_array}
    };

    struct PrivateKernelTailInputsBuilder {
        previous_kernel: PreviousKernelDataBuilder,
        read_commitment_hints: [u64; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
        nullifier_commitment_hints: [u64; MAX_NEW_NULLIFIERS_PER_TX],
        nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder,
    }

    impl PrivateKernelTailInputsBuilder {
        pub fn new() -> Self {
            PrivateKernelTailInputsBuilder {
                previous_kernel: PreviousKernelDataBuilder::new(false),
                read_commitment_hints: [0; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
                nullifier_commitment_hints: [0; MAX_NEW_NULLIFIERS_PER_TX],
                nullifier_read_request_hints_builder: NullifierReadRequestHintsBuilder::new(MAX_NULLIFIER_READ_REQUESTS_PER_TX)
            }
        }

        pub fn get_new_note_hashes(self) -> [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX] {
            self.previous_kernel.end.new_note_hashes.storage
        }

        pub fn get_new_nullifiers(self) -> [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX] {
            self.previous_kernel.end.new_nullifiers.storage
        }

        pub fn get_unique_siloed_note_hashes(self) -> [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX] {
            self.compute_unique_siloed_note_hashes(self.previous_kernel.end.new_note_hashes.storage)
        }

        // A helper function that uses the first nullifer in the previous kernel to compute the unique siloed
        // commitments for the given commitments.
        pub fn compute_unique_siloed_note_hashes<N>(self, commitments: [SideEffect; N]) -> [SideEffect; N] {
            let first_nullifier = self.previous_kernel.end.new_nullifiers.get_unchecked(0);
            compute_unique_siloed_note_hashes(first_nullifier.value, commitments)
        }

        pub fn append_transient_commitments(&mut self, num_commitments: u64) {
            // All new note hashes aggregated in the previous kernel are transient commitments.
            self.previous_kernel.append_new_note_hashes(num_commitments);
        }

        pub fn add_transient_read(&mut self, commitment_index: u64) {
            let read_request_index = self.previous_kernel.add_read_request_for_transient_commitment(commitment_index);
            self.read_commitment_hints[read_request_index] = commitment_index;
        }

        pub fn append_nullifiers(&mut self, num_nullifiers: u64) {
            self.previous_kernel.append_new_nullifiers_from_private(num_nullifiers);
        }

        pub fn add_nullifier_pending_read(&mut self, nullifier_index_offset_one: u64) {
            let nullifier_index = nullifier_index_offset_one + 1; // + 1 is for the first nullifier
            let read_request_index = self.previous_kernel.add_read_request_for_pending_nullifier(nullifier_index);
            let hint_index = self.nullifier_read_request_hints_builder.pending_read_hints.len();
            let hint = PendingReadHint { read_request_index, pending_value_index: nullifier_index };
            self.nullifier_read_request_hints_builder.pending_read_hints.push(hint);
            self.nullifier_read_request_hints_builder.read_request_statuses[read_request_index] = ReadRequestStatus { state: ReadRequestState.PENDING, hint_index };
        }

        pub fn nullify_transient_commitment(&mut self, nullifier_index: Field, commitment_index: u64) {
            self.previous_kernel.end.new_nullifiers.storage[nullifier_index].note_hash = self.previous_kernel.end.new_note_hashes.get(commitment_index).value;
            self.nullifier_commitment_hints[nullifier_index] = commitment_index;
        }

        fn sort_sideffects<T, N>(original: [T; N]) -> ([T; N], [u64; N]) where T: Ordered + Eq + Empty {
            let mut indexes = [0; N];
            for i in 0..N {
                indexes[i] = i;
            }
            let sorted_indexes = indexes.sort_via(
                |a_index: u64, b_index: u64| {
                let a = original[a_index];
                let b = original[b_index];
                if is_empty(b) {
                    true
                } else if is_empty(a) {
                    false
                } else {
                    a.counter() < b.counter()
                }
            }
            );
            let sorted_sideffects = sorted_indexes.map(|i: u64| original[i]);
            let mut reverse_map = [0; N];
            for i in 0..N {
                reverse_map[sorted_indexes[i]] = i;
            }

            (sorted_sideffects, reverse_map)
        }

        pub fn execute(&mut self) -> PrivateKernelTailCircuitPublicInputs {
            let (sorted_new_note_hashes, sorted_new_note_hashes_indexes) = PrivateKernelTailInputsBuilder::sort_sideffects(self.get_new_note_hashes());
            let mut sorted_read_commitment_hints = [0; MAX_NOTE_HASH_READ_REQUESTS_PER_TX];
            for i in 0..sorted_read_commitment_hints.len() {
                sorted_read_commitment_hints[i] = sorted_new_note_hashes_indexes[self.read_commitment_hints[i]];
            }
            let (sorted_new_nullifiers, sorted_new_nullifiers_indexes) = PrivateKernelTailInputsBuilder::sort_sideffects(self.get_new_nullifiers());
            let mut sorted_nullifier_commitment_hints = [0; MAX_NEW_NULLIFIERS_PER_TX];
            for i in 0..sorted_nullifier_commitment_hints.len() {
                sorted_nullifier_commitment_hints[i] = sorted_new_nullifiers_indexes[self.nullifier_commitment_hints[i]];
            }

            let kernel = PrivateKernelTailCircuitPrivateInputs {
                previous_kernel: self.previous_kernel.to_private_kernel_inner_data(),
                sorted_new_note_hashes,
                sorted_new_note_hashes_indexes,
                read_commitment_hints: sorted_read_commitment_hints,
                sorted_new_nullifiers,
                sorted_new_nullifiers_indexes,
                nullifier_read_request_hints: self.nullifier_read_request_hints_builder.to_hints(),
                nullifier_commitment_hints: sorted_nullifier_commitment_hints,
                master_nullifier_secret_keys: unsafe::zeroed()
            };
            kernel.native_private_kernel_circuit_tail()
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }
    }

    #[test]
    unconstrained fn splits_tx_nullifier_to_non_revertible() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end_non_revertible.new_nullifiers) == 1);
        assert(array_length(public_inputs.end.new_nullifiers) == 0);
    }

    #[test]
    unconstrained fn native_matching_one_read_request_to_commitment_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.append_transient_commitments(1);
        builder.add_transient_read(0);

        let unique_siloed_note_hashes = builder.get_unique_siloed_note_hashes();

        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_note_hashes) == 1);
        assert(public_inputs.end.new_note_hashes[0].eq(unique_siloed_note_hashes[0]));
    }

    #[test]
    unconstrained fn native_matching_some_read_requests_to_commitments_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_transient_commitments(MAX_NEW_NOTE_HASHES_PER_TX);
        // prepare for the split: first MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX are added to end_non_revertible_accumulted_data
        // neeed to take the counter of the side effect at the given index because
        builder.previous_kernel.min_revertible_side_effect_counter = builder.previous_kernel.end.new_note_hashes.get(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX).counter;
        // Read the hash at index 1;
        builder.add_transient_read(1);
        // Read the hash at index 3;
        builder.add_transient_read(3);
        let unique_siloed_note_hashes = builder.get_unique_siloed_note_hashes();
        let public_inputs = builder.execute();
        assert_eq(array_length(public_inputs.end.new_note_hashes), MAX_REVERTIBLE_NOTE_HASHES_PER_TX);
        for i in 0..MAX_REVERTIBLE_NOTE_HASHES_PER_TX {
            assert(
                public_inputs.end.new_note_hashes[i].eq(unique_siloed_note_hashes[MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX + i])
            );
        }
    }

    #[test(should_fail_with="Hinted hash does not match read request")]
    unconstrained fn native_read_request_unknown_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_transient_commitments(1);
        builder.add_transient_read(0);
        // Tweak the read request so that it does not match the hash at index 0;
        let read_request = builder.previous_kernel.validation_requests.note_hash_read_requests.pop();
        builder.previous_kernel.validation_requests.note_hash_read_requests.push(SideEffect { value: read_request.value + 1, counter: 0 });
        builder.failed();
    }

    #[test]
    fn propagate_previous_kernel_max_block_number() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.validation_requests.max_block_number = MaxBlockNumber::new(13);
        let public_inputs = builder.execute();

        assert_eq(public_inputs.rollup_validation_requests.max_block_number.unwrap(), 13);
    }

    #[test]
    unconstrained fn one_pending_nullifier_read_request() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);

        builder.succeeded();
    }

    #[test]
    unconstrained fn two_pending_nullifier_read_requests() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        builder.add_nullifier_pending_read(0);

        builder.succeeded();
    }

    #[test(should_fail_with="Hinted value does not match read request")]
    unconstrained fn pending_nullifier_read_request_wrong_hint_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        let mut hint = builder.nullifier_read_request_hints_builder.pending_read_hints.pop();
        assert(hint.pending_value_index == 2);
        hint.pending_value_index = 1;
        builder.nullifier_read_request_hints_builder.pending_read_hints.push(hint);

        builder.failed();
    }

    #[test(should_fail_with="Read request counter must be greater than counter of the value being read")]
    unconstrained fn pending_nullifier_read_request_reads_before_value_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        builder.append_nullifiers(3);
        builder.add_nullifier_pending_read(1);
        let nullifier_being_read = builder.get_new_nullifiers()[2];
        let mut read_request = builder.previous_kernel.validation_requests.nullifier_read_requests.pop();
        read_request.counter = nullifier_being_read.counter - 1;
        builder.previous_kernel.validation_requests.nullifier_read_requests.push(read_request);

        builder.failed();
    }

    #[test]
    unconstrained fn native_squash_one_of_one_transient_matches_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_transient_commitments(1);
        builder.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_transient_commitment(1, 0);
        let new_nullifiers = builder.get_new_nullifiers();
        let public_inputs = builder.execute();
        assert(is_empty_array(public_inputs.end.new_note_hashes));

        // The nullifier at index 1 is chopped.
        assert(array_eq(public_inputs.end.new_nullifiers, [new_nullifiers[2]]));
        // tx nullifier is part of non revertible accumulated data
        assert(
            array_eq(
                public_inputs.end_non_revertible.new_nullifiers,
                [new_nullifiers[0]]
            )
        );
    }

    #[test]
    unconstrained fn native_squash_one_of_two_transient_matches_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_transient_commitment(1, 0);
        let new_note_hashes = builder.get_new_note_hashes();
        // The 0th hash will be chopped.
        let unique_siloed_note_hashes = builder.compute_unique_siloed_note_hashes([new_note_hashes[1]]);
        let new_nullifiers = builder.get_new_nullifiers();
        let public_inputs = builder.execute();
        assert(
            array_eq(
                public_inputs.end.new_note_hashes,
                [unique_siloed_note_hashes[0]]
            )
        );
        // The nullifier at index 1 is chopped.
        assert(array_eq(public_inputs.end.new_nullifiers, [new_nullifiers[2]]));
        // tx nullifier is part of non revertible accumulated data
        assert(
            array_eq(
                public_inputs.end_non_revertible.new_nullifiers,
                [new_nullifiers[0]]
            )
        );
    }

    #[test]
    unconstrained fn native_squash_two_of_two_transient_matches_works() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 1;
        builder.nullify_transient_commitment(1, 1);
        // The nullifier at index 2 is nullifying the hash at index 0;
        builder.nullify_transient_commitment(2, 0);
        let new_nullifiers = builder.get_new_nullifiers();
        let public_inputs = builder.execute();

        // app logic will be completely empty after squashing
        assert(is_empty_array(public_inputs.end.new_note_hashes));
        assert(is_empty_array(public_inputs.end.new_nullifiers));

        // and the 0th nullifier will be moved to the non-revertible array
        assert(
            array_eq(
                public_inputs.end_non_revertible.new_nullifiers,
                [new_nullifiers[0]]
            )
        );
    }

    #[test]
    unconstrained fn ordering_of_commitments_and_nullifiers() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        let mut sorted_new_note_hashes = [SideEffect::empty(); 10];
        let mut sorted_new_nullifiers = [SideEffectLinkedToNoteHash::empty(); 10];

        for i in 0..10 {
            sorted_new_note_hashes[i] = SideEffect { value: (i + 1) as Field, counter: builder.previous_kernel.next_sideffect_counter() };
            sorted_new_nullifiers[i] = SideEffectLinkedToNoteHash { value: (i + 11) as Field, counter: builder.previous_kernel.next_sideffect_counter(), note_hash: 0 };
        }

        for i in 0..10 {
            builder.previous_kernel.end.new_note_hashes.push(sorted_new_note_hashes[9 - i]);
            builder.previous_kernel.end.new_nullifiers.push(sorted_new_nullifiers[9 - i]);
        }

        let public_inputs = builder.execute();

        let sorted_unique_note_hashes = compute_unique_siloed_note_hashes(
            // tx nullifier is part of non revertible accumulated data
            public_inputs.end_non_revertible.new_nullifiers[0].value,
            sorted_new_note_hashes
        );

        for i in 0..10 {
            assert(public_inputs.end.new_note_hashes[i].eq(sorted_unique_note_hashes[i]));
            assert(public_inputs.end.new_nullifiers[i].eq(sorted_new_nullifiers[i]));
        }
    }

    #[test]
    unconstrained fn native_empty_nullified_commitment_means_persistent_nullifier_0() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);
        let public_inputs = builder.execute();
        assert_eq(array_length(public_inputs.end.new_note_hashes), 2);
        assert_eq(array_length(public_inputs.end.new_nullifiers), 2);
        assert_eq(array_length(public_inputs.end_non_revertible.new_nullifiers), 1);
    }
    // same as previous test, but this time there are 0 commitments!
    // (Do we really need this test?)

    #[test]
    unconstrained fn native_empty_nullified_commitment_means_persistent_nullifier_1() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_nullifiers(2);
        let public_inputs = builder.execute();
        assert(array_length(public_inputs.end.new_note_hashes) == 0);
        assert(array_length(public_inputs.end.new_nullifiers) == 2);
        assert_eq(array_length(public_inputs.end_non_revertible.new_nullifiers), 1);
    }

    #[test(should_fail)]
    unconstrained fn invalid_nullifier_commitment_hint_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_transient_commitments(1);
        builder.append_nullifiers(1);
        // The nullifier at index 1 is nullifying the hash at index 0;
        builder.nullify_transient_commitment(1, 0);
        // Change the hint to be out of bounds.
        builder.nullifier_commitment_hints[1] = MAX_NEW_NOTE_HASHES_PER_TX;
        builder.failed();
    }

    #[test(should_fail_with="Hinted hash does not match")]
    unconstrained fn wrong_nullifier_commitment_hint_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.append_transient_commitments(2);
        builder.append_nullifiers(2);
        // The nullifier at index 1 is nullifying the hash at index 1;
        builder.nullify_transient_commitment(1, 1);
        // The nullifier at index 2 is nullifying the hash at index 0;
        builder.nullify_transient_commitment(2, 0);
        // Tweak the hint to be for the hash at index 1.
        builder.nullifier_commitment_hints[2] = 1;
        builder.failed();
    }

    #[test(should_fail_with="Private call stack must be empty when executing the tail circuit")]
    unconstrained fn non_empty_private_call_stack_should_fail() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.push_private_call_request(1, false);
        builder.failed();
    }

    #[test(should_fail_with="The 0th nullifier in the accumulated nullifier array is zero")]
    unconstrained fn zero_0th_nullifier_fails() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        builder.previous_kernel.end.new_nullifiers = BoundedVec::new();
        builder.failed();
    }

    #[test]
    unconstrained fn split_nullifiers_into_non_revertible() {
        let mut builder = PrivateKernelTailInputsBuilder::new();
        // expect 3 non-revertible nullifiers: the tx nullifier + 2 new ones
        builder.previous_kernel.append_new_nullifiers_from_private(2);
        builder.previous_kernel.capture_min_revertible_side_effect_counter();

        // expect 2 revertible nullifiers
        builder.previous_kernel.append_new_nullifiers_from_private(2);

        let new_nullifiers = builder.previous_kernel.end.new_nullifiers.storage;
        let public_inputs = builder.execute();

        assert(
            array_eq(
                public_inputs.end_non_revertible.new_nullifiers,
                [new_nullifiers[0], new_nullifiers[1], new_nullifiers[2]]
            )
        );

        assert(
            array_eq(
                public_inputs.end.new_nullifiers,
                [new_nullifiers[3], new_nullifiers[4]]
            )
        );
    }

    #[test]
    unconstrained fn split_commitments_into_non_revertible() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        // expect 2 non-revertible commitments
        builder.previous_kernel.append_new_note_hashes(2);
        builder.previous_kernel.capture_min_revertible_side_effect_counter();

        // expect 2 revertible commitments
        builder.previous_kernel.append_new_note_hashes(2);

        let new_note_hashes = builder.previous_kernel.end.new_note_hashes.storage;
        let public_inputs = builder.execute();

        let siloed_note_hashes = compute_unique_siloed_note_hashes(
            // tx nullifier is part of non revertible accumulated data
            public_inputs.end_non_revertible.new_nullifiers[0].value,
            new_note_hashes
        );

        assert(
            array_eq(
                public_inputs.end_non_revertible.new_note_hashes,
                [siloed_note_hashes[0], siloed_note_hashes[1]]
            )
        );

        assert(
            array_eq(
                public_inputs.end.new_note_hashes,
                [siloed_note_hashes[2], siloed_note_hashes[3]]
            )
        );
    }

    #[test]
    unconstrained fn split_side_effect_squashing() {
        let mut builder = PrivateKernelTailInputsBuilder::new();

        // add one hash in non-revertible part
        builder.previous_kernel.append_new_note_hashes(1);
        builder.previous_kernel.capture_min_revertible_side_effect_counter();

        // nullify it in revertible part
        builder.previous_kernel.append_new_nullifiers_from_private(1);
        builder.nullify_transient_commitment(1, 0);

        let public_inputs = builder.execute();

        assert(!is_empty_array(public_inputs.end_non_revertible.new_nullifiers));
        assert(is_empty_array(public_inputs.end_non_revertible.new_note_hashes));
        assert(is_empty_array(public_inputs.end.new_note_hashes));
        assert(is_empty_array(public_inputs.end.new_nullifiers));
    }
}
