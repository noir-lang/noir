contract ZkToken {
    mod balance_utils;
    use dep::aztec3::abi;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::notes::value_note::Note;
    use dep::aztec3::types::point::Point;
    use dep::aztec3::state_vars::map::Map;
    use dep::aztec3::state_vars::set::Set;

    // oracles
    use dep::aztec3::oracle::logs::emit_encrypted_log;
    use dep::aztec3::oracle::logs::emit_unencrypted_log;

    //*********************************/
    global balances = Map { storage_slot: 1 };
    // Should eventually be replaced by:
    // `balances: Mapping<Field, Set<Note>>;`
    //*********************************/

    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        initial_supply: Field, 
        owner: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateFunctionContext::new();
        context.args = context.args.push_array([initial_supply, owner.x, owner.y]);

        let owner_balance = balances.at(owner.x);
        let note = Note::new(initial_supply, owner);

        // Insert note and emit encrypted note preimage via oracle call
        context = owner_balance.insert(context, note);
        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note) == 0);
        assert(emit_unencrypted_log("Balance set in constructor") == 0);

        context.finish(inputs)
    }

    fn mint(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field, 
        owner: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateFunctionContext::new();
        context.args = context.args.push_array([amount, owner.x, owner.y]);

        let owner_balance = balances.at(owner.x);
        let note = Note::new(amount, owner);

        // Insert note and emit encrypted note preimage via oracle call
        context = owner_balance.insert(context, note);
        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note) == 0);
        assert(emit_unencrypted_log("Coins minted") == 0);

        context.finish(inputs)
    }

    fn transfer(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field, 
        sender: Point, 
        recipient: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut initialContext = PrivateFunctionContext::new();
        initialContext.args = initialContext.args.push_array([amount, sender.x, sender.y, recipient.x, recipient.y]);

        let sender_balance = balances.at(sender.x);
        let (mut context, (note1, note2)) = sender_balance.get_2(initialContext);

        note1.validate(sender);
        note2.validate(sender);

        let sum = note1.value + note2.value;
        assert(sum as u64 >= amount as u64);
        let change_value = sum - amount;

        context = sender_balance.remove(inputs, context, note1);
        context = sender_balance.remove(inputs, context, note2);

        let change_note = Note::new(change_value, sender);
        let recipient_note = Note::new(amount, recipient);

        let recipient_balance = balances.at(recipient.x);

        context = recipient_balance.insert(context, recipient_note);
        context = sender_balance.insert(context, change_note);

        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, recipient_balance.storage_slot, recipient_note.owner, recipient_note) == 0);
        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, sender_balance.storage_slot, change_note.owner, change_note) == 0);
        assert(emit_unencrypted_log("Coins transferred") == 0);

        context.finish(inputs)
    }

    /// ABI getBalance type "unconstrained"
    fn getBalance(
        owner: Point,
        _padding: [Field; abi::MAX_ARGS - 2]
    ) -> pub Field {
        let owner_balance = balances.at(owner.x);
        
        balance_utils::get_balance(owner_balance.storage_slot)
    }
}
