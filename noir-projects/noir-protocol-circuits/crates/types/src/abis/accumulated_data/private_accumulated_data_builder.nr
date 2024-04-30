use crate::{
    hash::compute_tx_logs_hash,
    abis::{
    gas::Gas,
    accumulated_data::{
    combined_accumulated_data::CombinedAccumulatedData,
    private_accumulated_data::PrivateAccumulatedData, public_accumulated_data::PublicAccumulatedData,
    public_accumulated_data_builder::PublicAccumulatedDataBuilder
},
    call_request::CallRequest, note_hash::{NoteHash, NoteHashContext}, nullifier::Nullifier,
    public_data_update_request::PublicDataUpdateRequest, side_effect::SideEffect
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX,
    DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE
},
    traits::{Empty, is_empty}
};

// Builds via PrivateKernelCircuitPublicInputsBuilder:
// .finish: PrivateKernelCircuitPublicInputs.end
// .to_combined: KernelCircuitPublicInputs.end
// .split_to_public: PublicKernelCircuitPublicInputs.(end,end_non_revertible)
struct PrivateAccumulatedDataBuilder {
    new_note_hashes: BoundedVec<NoteHashContext, MAX_NEW_NOTE_HASHES_PER_TX>,
    new_nullifiers: BoundedVec<Nullifier, MAX_NEW_NULLIFIERS_PER_TX>,
    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_TX>,

    encrypted_logs_hashes: BoundedVec<SideEffect, MAX_ENCRYPTED_LOGS_PER_TX>,
    unencrypted_logs_hashes: BoundedVec<SideEffect, MAX_UNENCRYPTED_LOGS_PER_TX>,

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    private_call_stack: BoundedVec<CallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,

    gas_used: Gas,
    non_revertible_gas_used: Gas,
}

impl PrivateAccumulatedDataBuilder {
    pub fn finish(self) -> PrivateAccumulatedData {
        assert(self.gas_used.is_empty());
        assert(self.non_revertible_gas_used.is_empty());

        PrivateAccumulatedData {
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage
        }
    }

    pub fn to_combined(self, teardown_gas: Gas) -> CombinedAccumulatedData {
        // TODO(Miranda): Hash here or elsewhere?
        let encrypted_logs_hash = compute_tx_logs_hash(self.encrypted_logs_hashes.storage);
        let unencrypted_logs_hash = compute_tx_logs_hash(self.unencrypted_logs_hashes.storage);
        let gas_used = self.to_metered_gas_used() + Gas::tx_overhead() + teardown_gas;

        CombinedAccumulatedData {
            new_note_hashes: self.new_note_hashes.storage.map(|n: NoteHashContext| n.value),
            new_nullifiers: self.new_nullifiers.storage.map(|n: Nullifier| n.value),
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash,
            unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            public_data_update_requests: [PublicDataUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
            gas_used
        }
    }

    pub fn to_metered_gas_used(self) -> Gas {
        let mut metered_bytes = 0;

        // note_hash_gas
        for i in 0..self.new_note_hashes.storage.len() {
            if !is_empty(self.new_note_hashes.get_unchecked(i)) {
                metered_bytes += DA_BYTES_PER_FIELD;
            }
        }

        // nullifier_gas
        for i in 0..self.new_nullifiers.storage.len() {
            if !is_empty(self.new_nullifiers.get_unchecked(i)) {
                metered_bytes += DA_BYTES_PER_FIELD;
            }
        }

        // l2_to_l1_msg_gas
        for i in 0..self.new_l2_to_l1_msgs.storage.len() {
            if !is_empty(self.new_l2_to_l1_msgs.get_unchecked(i)) {
                metered_bytes += DA_BYTES_PER_FIELD;
            }
        }

        // encrypted_logs_hash_gas
        metered_bytes += self.encrypted_log_preimages_length as u32;

        // unencrypted_logs_hash_gas
        metered_bytes += self.unencrypted_log_preimages_length as u32;

        Gas::new(DA_GAS_PER_BYTE * metered_bytes, 0)
    }

    pub fn split_to_public(
        self,
        min_revertible_side_effect_counter: u32,
        teardown_gas: Gas
    ) -> (PublicAccumulatedData, PublicAccumulatedData) {
        let mut non_revertible_builder = PublicAccumulatedDataBuilder::empty();
        let mut revertible_builder = PublicAccumulatedDataBuilder::empty();
        let mut non_revertible_da_gas_used = 0;
        let mut non_revertible_l2_gas_used = 0;
        let mut revertible_da_gas_used = teardown_gas.da_gas; // pre-pay for teardown gas
        let mut revertible_l2_gas_used = teardown_gas.l2_gas;
        let DA_GAS_PER_FIELD = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;

        for i in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            let note_hash = self.new_note_hashes.storage[i];
            let public_note_hash = note_hash.expose_to_public();
            if note_hash.counter < min_revertible_side_effect_counter {
                non_revertible_builder.new_note_hashes.push(public_note_hash);
                if !is_empty(public_note_hash) {
                    non_revertible_da_gas_used += DA_GAS_PER_FIELD ;
                }
            } else {
                revertible_builder.new_note_hashes.push(public_note_hash);
                if !is_empty(public_note_hash) {
                    revertible_da_gas_used += DA_GAS_PER_FIELD;
                }
            }
        }

        for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
            let nullifier = self.new_nullifiers.storage[i];
            if nullifier.counter < min_revertible_side_effect_counter {
                non_revertible_builder.new_nullifiers.push(nullifier);
                if !is_empty(nullifier) {
                    non_revertible_da_gas_used += DA_GAS_PER_FIELD;
                }
            } else {
                revertible_builder.new_nullifiers.push(nullifier);
                if !is_empty(nullifier) {
                    revertible_da_gas_used += DA_GAS_PER_FIELD;
                }
            }
        }

