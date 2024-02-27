// A contract used along with `Parent` contract to test nested calls.
contract Delegator {
    use dep::std::option::Option;

    use dep::aztec::{
        log::emit_unencrypted_log, state_vars::{PublicMutable, PrivateSet},
        protocol_types::{abis::{function_selector::FunctionSelector}, address::AztecAddress},
        note::{note_viewer_options::NoteViewerOptions, utils as note_utils, note_header::NoteHeader}
    };
    use dep::value_note::value_note::{ValueNote, VALUE_NOTE_LEN};

    struct Storage {
        current_value: PublicMutable<Field>,
        a_private_value: PrivateSet<ValueNote>,
    }

    #[aztec(private)]
    fn constructor() {}

    #[aztec(private)]
    fn private_delegate_set_value(
        targetContract: AztecAddress,
        targetSelector: FunctionSelector,
        args: [Field; 2]
    ) {
        // Call the target private function
        let return_values = context.delegate_call_private_function(targetContract, targetSelector, args);
        // Copy the return value from the call to this function's return values
        return_values[0]
    }

    #[aztec(private)]
    fn enqueued_delegate_set_value(
        targetContract: AztecAddress,
        targetSelector: FunctionSelector,
        args: [Field; 1]
    ) {
        context.delegate_call_public_function(targetContract, targetSelector, args);
    }

    #[aztec(public)]
    fn public_delegate_set_value(
        targetContract: AztecAddress,
        targetSelector: FunctionSelector,
        args: [Field; 1]
    ) {
        let _ = context.delegate_call_public_function(targetContract, targetSelector, args);
    }

    unconstrained fn view_private_value(amount: Field, owner: AztecAddress) -> pub Field {
        let options = NoteViewerOptions::new().select(0, amount, Option::none()).select(1, owner.to_field(), Option::none()).set_limit(1);
        let notes = storage.a_private_value.view_notes(options);
        notes[0].unwrap_unchecked().value
    }

    unconstrained fn view_public_value() -> pub Field {
        storage.current_value.read()
    }

    // TODO: remove this placeholder once https://github.com/AztecProtocol/aztec-packages/issues/2918 is implemented
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        note_type_id: Field,
        serialized_note: [Field; VALUE_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNote::deserialize_content, note_header, serialized_note)
    }
}
