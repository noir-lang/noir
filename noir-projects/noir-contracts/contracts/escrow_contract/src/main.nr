// Sample escrow contract that stores a balance of a private token on behalf of an owner.
contract Escrow {
    use dep::std::option::Option;

    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress};

    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        note::{note_getter_options::NoteGetterOptions, note_header::NoteHeader, utils as note_utils},
        oracle::get_public_key::get_public_key, state_vars::PrivateSet
    };

    use dep::address_note::address_note::{AddressNote, ADDRESS_NOTE_LEN};

    struct Storage {
        owners: PrivateSet<AddressNote>,
    }

    // Creates a new instance
    // docs:start:constructor
    #[aztec(private)]
    fn constructor(owner: pub AztecAddress) {
        let this = context.this_address();

        // Create a new note and add it to the owners set.
        let mut note = AddressNote::new(owner, this);

        // Insert the owner into storage
        storage.owners.insert(&mut note, true);
    }
    // docs:end:constructor

    // Withdraws balance. Requires that msg.sender is registered as an owner.
    #[aztec(private)]
    fn withdraw(token: AztecAddress, amount: Field, recipient: AztecAddress) {
        let this = context.this_address();
        let sender = context.msg_sender();

        // We don't remove note from the owners set. If a note exists, the owner and recipient are legit.
        let options = NoteGetterOptions::new().select(0, sender.to_field(), Option::none()).select(1, this.to_field(), Option::none()).set_limit(1);
        let notes = storage.owners.get_notes(options);
        assert(notes[0].is_some(), "Sender is not an owner.");

        let selector = FunctionSelector::from_signature("transfer((Field),(Field),Field,Field)");
        let _callStackItem = context.call_private_function(
            token,
            selector,
            [this.to_field(), recipient.to_field(), amount, 0]
        );
    }
}