        // TODO(gas): add AVM_STARTUP_L2_GAS here
        for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX {
            let call_stack_item = self.public_call_stack.storage[i];
            if call_stack_item.start_side_effect_counter < min_revertible_side_effect_counter {
                non_revertible_builder.public_call_stack.push(call_stack_item);
            } else {
                revertible_builder.public_call_stack.push(call_stack_item);
            }
        }

        for i in 0..MAX_ENCRYPTED_LOGS_PER_TX {
            let encrypted_logs_hash = self.encrypted_logs_hashes.storage[i];
            if encrypted_logs_hash.counter < min_revertible_side_effect_counter {
                non_revertible_builder.encrypted_logs_hashes.push(encrypted_logs_hash);
            } else {
                revertible_builder.encrypted_logs_hashes.push(encrypted_logs_hash);
            }
        }

        for i in 0..MAX_UNENCRYPTED_LOGS_PER_TX {
            let unencrypted_logs_hash = self.unencrypted_logs_hashes.storage[i];
            if unencrypted_logs_hash.counter < min_revertible_side_effect_counter {
                non_revertible_builder.unencrypted_logs_hashes.push(unencrypted_logs_hash);
            } else {
                revertible_builder.unencrypted_logs_hashes.push(unencrypted_logs_hash);
            }
        }

        revertible_builder.new_l2_to_l1_msgs =  self.new_l2_to_l1_msgs;
        // TODO(1165): Once we have individual lens, split here
        revertible_builder.encrypted_log_preimages_length =  self.encrypted_log_preimages_length;
        revertible_builder.unencrypted_log_preimages_length = self.unencrypted_log_preimages_length;

        revertible_da_gas_used += DA_GAS_PER_BYTE * (self.encrypted_log_preimages_length as u32 + self.unencrypted_log_preimages_length as u32);

        revertible_builder.gas_used = Gas::new(revertible_da_gas_used, revertible_l2_gas_used);
        non_revertible_builder.gas_used = Gas::tx_overhead() + Gas::new(non_revertible_da_gas_used, non_revertible_l2_gas_used);
        (non_revertible_builder.finish(), revertible_builder.finish())
    }
}

