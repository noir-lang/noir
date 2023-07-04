// Noir implementation of a private token contract. It allows for the minting, and transfer of tokens. This contract
// only works with private state and for this reason user balances are stored as notes (total user balance is
// represented as a set of notes).
contract ZkToken {
    mod balance_utils;
    use dep::aztec3::abi;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::notes::value_note::Note;
    use dep::aztec3::types::point::Point;
    use dep::aztec3::state_vars::map::Map;
    use dep::aztec3::state_vars::set::Set;

    // oracles used to emit logs
    use dep::aztec3::oracle::logs::emit_encrypted_log;
    use dep::aztec3::oracle::logs::emit_unencrypted_log;

    //*********************************/
    // A map from `owner.x` (Field type) to a set of notes (Set type) representing user balance.
    global balances = Map { storage_slot: 1 };
    // Should eventually be replaced by:
    // `balances: Mapping<Field, Set<Note>>;`
    //*********************************/

    // Constructs the contract and sets `initial_supply` which is fully owned by `owner`.
    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        initial_supply: Field, 
        owner: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateFunctionContext::new(inputs, abi::hash_args([initial_supply, owner.x, owner.y]));

        let owner_balance = balances.at(owner.x);
        let note = Note::new(initial_supply, owner);

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        context = owner_balance.insert(context, note);
        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note) == 0);
        assert(emit_unencrypted_log("Balance set in constructor") == 0); // unencrypted log is emitted here only for testing purposes

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Mints `amount` of tokens to `owner`.
    fn mint(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field, 
        owner: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateFunctionContext::new(inputs, abi::hash_args([amount, owner.x, owner.y]));

        let owner_balance = balances.at(owner.x);
        let note = Note::new(amount, owner);

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        context = owner_balance.insert(context, note);
        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note) == 0);
        assert(emit_unencrypted_log("Coins minted") == 0); // unencrypted log is emitted here only for testing purposes

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
    }

    // Transfers `amount` of tokens from `sender` to a `recipient`.
    fn transfer(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field, 
        sender: Point, 
        recipient: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut initialContext = PrivateFunctionContext::new(inputs, abi::hash_args([amount, sender.x, sender.y, recipient.x, recipient.y]));

        // Gets the set of sender's notes and picks 2 of those.
        let sender_balance = balances.at(sender.x);
        let (mut context, (note1, note2)) = sender_balance.get_2(initialContext);

        // Ensure the notes are actually owned by the sender (to prevent user from generating a valid proof while
        // spending someone else's notes).
        note1.validate(sender);
        note2.validate(sender);

        // Checks that the sum of the notes is greater than or equal to the amount being transferred.
        let sum = note1.value + note2.value;
        assert(sum as u64 >= amount as u64);
        let change_value = sum - amount;

        // Removes the 2 notes from the sender's set of notes.
        context = sender_balance.remove(inputs, context, note1);
        context = sender_balance.remove(inputs, context, note2);

        // Creates 2 new notes, one for the recipient and one for the sender (change note).
        let change_note = Note::new(change_value, sender);
        let recipient_note = Note::new(amount, recipient);

        // Get the recipient's set of notes.
        let recipient_balance = balances.at(recipient.x);

        // Insert the 2 new notes to the recipient's and sender's sets of notes.
        context = recipient_balance.insert(context, recipient_note);
        context = sender_balance.insert(context, change_note);

        // Emit the newly created encrypted note preimages via oracle calls.
        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, recipient_balance.storage_slot, recipient_note.owner, recipient_note) == 0);
        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, sender_balance.storage_slot, change_note.owner, change_note) == 0);
        assert(emit_unencrypted_log("Coins transferred") == 0); // unencrypted log is emitted here only for testing purposes

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
    }

    /// ABI getBalance type "unconstrained"
    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    fn getBalance(
        owner: Point,
    ) -> pub Field {
        // Get the set of notes owned by the user.
        let owner_balance = balances.at(owner.x);
        
        // Return the sum of all notes in the set.
        balance_utils::get_balance(owner_balance.storage_slot)
    }
}
