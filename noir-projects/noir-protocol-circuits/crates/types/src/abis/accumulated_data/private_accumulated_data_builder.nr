use crate::{
    abis::{
    accumulated_data::{
    combined_accumulated_data::CombinedAccumulatedData,
    private_accumulated_data::PrivateAccumulatedData, public_accumulated_data::PublicAccumulatedData,
    public_accumulated_data_builder::PublicAccumulatedDataBuilder
},
    call_request::CallRequest, public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
}
};
use dep::std::unsafe;

struct PrivateAccumulatedDataBuilder {
    new_note_hashes: BoundedVec<SideEffect, MAX_NEW_NOTE_HASHES_PER_TX>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NEW_NULLIFIERS_PER_TX>,
    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_TX>,

    encrypted_logs_hash: Field,
    unencrypted_logs_hash: Field,

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    private_call_stack: BoundedVec<CallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
}

impl PrivateAccumulatedDataBuilder {
    pub fn finish(self) -> PrivateAccumulatedData {
        PrivateAccumulatedData {
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage
        }
    }

    pub fn to_combined(self) -> CombinedAccumulatedData {
        CombinedAccumulatedData {
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            public_data_update_requests: unsafe::zeroed()
        }
    }

    pub fn split_to_public(
        self,
        min_revertible_side_effect_counter: u32
    ) -> (PublicAccumulatedData, PublicAccumulatedData) {
        let mut non_revertible_builder: PublicAccumulatedDataBuilder = unsafe::zeroed();
        let mut revertible_builder: PublicAccumulatedDataBuilder = unsafe::zeroed();

        for i in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            let note_hash = self.new_note_hashes.storage[i];
            if note_hash.counter < min_revertible_side_effect_counter {
                non_revertible_builder.new_note_hashes.push(note_hash);
            } else {
                revertible_builder.new_note_hashes.push(note_hash);
            }
        }
        for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
            let nullifier = self.new_nullifiers.storage[i];
            if nullifier.counter < min_revertible_side_effect_counter {
                non_revertible_builder.new_nullifiers.push(nullifier);
            } else {
                revertible_builder.new_nullifiers.push(nullifier);
            }
        }

        for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX {
            let call_stack_item = self.public_call_stack.storage[i];
            if call_stack_item.start_side_effect_counter < min_revertible_side_effect_counter {
                non_revertible_builder.public_call_stack.push(call_stack_item);
            } else {
                revertible_builder.public_call_stack.push(call_stack_item);
            }
        }

        revertible_builder.new_l2_to_l1_msgs =  self.new_l2_to_l1_msgs;
        revertible_builder.encrypted_logs_hash = self.encrypted_logs_hash;
        revertible_builder.unencrypted_logs_hash = self.unencrypted_logs_hash;
        revertible_builder.encrypted_log_preimages_length =  self.encrypted_log_preimages_length;
        revertible_builder.unencrypted_log_preimages_length= self.unencrypted_log_preimages_length;

        (non_revertible_builder.finish(), revertible_builder.finish())
    }
}

mod tests {
    use crate::{
        abis::{
        accumulated_data::private_accumulated_data_builder::PrivateAccumulatedDataBuilder,
        call_request::CallRequest, caller_context::CallerContext,
        public_data_update_request::PublicDataUpdateRequest,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash}
    },
        address::AztecAddress, utils::arrays::array_eq
    };
    use dep::std::unsafe;

    #[test]
    unconstrained fn splits_revertible_and_non_revertible() {
        let mut builder: PrivateAccumulatedDataBuilder = unsafe::zeroed();

        let non_revertible_commitments = [
            SideEffect { value: 1, counter: 1 },
            SideEffect { value: 2, counter: 3 }
        ];

        let non_revertible_nullifiers = [
            SideEffectLinkedToNoteHash { value: 10, note_hash: 1, counter: 2 },
            SideEffectLinkedToNoteHash { value: 20, note_hash: 2, counter: 4 }
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

        let revertible_commitments = [
            SideEffect { value: 3, counter: 7 },
            SideEffect { value: 4, counter: 10 }
        ];

        let revertible_nullifiers = [
            SideEffectLinkedToNoteHash { value: 30, note_hash: 3, counter: 8 },
            SideEffectLinkedToNoteHash { value: 40, note_hash: 4, counter: 11 }
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

        builder.new_note_hashes.extend_from_array(non_revertible_commitments);
        builder.new_note_hashes.extend_from_array(revertible_commitments);

        builder.new_nullifiers.extend_from_array(non_revertible_nullifiers);
        builder.new_nullifiers.extend_from_array(revertible_nullifiers);

        builder.public_call_stack.extend_from_array(non_revertible_public_stack);
        builder.public_call_stack.extend_from_array(revertible_public_call_stack);

        let (non_revertible, revertible) = builder.split_to_public(7);

        assert(array_eq(non_revertible.new_note_hashes, non_revertible_commitments));
        assert(array_eq(non_revertible.new_nullifiers, non_revertible_nullifiers));
        assert(array_eq(non_revertible.public_call_stack, non_revertible_public_stack));

        assert(array_eq(revertible.new_note_hashes, revertible_commitments));
        assert(array_eq(revertible.new_nullifiers, revertible_nullifiers));
        assert(array_eq(revertible.public_call_stack, revertible_public_call_stack));
    }
}
