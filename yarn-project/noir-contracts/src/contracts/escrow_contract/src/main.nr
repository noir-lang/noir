mod address_note;
mod storage;
mod private_token_contract_interface;

// Sample escrow contract that stores a balance of a private token on behalf of an owner.
contract Escrow {
    use dep::std;
    use dep::aztec::abi::CallContext;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
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

    use crate::private_token_contract_interface::PrivateTokenPrivateContextInterface;

    // Creates a new instance
    #[aztec(private)]
    fn constructor(
        owner: pub Field
    ) {
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
    }

    // Withdraws balance. Requires that msg.sender is registered as an owner.
    #[aztec(private)]
    fn withdraw(
        token: pub Field,
        amount: pub Field,
        recipient: pub Field,
    ) {
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

        let _callStackItem = PrivateTokenPrivateContextInterface::at(token).transfer(&mut context, amount, recipient);
    }

    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; ADDRESS_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        assert(storage_slot == 1);
        note_utils::compute_note_hash_and_nullifier(AddressNoteMethods, note_header, preimage)
    }
}
