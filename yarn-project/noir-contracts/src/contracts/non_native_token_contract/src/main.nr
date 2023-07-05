mod storage;

// Represents an ERC20 token bridged from L1 to L2 via `l1-contracts/test/portals/TokenPortal.sol`. The bridged user
// balance can be stored in both public and private domain. If the balance is stored in the private domain then it
// is represented as a set of notes in the private data tree. If the balance is stored in the public domain then it is
// represented as a single value in the public data tree.
contract NonNativeToken {
    // Libs
    use dep::token_utils::{
        balance_utils,
        value_note::{
            value_note::ValueNote,
            filter::get_2_notes,
        }
    };
    use dep::custom_notes::transparent_note::TransparentNote;

    use crate::storage::Storage;

    use dep::aztec::context::Context;
    use dep::aztec::types::point::Point;
    use dep::aztec::{
        abi,
        abi::{
            PublicContextInputs,
            PrivateContextInputs,
        }
    };

    // oracles
    use dep::aztec::oracle::{
        create_l2_to_l1_message::create_l2_to_l1_message,
        create_commitment::create_commitment,
        create_nullifier::create_nullifier,
        get_commitment::get_commitment,
    };

    // public messaging
    use dep::aztec::messaging::consume_l1_to_l2_message_public;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;

    use dep::aztec::log::emit_encrypted_log;
    use dep::aztec::note::note_getter::NoteGetterOptions;

    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        initial_supply: Field, 
        owner: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let balances = storage.balances;

        let mut context = Context::new(inputs, abi::hash_args([initial_supply, owner.x, owner.y]));

        let owner_balance = balances.at(owner.x);
        let note = ValueNote::new(initial_supply, owner);

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        context = owner_balance.insert(context, note);
        context = emit_encrypted_log(context, inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note.serialise());

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Mint Private Function
    // This mint function differs to the typical token mint function as it only allows minting 
    // upon consuming valid messages from a token portal contract
    fn mint(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field, 
        owner: Point,
        owner_address: Field,
        // This field should be hidden
        msg_key: Field,
        secret: Field,
        canceller: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let balances = storage.balances;

        let mut context = Context::new(inputs, abi::hash_args([
            amount, owner.x, owner.y, owner_address, msg_key, secret, canceller
        ]));

        let content_hash = _get_mint_content_hash(amount, owner_address, canceller);

        // Get the l1 message from an oracle call
        let updated_context = context.consume_l1_to_l2_message(inputs, msg_key, content_hash, secret);
        context = updated_context;

        let owner_balance = balances.at(owner.x);
        let note = ValueNote::new(amount, owner);

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        context = owner_balance.insert(context, note);
        context = emit_encrypted_log(context, inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note.serialise());

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }



    // Withdraws using user's private balance.
    // @dev Destroys 2 of user's notes and sends a message to the L1 portal contract. That message can then be consumed
    //      by calling the `withdraw` function on the L1 portal contract (assuming the args are set correctly).
    fn withdraw(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field, 
        sender: Point, 
        recipient: Field, // ethereum address in the field
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let balances = storage.balances;

        let mut initialContext = Context::new(inputs, abi::hash_args([
            amount, sender.x, sender.y, recipient, callerOnL1
        ]));

        // Gets the set of sender's notes and picks 2 of those.
        let sender_balance = balances.at(sender.x);
        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        let (mut context, notes) = sender_balance.get_notes(initialContext, options);
        let note1 = notes[0];
        let note2 = notes[1];

        // TODO: Removing this return breaks the circuit (???)
        context.return_values = context.return_values.push(note1.is_real as Field);
        context.return_values = context.return_values.push(note2.is_real as Field);

        // Ensure the notes are actually owned by the sender (to prevent user from generating a valid proof while
        // spending someone else's notes).
        note1.validate(sender);
        note2.validate(sender);

        let sum = note1.value + note2.value;
        assert(sum as u64 >= amount as u64);
        let change_value = sum - amount;

        context = sender_balance.remove(context, note1);
        context = sender_balance.remove(context, note2);

        let change_note = ValueNote::new(change_value, sender);

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        context = sender_balance.insert(context, change_note);
        context = emit_encrypted_log(context, inputs.call_context.storage_contract_address, sender_balance.storage_slot, change_note.owner, change_note.serialise());
        
        let content = _get_withdraw_content_hash(amount, recipient, callerOnL1);

        context = context.message_portal(content);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Mint Public Function
    // This mint function differs to the typical token mint function as it only allows minting 
    // upon consuming valid messages from a token portal contract
    open fn mintPublic(
        //*********************************/
        // Should eventually be hidden:
        inputs: PublicContextInputs,
        //*********************************/
        amount: Field, 
        owner_address: Field,
        // This field should be hidden
        msg_key: Field,
        secret: Field,
        canceller: Field,
    ) -> Field {
        let storage = Storage::init();
        let public_balances = storage.public_balances;

        let content_hash = _get_mint_content_hash(amount, owner_address, canceller);

        // Consume message and emit nullifier
        consume_l1_to_l2_message_public(inputs, msg_key, content_hash, secret);

        // Update the public balance
        let recipient_balance = public_balances.at(owner_address);
        let new_balance = recipient_balance.read() + amount;
        recipient_balance.write(new_balance);
        new_balance
    }


    // Withdraws using user's public balance.
    open fn withdrawPublic(
        //*********************************/
        // Should eventually be hidden:
        inputs: PublicContextInputs,
        //*********************************/
        amount: Field,
        recipient: Field,
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) {
        let storage = Storage::init();
        let public_balances = storage.public_balances;
        
        let sender = inputs.call_context.msg_sender;
        let sender_balance = public_balances.at(sender);

        let current_sender_balance: Field = sender_balance.read();

        if (current_sender_balance as u120) > (amount as u120) {
            // User has sufficient balance so we decrement it by `amount`
            let _void1 = sender_balance.write(current_sender_balance - amount);
        }  
        // TODO: Revert if there is not enough balance

        let content = _get_withdraw_content_hash(amount, recipient, callerOnL1);

        // Emit the l2 to l1 message
        create_l2_to_l1_message(content);
    }


    // Transfers `amount` of tokens from `sender`'s private balance to a `recipient`.
    // Note: Copied from ZkToken
    fn transfer(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field, 
        sender: Point, 
        recipient: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let balances = storage.balances;

        let mut initialContext = Context::new(inputs, abi::hash_args([
            amount, sender.x, sender.y, recipient.x, recipient.y
        ]));

        // Gets the set of sender's notes and picks 2 of those.
        let sender_balance = balances.at(sender.x);
        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        let (mut context, notes) = sender_balance.get_notes(initialContext, options);
        let note1 = notes[0];
        let note2 = notes[1];

        // Ensure the notes are actually owned by the sender (to prevent user from generating a valid proof while
        // spending someone else's notes).
        note1.validate(sender);
        note2.validate(sender);

        // Checks that the sum of the notes is greater than or equal to the amount being transferred.
        let sum = note1.value + note2.value;
        assert(sum as u64 >= amount as u64);
        let change_value = sum - amount;

        // Removes the 2 notes from the sender's set of notes.
        context = sender_balance.remove(context, note1);
        context = sender_balance.remove(context, note2);

        // Creates 2 new notes, one for the recipient and one for the sender (change note).
        let change_note = ValueNote::new(change_value, sender);
        let recipient_note = ValueNote::new(amount, recipient);

        // Get the recipient's set of notes.
        let recipient_balance = balances.at(recipient.x);

        // Insert the 2 new notes to the recipient's and sender's sets of notes.
        context = recipient_balance.insert(context, recipient_note);
        context = sender_balance.insert(context, change_note);

        // Emit the newly created encrypted note preimages via oracle calls.
        context = emit_encrypted_log(context, inputs.call_context.storage_contract_address, recipient_balance.storage_slot, recipient_note.owner, recipient_note.serialise());
        context = emit_encrypted_log(context, inputs.call_context.storage_contract_address, sender_balance.storage_slot, change_note.owner, change_note.serialise());

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Shield creates a way for a user to move tokens from the public context into the private context.
    open fn shield(
        inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
    ) {
        let storage = Storage::init();
        let public_balances = storage.public_balances;

        // Decrease user's balance.
        let sender = inputs.call_context.msg_sender;
        let sender_balance = public_balances.at(sender);
        let current_sender_balance: Field = sender_balance.read();

        if (current_sender_balance as u120) > (amount as u120) {
            // User has sufficient balance so we decrement it by `amount`
            let _void1 = sender_balance.write(current_sender_balance - amount);
        }  
        
        // Create a commitment to the amount
        let note = TransparentNote::new(amount, secretHash);
        
        // Public oracle call to emit new commitment.
        create_commitment(note.get_commitment());
    }

    // The shield function takes a public balance, and creates a commitment containing the amount of tokens
    // in the private context. 
    fn redeemShield(
        inputs: PrivateContextInputs,
        amount: Field,
        secret: Field,
        owner: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let balances = storage.balances;

        let mut context = Context::new(inputs, abi::hash_args([
            amount, secret, owner.x, owner.y,
        ]));

        // Assert that the note exists within the tree
        let public_note = TransparentNote::new_from_secret(amount, secret);
        context = public_note.consume_in_secret(context, inputs.roots.private_data_tree_root, secret);

        // Mint the tokens
        let owner_balance = balances.at(owner.x);
        let note = ValueNote::new(amount, owner);

        // Insert note and emit encrypted preimage
        context = owner_balance.insert(context, note);
        context = emit_encrypted_log(context, inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note.serialise());

        context.finish()
    }

    fn unshieldTokens(
        inputs: PrivateContextInputs,
        amount: Field,
        owner: Point,
        recipient: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let balances = storage.balances;

        let mut initialContext = Context::new(inputs, abi::hash_args([
            amount, owner.x, owner.y
        ]));

        let sender_balance = balances.at(owner.x);
        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        let (mut context, notes) = sender_balance.get_notes(initialContext, options);
        let note1 = notes[0];
        let note2 = notes[1];

        // TODO: Removing this return breaks the circuit (???)
        context.return_values = context.return_values.push(note1.is_real as Field);
        context.return_values = context.return_values.push(note2.is_real as Field);

        note1.validate(owner);
        note2.validate(owner);

        let sum = note1.value + note2.value;
        assert(sum as u120 >= amount as u120);
        let change_value = sum - amount;

        context = sender_balance.remove(context, note1);
        context = sender_balance.remove(context, note2);

        let change_note = ValueNote::new(change_value, owner);

        // enqueue a public function to perform the public state update.
        context = sender_balance.insert(context, change_note);
        context = emit_encrypted_log(context, inputs.call_context.storage_contract_address, sender_balance.storage_slot, change_note.owner, change_note.serialise());

        let thisAddress = inputs.call_context.storage_contract_address;
        let mut args = [0; abi::MAX_ARGS];
        
        // addUnshieldedBalance selector (in decimal)
        // recompute by: `cast keccak addUnshieldedBalance(field,field)` -> convert to decimal
        let pubEntryPointSelector = 753269941;
        args[0] = amount;
        args[1] = recipient;
        
        let (_callStackItem1, mut context1) = PublicCallStackItem::call(thisAddress, pubEntryPointSelector, args, context);

        context1.finish()
    }

    open fn addUnshieldedBalance(
        _inputs: PublicContextInputs,
        amount: Field,
        recipient: Field,
    ) {
        let storage = Storage::init();
        let public_balances = storage.public_balances;

        let recipient_balance = public_balances.at(recipient);
        let current_balance = recipient_balance.read();
        recipient_balance.write(current_balance + amount);
    }

    /// ABI getBalance type "unconstrained"
    fn getBalance(
        owner: Point,
    ) -> pub Field {
        let storage = Storage::init();
        let owner_balance = storage.balances.at(owner.x);
        
        balance_utils::get_balance(owner_balance.storage_slot)
    }

    // Computes a content hash of a deposit/mint message.
    fn _get_mint_content_hash(amount: Field, owner_address: Field, canceller: Field) -> pub Field {
        let mut hash_bytes: [u8; 100] = [0; 100];
        let amount_bytes = amount.to_be_bytes(32);
        let recipient_bytes = owner_address.to_be_bytes(32);
        let canceller_bytes = canceller.to_be_bytes(32);

        for i in 0..32 {
            hash_bytes[i + 4] = amount_bytes[i];
            hash_bytes[i + 36] = recipient_bytes[i];
            hash_bytes[i + 68] = canceller_bytes[i];
        }
        
        // Function selector: 0xeeb73071 keccak256('mint(uint256,bytes32,address)')
        hash_bytes[0] = 0xee;
        hash_bytes[1] = 0xb7;
        hash_bytes[2] = 0x30;
        hash_bytes[3] = 0x71;
        
        let content_sha256 = dep::std::hash::sha256(hash_bytes);

        // // Convert the content_sha256 to a field element
        let mut v = 1;
        let mut high = 0 as Field;
        let mut low = 0 as Field;

        for i in 0..16 {
            high = high + (content_sha256[15 - i] as Field) * v;
            low = low + (content_sha256[16 + 15 - i] as Field) * v;
            v = v * 256;
        }

        // Abuse that a % p + b % p = (a + b) % p and that low < p
        let content_hash = low + high * v;
        content_hash
    }

    // Computes a content hash of a withdraw message.
    fn _get_withdraw_content_hash(amount: Field, recipient: Field, callerOnL1: Field) -> pub Field {
        // Compute the content hash
        // Compute sha256(selector || amount || recipient)
        // then convert to a single field element
        // add that to the l2 to l1 messages
        let mut hash_bytes: [u8; 100] = [0; 100];
        let amount_bytes = amount.to_be_bytes(32);
        let recipient_bytes = recipient.to_be_bytes(32);
        let callerOnL1_bytes = callerOnL1.to_be_bytes(32);
        
        //  0xb460af94, selector for "withdraw(uint256,address,address)"
        hash_bytes[0] = 0xb4;
        hash_bytes[1] = 0x60;
        hash_bytes[2] = 0xaf;
        hash_bytes[3] = 0x94;

        for i in 0..32 {
            hash_bytes[i + 4] = amount_bytes[i];
            hash_bytes[i + 36] = recipient_bytes[i];
            hash_bytes[i + 68] = callerOnL1_bytes[i];
        }
        let content_sha256 = dep::std::hash::sha256(hash_bytes);

        // Convert the content_sha256 to a field element
        let mut v = 1;
        let mut high = 0 as Field;
        let mut low = 0 as Field;
        
        for i in 0..16 {
            high = high + (content_sha256[15 - i] as Field) * v;
            low = low + (content_sha256[16 + 15 - i] as Field) * v;
            v = v * 256;
        }

        // Abuse that a % p + b % p = (a + b) % p and that low < p
        let content = low + high * v;
        content
    }
}
