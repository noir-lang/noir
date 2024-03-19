// Sample escrow contract that stores a balance of a private token on behalf of an owner.
contract Escrow {
    use dep::aztec::prelude::{AztecAddress, EthAddress, FunctionSelector, NoteHeader, PrivateContext, PrivateImmutable};

    use dep::aztec::{context::{PublicContext, Context}, oracle::get_public_key::get_public_key};

    use dep::address_note::address_note::AddressNote;

    struct Storage {
        owner: PrivateImmutable<AddressNote>,
    }

    // Creates a new instance
    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(owner: pub AztecAddress) {
        let mut note = AddressNote::new(owner, owner);
        storage.owner.initialize(&mut note, true);
    }

    // Withdraws balance. Requires that msg.sender is the owner.
    #[aztec(private)]
    fn withdraw(token: AztecAddress, amount: Field, recipient: AztecAddress) {
        let this = context.this_address();
        let sender = context.msg_sender();

        let note = storage.owner.get_note();
        assert(note.address == sender);

        let selector = FunctionSelector::from_signature("transfer((Field),(Field),Field,Field)");
        let _callStackItem = context.call_private_function(
            token,
            selector,
            [this.to_field(), recipient.to_field(), amount, 0]
        );
    }
}
