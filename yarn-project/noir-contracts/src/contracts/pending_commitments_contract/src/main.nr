mod storage;

// Test contract to confirm that notes can be inserted and then later
// read (eventually even nullified) in the same TX. This contract
// also contains some "bad" test cases to ensure that notes cannot
// be read/nullified before their creation etc.
contract PendingCommitments {
    // Libs
    use dep::value_note::{
        balance_utils,
        filter::get_2_notes,
        value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
    };

    use crate::storage::Storage;

    use dep::aztec::constants_gen::ARGS_LENGTH;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::context::PrivateContext;
    use dep::aztec::note::{
        note_getter::NoteGetterOptions,
        note_header::NoteHeader,
        utils as note_utils,
    };
    use dep::aztec::log::emit_encrypted_log;
    use dep::aztec::oracle::get_public_key::get_public_key;

    // TODO(dbanks12): consolidate code into internal helper functions
    // (once Noir's support for this is more robust)

    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs
        //*********************************/
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let context = PrivateContext::new(inputs, 0);
        context.finish()
    }

    // Confirm can access pending commitments by creating / inserting a note and then
    // getting / reading that note all in the same contract function
    // Realistic way to describe this test is "Mint note A, then burn note A in the same transaction"
    fn test_insert_then_get_then_nullify_flat(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner]));

        let owner_balance = storage.balances.at(owner);
        let mut note = ValueNote::new(amount, owner);

        // Insert note and emit encrypted note preimage via oracle call
        owner_balance.insert(&mut context, &mut note);
        emit_encrypted_log(
            &mut context,
            context.inputs.call_context.storage_contract_address,
            owner_balance.storage_slot,
            get_public_key(owner),
            note.serialise(),
        );

        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        // get note inserted above
        let maybe_notes = owner_balance.get_notes(&mut context, options);

        let note0 = maybe_notes[0].unwrap();
        assert(note.value == note0.value);
        assert(maybe_notes[1].is_none());

        context.return_values.push(note0.value);

        owner_balance.remove(&mut context, note0);

        context.finish()
    }

    // Confirm cannot access commitments inserted later in same function
    fn test_bad_get_then_insert_flat(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner]));

        let owner_balance = storage.balances.at(owner);

        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        // get note (note inserted at bottom of function shouldn't exist yet)
        let maybe_notes = owner_balance.get_notes(&mut context, options);

        assert(maybe_notes[0].is_none());
        assert(maybe_notes[1].is_none());

        context.return_values.push(0);

        // Insert note and emit encrypted note preimage via oracle call
        let mut note = ValueNote::new(amount, owner);
        owner_balance.insert(&mut context, &mut note);
        emit_encrypted_log(
            &mut context,
            context.inputs.call_context.storage_contract_address,
            owner_balance.storage_slot,
            get_public_key(owner),
            note.serialise(),
        );

        context.finish()
    }

    // Dumy nested/inner function (to pass a function which does nothing)
    fn dummy(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner]));
        context.finish()
    }

    // Nested/inner function to create and insert a note
    fn insert_note(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner]));

        let owner_balance = storage.balances.at(owner);
        let mut note = ValueNote::new(amount, owner);

        // Insert note and emit encrypted note preimage via oracle call
        owner_balance.insert(&mut context, &mut note);
        emit_encrypted_log(
            &mut context,
            context.inputs.call_context.storage_contract_address,
            owner_balance.storage_slot,
            get_public_key(owner),
            note.serialise(),
        );

        context.finish()
    }

    // Nested/inner function to get a note and confirm it matches the expected value
    fn get_then_nullify_note(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        expected_value: Field,
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([expected_value, owner]));

        let owner_balance = storage.balances.at(owner);

        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        let note = owner_balance.get_notes(&mut context, options)[0].unwrap();

        assert(expected_value == note.value);

        context.return_values.push(expected_value);

        owner_balance.remove(&mut context, note);

        context.finish()
    }

    // Nested/inner function to get a note and confirms that none is returned
    fn get_note_zero_balance(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([owner]));

        let owner_balance = storage.balances.at(owner);

        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        let maybe_notes = owner_balance.get_notes(&mut context, options);

        assert(maybe_notes[0].is_none());
        assert(maybe_notes[1].is_none());

        context.finish()
    }

    // Test pending commitments with note insertion done in a nested call
    // and "read" / get of that pending note/commitment in another nested call
    // Realistic way to describe this test is "Mint note A, then burn note A in the same transaction"
    fn test_insert_then_get_then_nullify_all_in_nested_calls(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
        insert_fn_selector: Field,
        get_then_nullify_fn_selector: Field,
        get_note_zero_fn_selector: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner, insert_fn_selector, get_then_nullify_fn_selector, get_note_zero_fn_selector]));

        // args for nested calls
        let mut args = [0; ARGS_LENGTH];
        args[0] = amount;
        args[1] = owner;

        // nested call to create/insert note
        let _callStackItem1 = context.call_private_function(inputs.call_context.storage_contract_address, insert_fn_selector, args);
        // nested call to read and nullify that note
        let _callStackItem2 = context.call_private_function(inputs.call_context.storage_contract_address, get_then_nullify_fn_selector, args);
         // nested call to confirm that balance is zero
        let _callStackItem3 = context.call_private_function(inputs.call_context.storage_contract_address, get_note_zero_fn_selector, [owner]);

        context.finish()
    }

    // same test as above, but insert 2, get 2, nullify 2
    fn test_insert2_then_get2_then_nullify2_all_in_nested_calls(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
        insert_fn_selector: Field,
        get_then_nullify_fn_selector: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner, insert_fn_selector, get_then_nullify_fn_selector]));

        // args for nested calls
        let mut args = [0; ARGS_LENGTH];
        args[0] = amount;
        args[1] = owner;

        // nested call to create/insert note
        let _callStackItem1 = context.call_private_function(inputs.call_context.storage_contract_address, insert_fn_selector, args);
        let _callStackItem2 = context.call_private_function(inputs.call_context.storage_contract_address, insert_fn_selector, args);
        // nested call to read and nullify that note
        let _callStackItem3 = context.call_private_function(inputs.call_context.storage_contract_address, get_then_nullify_fn_selector, args);
        let _callStackItem4 = context.call_private_function(inputs.call_context.storage_contract_address, get_then_nullify_fn_selector, args);
        // nested call to confirm that balance is zero
        // TODO(dbanks12): once > 4 nested calls is supported, can confirm 0 balance:
        //let _callStackItem5 = context.call_private_function(inputs.call_context.storage_contract_address, get_note_zero_fn_selector, [owner]);

        context.finish()
    }

    // same test as above, but insert 2, get 1, nullify 1
    fn test_insert2_then_get2_then_nullify1_all_in_nested_calls(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
        insert_fn_selector: Field,
        get_then_nullify_fn_selector: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner, insert_fn_selector, get_then_nullify_fn_selector]));

        // args for nested calls
        let mut args = [0; ARGS_LENGTH];
        args[0] = amount;
        args[1] = owner;

        // nested call to create/insert note
        let _callStackItem1 = context.call_private_function(inputs.call_context.storage_contract_address, insert_fn_selector, args);
        let _callStackItem2 = context.call_private_function(inputs.call_context.storage_contract_address, insert_fn_selector, args);
        // nested call to read and nullify that note
        let _callStackItem3 = context.call_private_function(inputs.call_context.storage_contract_address, get_then_nullify_fn_selector, args);

        context.finish()
    }

    // insert 1 note, then get 2 notes (one pending, one persistent) and nullify both.
    // one nullifier will be squashed with the pending note, one will become persistent.
    // ONLY WORKS IF THERE IS A PERSISTENT NOTE TO GET
    fn test_insert1_then_get2_then_nullify2_all_in_nested_calls(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
        insert_fn_selector: Field,
        get_then_nullify_fn_selector: Field,
        get_note_zero_fn_selector: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        //let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner, insert_fn_selector, get_then_nullify_fn_selector]));
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner, insert_fn_selector, get_then_nullify_fn_selector, get_note_zero_fn_selector]));

        // args for nested calls
        let mut args = [0; ARGS_LENGTH];
        args[0] = amount;
        args[1] = owner;

        // nested call to create/insert note
        let _callStackItem1 = context.call_private_function(inputs.call_context.storage_contract_address, insert_fn_selector, args);
        // nested call to read and nullify that note
        let _callStackItem2 = context.call_private_function(inputs.call_context.storage_contract_address, get_then_nullify_fn_selector, args);
        let _callStackItem3 = context.call_private_function(inputs.call_context.storage_contract_address, get_then_nullify_fn_selector, args);

        let _callStackItem4 = context.call_private_function(inputs.call_context.storage_contract_address, get_note_zero_fn_selector, [owner]);

        context.finish()
    }

    // Confirm cannot get/read a pending commitment in a nested call
    // that is created/inserted later in execution but in the parent.
    // NOTE: This test is especially important in an end-to-end context because the parent call
    // (and therefore the insertion) will be processed in an earlier kernel iteration, but the
    // nested call (later kernel iteration) should not be able to read the commitment despite
    // it being present at that stage in the kernel.
    // If we can somehow force the simulator to allow execution to succeed can ensure that this test fails in the kernel
    //fn test_bad_get_in_nested_call_then_insert(
    //    //*********************************/
    //    // Should eventually be hidden:
    //    inputs: PrivateContextInputs,
    //    //*********************************/
    //    amount: Field,
    //    owner: Field,
    //    get_then_nullify_fn_selector: Field,
    //) -> distinct pub abi::PrivateCircuitPublicInputs {
    //}

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
    }
}