use dep::std::hash::pedersen;

contract PublicPrivate {
    mod balance_utils;
    mod transparent_note;

    use transparent_note::TransparentNote;

    use dep::aztec3::abi;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::abi::PublicContextInputs;

    // Public state
    use dep::aztec3::state_vars::storage_map::StorageMap;
    use dep::aztec3::state_vars::storage_field::StorageField;

    // Private state
    use dep::aztec3::types::point::Point;
    use dep::aztec3::state_vars::map::Map;
    use dep::aztec3::state_vars::set::Set;
    use dep::aztec3::notes::value_note::Note;

    // oracles
    use dep::aztec3::oracle::logs::emit_encrypted_log;
    use dep::aztec3::oracle::create_commitment::create_commitment;
    use dep::aztec3::oracle::create_l2_to_l1_message::create_l2_to_l1_message;
    use dep::aztec3::oracle::create_nullifier::create_nullifier;
    use dep::aztec3::oracle::get_commitment::get_commitment;

    global balances = Map { storage_slot: 1 };

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        PrivateFunctionContext::new().finish(inputs)
    }


    open fn mintFromPublicToPrivate(
        _inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
        _padding: [Field; abi::MAX_ARGS - 3]
    ) {
    
        // Create a commitment to the amount
        let note = TransparentNote::new(amount, secretHash);
        
        // Public oracle call to emit new commitment.
        create_commitment(note.get_commitment());
    }

    // Purely exists for testing
    open fn createL2ToL1MessagePublic(
        _inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
        _padding: [Field; abi::MAX_ARGS - 3]
    ) {
    
        // Create a commitment to the amount
        let note = TransparentNote::new(amount, secretHash);
        
        // Public oracle call to emit new commitment.
        create_l2_to_l1_message(note.get_commitment());
    }

    // Purely exists for testing
    open fn createNullifierPublic(
        _inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
        _padding: [Field; abi::MAX_ARGS - 3]
    ) -> Field {
    
        // Create a commitment to the amount
        let note = TransparentNote::new(amount, secretHash);
        
        // Public oracle call to emit new commitment.
        create_nullifier(note.get_commitment());
        0
    }

    


    fn mintFromPublicMessage(
        inputs: PrivateContextInputs,
        amount: Field,
        secret: Field,
        owner: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateFunctionContext::new();
        context.args = context.args.push_array([amount, secret, owner.x, owner.y]);

        // Assert that the note exists within the tree
        let public_note = TransparentNote::new_from_secret(amount, secret);
        context = public_note.consume_in_secret(context, inputs.roots.private_data_tree_root, secret);

        // Mint the tokens
        let owner_balance = balances.at(owner.x);
        let note = Note::new(amount, owner);

        // Insert note and emit encrypted preimage
        context = owner_balance.insert(context, note);
        assert(emit_encrypted_log(inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note) == 0);

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
