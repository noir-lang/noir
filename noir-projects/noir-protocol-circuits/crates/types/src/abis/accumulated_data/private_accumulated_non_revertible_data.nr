use crate::{abis::{call_request::CallRequest, side_effect::{SideEffect, SideEffectLinkedToNoteHash}}};
use crate::constants::{
    MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
    MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX
};

struct PrivateAccumulatedNonRevertibleData {
    revert_code: u8,
    new_note_hashes: [SideEffect; MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX],
    public_call_stack: [CallRequest; MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX],
}

impl PrivateAccumulatedNonRevertibleData {
    pub fn needs_setup(self) -> bool {
        // By definition, the final non-revertible enqueued call is for teardown.
        // since this is a stack, the teardown call would be the 0th element.
        // So if we have more than one element, we need setup.
        !self.public_call_stack[1].is_empty()
    }

    pub fn needs_teardown(self) -> bool {
        // By definition, the final non-revertible enqueued call is for teardown.
        // since this is a stack, the teardown call would be the 0th element.
        !self.public_call_stack[0].is_empty()
    }
}
