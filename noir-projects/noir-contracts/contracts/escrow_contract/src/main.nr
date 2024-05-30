// Sample escrow contract that stores a balance of a private token on behalf of an owner.
contract Escrow {
    use dep::aztec::prelude::{AztecAddress, EthAddress, FunctionSelector, NoteHeader, PrivateContext, PrivateImmutable};

    use dep::address_note::address_note::AddressNote;

    use dep::token::Token;

    #[aztec(storage)]
    struct Storage {
        owner: PrivateImmutable<AddressNote>,
    }

    // Creates a new instance
    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(owner: AztecAddress) {
        let header = context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(&mut context, owner);
        let msg_sender_ovpk_m = header.get_ovpk_m(&mut context, context.msg_sender());
        let owner_ivpk_m = header.get_ivpk_m(&mut context, owner);

        let mut note = AddressNote::new(owner, owner_npk_m_hash);
        storage.owner.initialize(&mut note, msg_sender_ovpk_m, owner_ivpk_m);
    }

    // Withdraws balance. Requires that msg.sender is the owner.
    #[aztec(private)]
    fn withdraw(token: AztecAddress, amount: Field, recipient: AztecAddress) {
        let this = context.this_address();
        let sender = context.msg_sender();

        let note = storage.owner.get_note();
        assert(note.address == sender);

        Token::at(token).transfer(this, recipient, amount, 0).call(&mut context);
    }
}
