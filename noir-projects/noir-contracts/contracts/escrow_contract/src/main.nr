// Sample escrow contract that stores a balance of a private token on behalf of an owner.
contract Escrow {
    use dep::aztec::prelude::{AztecAddress, EthAddress, FunctionSelector, NoteHeader, PrivateContext, PrivateImmutable};
    use dep::aztec::{
        encrypted_logs::encrypted_note_emission::encode_and_encrypt_note_with_keys,
        keys::getters::get_current_public_keys
    };

    // docs:start:addressnote_import
    use dep::address_note::address_note::AddressNote;
    // docs:end:addressnote_import
    use dep::token::Token;

    #[aztec(storage)]
    struct Storage {
        owner: PrivateImmutable<AddressNote>,
    }

    // Creates a new instance
    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(owner: AztecAddress) {
        let owner_keys = get_current_public_keys(&mut context, owner);
        let msg_sender_keys = get_current_public_keys(&mut context, context.msg_sender());
        // docs:start:addressnote_new
        let mut note = AddressNote::new(owner, owner_keys.npk_m.hash());
        // docs:end:addressnote_new
        storage.owner.initialize(&mut note).emit(
            encode_and_encrypt_note_with_keys(&mut context, msg_sender_keys.ovpk_m, owner_keys.ivpk_m, owner)
        );
    }

    // Withdraws balance. Requires that msg.sender is the owner.
    #[aztec(private)]
    fn withdraw(token: AztecAddress, amount: Field, recipient: AztecAddress) {
        let sender = context.msg_sender();

        let note = storage.owner.get_note();
        assert(note.address == sender);

        Token::at(token).transfer(recipient, amount).call(&mut context);
    }
}
