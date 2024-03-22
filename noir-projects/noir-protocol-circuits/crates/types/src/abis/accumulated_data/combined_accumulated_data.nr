use crate::{
    abis::{
    accumulated_data::{
    public_accumulated_non_revertible_data::PublicAccumulatedNonRevertibleData,
    public_accumulated_revertible_data::PublicAccumulatedRevertibleData
},
    call_request::CallRequest, caller_context::CallerContext,
    public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
}
};
use crate::constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
};

use dep::std::unsafe;
use crate::traits::is_empty;

use crate::utils::arrays::{array_cp, array_concat, array_to_bounded_vec};

struct CombinedAccumulatedData {
    revert_code: u8,

    new_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hash: Field,
    unencrypted_logs_hash: Field,

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
}

impl CombinedAccumulatedData {
    pub fn needs_app_logic(self) -> bool {
        // if we have any enqueued revertible public calls, we need to run the public app logic circuit.
        !self.public_call_stack[0].is_empty()
    }

    pub fn recombine(
        non_revertible: PublicAccumulatedNonRevertibleData,
        revertible: PublicAccumulatedRevertibleData
    ) -> CombinedAccumulatedData {
        CombinedAccumulatedData {
            revert_code: non_revertible.revert_code,
            new_note_hashes: array_concat(non_revertible.new_note_hashes, revertible.new_note_hashes),
            new_nullifiers: array_concat(non_revertible.new_nullifiers, revertible.new_nullifiers),
            private_call_stack: revertible.private_call_stack,
            public_call_stack: array_concat(
                non_revertible.public_call_stack,
                revertible.public_call_stack
            ),
            new_l2_to_l1_msgs: revertible.new_l2_to_l1_msgs,
            encrypted_logs_hash: revertible.encrypted_logs_hash,
            unencrypted_logs_hash: revertible.unencrypted_logs_hash,
            encrypted_log_preimages_length: revertible.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: revertible.unencrypted_log_preimages_length,
            public_data_update_requests: array_concat(
                non_revertible.public_data_update_requests,
                revertible.public_data_update_requests
            )
        }
    }
}

mod tests {
    use crate::abis::{
        accumulated_data::combined_accumulated_data_builder::CombinedAccumulatedDataBuilder,
        call_request::CallRequest, caller_context::CallerContext,
        public_data_update_request::PublicDataUpdateRequest,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash}
    };
    use crate::address::AztecAddress;
    use crate::utils::arrays::array_eq;
    use dep::std::unsafe;

    #[test]
    unconstrained fn splits_revertible_and_non_revertible() {
        let mut builder: CombinedAccumulatedDataBuilder = unsafe::zeroed();

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

        let (non_revertible, revertible) = builder.split(7);

        assert(array_eq(non_revertible.new_note_hashes, non_revertible_commitments));
        assert(array_eq(non_revertible.new_nullifiers, non_revertible_nullifiers));
        assert(array_eq(non_revertible.public_call_stack, non_revertible_public_stack));

        assert(array_eq(revertible.new_note_hashes, revertible_commitments));
        assert(array_eq(revertible.new_nullifiers, revertible_nullifiers));
        assert(array_eq(revertible.public_call_stack, revertible_public_call_stack));
    }
}
