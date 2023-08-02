mod address_note;
mod storage;

// Sample escrow contract that stores a balance of a private token on behalf of an owner.
contract Escrow {
    use dep::std;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::CallContext;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::context::Context;
    use dep::aztec::log::emit_encrypted_log;
    use dep::aztec::oracle::get_public_key::get_public_key;

    use dep::aztec::note::{
        note_header::{NoteHeader},
        utils as note_utils,
    };

    use crate::address_note::{
        AddressNote,
        AddressNoteMethods,
        ADDRESS_NOTE_LEN
    };

    use crate::storage::Storage;

    // Creates a new instance
    fn constructor(
        inputs: pub PrivateContextInputs,
        owner: pub Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([owner]));
        let this = inputs.call_context.storage_contract_address;
        
        let storage = Storage::init();
        let mut note = AddressNote::new(owner, this);
        storage.owners.insert(&mut context, &mut note);
        emit_encrypted_log(
            &mut context,
            this,
            storage.owners.storage_slot,
            this,
            get_public_key(this),
            note.serialise(),
        );

        context.finish()
    }

    // Withdraws balance. Requires that msg.sender is registered as an owner.
    fn withdraw(
        inputs: pub PrivateContextInputs,
        token: pub Field,
        amount: pub Field,
        recipient: pub Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([token, amount, recipient]));
        let this = inputs.call_context.storage_contract_address;
        let sender = inputs.call_context.msg_sender;
        let storage = Storage::init();
        
        // TODO: Do we need to manually nullify and recreate this note for access control? Or does Set handle it for us? Or since we have no methods for updating it, we're good?
        let mut note = AddressNote::new(sender, this);
        storage.owners.assert_contains(&mut context, &mut note);
        
        // TODO: Can we dynamically get this selector?
        let transfer_selector = 0xdcd4c318;
        let _callStackItem = context.call_private_function(token, transfer_selector, [amount, this, recipient]);

        context.finish()
    }

    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; ADDRESS_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        assert(storage_slot == 1);
        note_utils::compute_note_hash_and_nullifier(AddressNoteMethods, note_header, preimage)
    }
}