mod tests {
    use crate::{
        abis::{
        accumulated_data::private_accumulated_data_builder::PrivateAccumulatedDataBuilder, gas::Gas,
        call_request::CallRequest, caller_context::CallerContext,
        note_hash::{NoteHash, NoteHashContext}, nullifier::Nullifier,
        public_data_update_request::PublicDataUpdateRequest, side_effect::SideEffect
    },
        address::AztecAddress, utils::arrays::array_eq, constants::{DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE}
    };

    #[test]
    unconstrained fn splits_revertible_and_non_revertible() {
        let mut builder = PrivateAccumulatedDataBuilder::empty();

        let non_revertible_note_hashes = [
            NoteHashContext { value: 1, counter: 1, nullifier_counter: 20 },
            NoteHashContext { value: 2, counter: 3, nullifier_counter: 5 }
        ];

        let non_revertible_nullifiers = [
            Nullifier { value: 10, note_hash: 1, counter: 2 },
            Nullifier { value: 20, note_hash: 2, counter: 4 }
        ];

        let non_revertible_public_stack = [
            CallRequest {
                hash: 1,
                caller_contract_address: AztecAddress::from_field(1),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 5,
                end_side_effect_counter: 0
            },
            CallRequest {
                hash: 2,
                caller_contract_address: AztecAddress::from_field(1),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 6,
                end_side_effect_counter: 0
            }
        ];

        let revertible_note_hashes = [
            NoteHashContext { value: 3, counter: 7, nullifier_counter: 15 },
            NoteHashContext { value: 4, counter: 10, nullifier_counter: 0 }
        ];

        let revertible_nullifiers = [
            Nullifier { value: 30, note_hash: 3, counter: 8 },
            Nullifier { value: 40, note_hash: 4, counter: 11 }
        ];

        let revertible_public_call_stack = [
            CallRequest {
                hash: 3,
                caller_contract_address: AztecAddress::from_field(3),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 9,
                end_side_effect_counter: 0
            }
        ];

        builder.new_note_hashes.extend_from_array(non_revertible_note_hashes);
        builder.new_note_hashes.extend_from_array(revertible_note_hashes);

        builder.new_nullifiers.extend_from_array(non_revertible_nullifiers);
        builder.new_nullifiers.extend_from_array(revertible_nullifiers);

        builder.public_call_stack.extend_from_array(non_revertible_public_stack);
        builder.public_call_stack.extend_from_array(revertible_public_call_stack);

        let (non_revertible, revertible) = builder.split_to_public(7, Gas::new(42, 17));

        assert(
            array_eq(
                non_revertible.new_note_hashes,
                [
            NoteHash { value: 1, counter: 0 },
            NoteHash { value: 2, counter: 0 }
        ]
            )
        );
        assert(array_eq(non_revertible.new_nullifiers, non_revertible_nullifiers));
        assert(array_eq(non_revertible.public_call_stack, non_revertible_public_stack));

        assert(
            array_eq(
                revertible.new_note_hashes,
                [
            NoteHash { value: 3, counter: 0 },
            NoteHash { value: 4, counter: 0 }
        ]
            )
        );
        assert(array_eq(revertible.new_nullifiers, revertible_nullifiers));
        assert(array_eq(revertible.public_call_stack, revertible_public_call_stack));

        assert_eq(
            revertible.gas_used, Gas::new(4 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE, 0) + Gas::new(42, 17)
        );

        print(non_revertible.gas_used);
        assert_eq(
            non_revertible.gas_used, Gas::new(4 * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE, 0) + Gas::tx_overhead()
        );
    }
}

impl Empty for PrivateAccumulatedDataBuilder {
    fn empty() -> Self {
        PrivateAccumulatedDataBuilder {
            new_note_hashes: BoundedVec::new(),
            new_nullifiers: BoundedVec::new(),
            new_l2_to_l1_msgs: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            private_call_stack: BoundedVec::new(),
            public_call_stack: BoundedVec::new(),
            gas_used: Gas::empty(),
            non_revertible_gas_used: Gas::empty(),
        }
    }
}
