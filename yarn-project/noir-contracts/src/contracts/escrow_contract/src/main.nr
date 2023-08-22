mod address_note;
mod storage;

// Sample escrow contract that stores a balance of a private token on behalf of an owner.
contract Escrow {
    use dep::std;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::CallContext;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::context::PrivateContext;
    use dep::aztec::log::emit_encrypted_log;
    use dep::aztec::oracle::get_public_key::get_public_key;

    use dep::aztec::note::{
        note_getter_options::NoteGetterOptions,
        note_header::{NoteHeader},
        utils as note_utils,
    };

    use crate::address_note::{
        AddressNote,
        AddressNoteMethods,
        ADDRESS_NOTE_LEN,
    };

    use crate::storage::Storage;

    // Creates a new instance
    fn constructor(
        inputs: pub PrivateContextInputs,
        owner: pub Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([owner]));
        let this = context.this_address();
        
        let storage = Storage::init();
        let mut note = AddressNote::new(owner, this);
        storage.owners.insert(&mut context, &mut note);
        emit_encrypted_log(
            &mut context,
            this,
            storage.owners.storage_slot,
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
        let mut context = PrivateContext::new(inputs, abi::hash_args([token, amount, recipient]));
        let this = context.this_address();
        let sender = context.msg_sender();
        let storage = Storage::init();
        
        // We don't remove note from the owners set. If a note exists, the owner and recipient are legit.
        let options = NoteGetterOptions::new().select(0, sender).select(1, this).set_limit(1);
        let notes = storage.owners.get_notes(&mut context, options);
        let note = notes[0].unwrap_unchecked();
        // Filter is not constrained. We still need to check if the note is what we expected.
        assert(note.address == sender);
        assert(note.owner == this);

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
