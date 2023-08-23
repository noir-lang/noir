mod storage;

// Testing token that can be bridged in and out.
// TODOS:
// - Add role based access control to mint functions
// - Add function for managing roles
// - Add public self-burn function for users to burn their own tokens
contract NativeToken {
    // Libs
    use dep::value_note::{
        balance_utils,
        utils::{send_note, spend_notes},
        value_note::{VALUE_NOTE_LEN, ValueNoteMethods},
    };

    use dep::non_native::{
        hash::{get_mint_content_hash, get_withdraw_content_hash},
        transparent_note::{
            TransparentNote,
            TransparentNoteMethods
        },
    };

    use crate::storage::Storage;

    use dep::aztec::context::{
        PrivateContext,
        PublicContext,
    };
    use dep::aztec::types::point::Point;
    use dep::aztec::{
        abi,
        abi::{
            PublicContextInputs,
            PrivateContextInputs,
        },
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
    };

    use dep::aztec::public_call_stack_item::PublicCallStackItem;

    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        initial_supply: Field,
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();

        let mut context = PrivateContext::new(inputs, abi::hash_args([initial_supply, owner]));

        let balance = storage.balances.at(owner);
        send_note(&mut context, balance, initial_supply, owner);

        context.finish()
    }

    // uint256(keccak256("owner_mint_pub(field,field)")) >> 224 -> 1071038680
    open fn owner_mint_pub(
        inputs: PublicContextInputs,
        to: Field,
        amount: Field,
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([to, amount]));
        let storage = Storage::init();
        let new_balance = storage.public_balances.at(to).read() + amount;
        storage.public_balances.at(to).write(new_balance);
        storage.total_supply.write(storage.total_supply.read() + amount);
        context.return_values.push(1);
        context.finish()
    }

    // uint256(keccak256("owner_mint_priv(field,field)")) >> 224 -> 3157518188
    open fn owner_mint_priv(
        inputs: PublicContextInputs,
        amount: Field,
        secret_hash: Field,
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([amount, secret_hash]));

        let storage = Storage::init();
        let pending_shields = storage.pending_shields;

        let mut note = TransparentNote::new(amount, secret_hash);
        pending_shields.insert_from_public(&mut context, &mut note);

        storage.total_supply.write(storage.total_supply.read() + amount);
    
        context.return_values.push(1);
        context.finish()
    }

    // Mint Private Function
    // This mint function differs to the typical token mint function as it only allows minting
    // upon consuming valid messages from a token portal contract
    // uint256(keccak256("mint(field,field,field,field,field)")) >> 224 -> 2341211258
    fn mint(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        owner: Field,
        // This field should be hidden
        msg_key: Field,
        secret: Field,
        canceller: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();

        let mut context = PrivateContext::new(inputs, abi::hash_args([
            amount, owner, msg_key, secret, canceller
        ]));

        let content_hash = get_mint_content_hash(amount, owner, canceller);

        // Get the l1 message from an oracle call
        context.consume_l1_to_l2_message(inputs, msg_key, content_hash, secret);

        let balance = storage.balances.at(owner);
        send_note(&mut context, balance, amount, owner);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Withdraws using user's private balance.
    // @dev Destroys 2 of user's notes and sends a message to the L1 portal contract. That message can then be consumed
    //      by calling the `withdraw` function on the L1 portal contract (assuming the args are set correctly).
    // uint256(keccak256("withdraw(field,field,field,field)")) >> 224 -> 3193431016
    fn withdraw(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        sender: Field,
        recipient: Field, // ethereum address in the field
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            amount, sender, recipient, callerOnL1
        ]));

        let sender_balance = storage.balances.at(sender);
        spend_notes(&mut context, sender_balance, amount, sender);

        let content = get_withdraw_content_hash(amount, recipient, callerOnL1);
        context.message_portal(content);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Mint Public Function
    // This mint function differs to the typical token mint function as it only allows minting
    // upon consuming valid messages from a token portal contract
    // uint256(keccak256("mintPublic(field,field,field,field,field)")) >> 224 -> 1598652179
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
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([amount, owner_address, msg_key, secret, canceller]));

        let storage = Storage::init();
        let public_balances = storage.public_balances;

        let content_hash = get_mint_content_hash(amount, owner_address, canceller);

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(msg_key, content_hash, secret);

        // Update the public balance
        let recipient_balance = public_balances.at(owner_address);
        let new_balance = recipient_balance.read() + amount;
        recipient_balance.write(new_balance);

        // Push the return value into the context
        context.return_values.push(new_balance);
        context.finish()
    }

    // Withdraws using user's public balance.
    // uint256(keccak256("withdrawPublic(field,field,field)")) >> 224 -> 2996031894
    open fn withdrawPublic(
        //*********************************/
        // Should eventually be hidden:
        inputs: PublicContextInputs,
        //*********************************/
        amount: Field,
        recipient: Field,
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([amount, recipient, callerOnL1]));

        let storage = Storage::init();
        let public_balances = storage.public_balances;

        let sender = context.msg_sender();
        let sender_balance = public_balances.at(sender);

        let current_sender_balance: Field = sender_balance.read();

        if (current_sender_balance as u120) > (amount as u120) {
            // User has sufficient balance so we decrement it by `amount`
            let _void1 = sender_balance.write(current_sender_balance - amount);
        }
        // TODO: Revert if there is not enough balance

        let content = get_withdraw_content_hash(amount, recipient, callerOnL1);

        // Emit the l2 to l1 message
        context.message_portal(content);

        // Push the return value into the context
        context.finish()
    }

    open fn approve(
        //*********************************/
        // Should eventually be hidden:
        inputs: PublicContextInputs,
        //*********************************/
        spender: Field,
        allowance: Field,
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([spender, allowance]));

        let storage = Storage::init();
        storage.public_allowances.at(context.msg_sender()).at(spender).write(allowance);

        context.finish()
    }

    // uint256(keccak256("transfer_pub(field,field)")) >> 224 -> 1012824788
    open fn transfer_pub(
        //*********************************/
        // Should eventually be hidden:
        inputs: PublicContextInputs,
        //*********************************/
        to: Field,
        amount: Field,
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([to, amount]));

        let storage = Storage::init();

        // Decrease user's balance.
        let sender = context.msg_sender();
        let sender_balance = storage.public_balances.at(sender);
        let current_sender_balance: Field = sender_balance.read();
        assert(current_sender_balance as u120 >= amount as u120);

        let to_balance = storage.public_balances.at(to);
        let current_to_balance: Field = to_balance.read();

        // User has sufficient balance so we decrement it by `amount`
        sender_balance.write(current_sender_balance - amount);
        to_balance.write(current_to_balance + amount);

        context.finish()
    }

    // uint256(keccak256("transfer_from_pub(field,field,field)")) >> 224 -> 1602017294
    open fn transfer_from_pub(
        //*********************************/
        // Should eventually be hidden:
        inputs: PublicContextInputs,
        //*********************************/
        from: Field,
        to: Field,
        amount: Field,
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([from, to ,amount]));

        let storage = Storage::init();

        // Decrease allowance
        let allowance = storage.public_allowances.at(from).at(context.msg_sender());
        let current_allowance: Field = allowance.read();
        assert(current_allowance as u120 >= amount as u120);
        allowance.write(current_allowance - amount);

        // Decrease user's balance.
        let sender_balance = storage.public_balances.at(from);
        let current_sender_balance: Field = sender_balance.read();
        assert(current_sender_balance as u120 >= amount as u120);

        let to_balance = storage.public_balances.at(to);
        let current_to_balance: Field = to_balance.read();

        // User has sufficient balance so we decrement it by `amount`
        sender_balance.write(current_sender_balance - amount);
        to_balance.write(current_to_balance + amount);

        context.finish()
    }

    // Transfers `amount` of tokens from `sender`'s private balance to a `recipient`.
    // Note: Copied from PrivateToken
    // uint256(keccak256("transfer(field,field,field)")) >> 224 -> 3704931096
    fn transfer(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        from: Field,
        to: Field,
        amount: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            from, to, amount
        ]));

        // Gets the set of sender's notes and picks 2 of those.
        let sender_balance = storage.balances.at(from);
        spend_notes(&mut context, sender_balance, amount, from);

        let balance = storage.balances.at(to);
        send_note(&mut context, balance, amount, to);

        context.finish()
    }

    // Shield creates a way for a user to move tokens from the public context into the private context.
    // uint256(keccak256("shield(field,field)")) >> 224 -> 845411215
    open fn shield(
        inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([amount, secretHash]));

        let storage = Storage::init();
        let public_balances = storage.public_balances;
        let pending_shields = storage.pending_shields;

        // Decrease user's balance.
        let sender = context.msg_sender();
        let sender_balance = public_balances.at(sender);
        let current_sender_balance: Field = sender_balance.read();

        assert(current_sender_balance as u120 >= amount as u120);

        // User has sufficient balance so we decrement it by `amount`
        let _void1 = sender_balance.write(current_sender_balance - amount);

        // Create a commitment to the "amount" using the "secretHash"
        // and insert it into the set of "pending_shields" and therefore
        // (eventually) the private data tree.
        let mut note = TransparentNote::new(amount, secretHash);
        pending_shields.insert_from_public(&mut context, &mut note);

        context.finish()
    }

    // The shield function takes a public balance, and creates a commitment containing the amount of tokens
    // in the private context.
    // uint256(keccak256("redeemShield(field,field,field)")) >> 224 -> 2260077221
    fn redeemShield(
        inputs: PrivateContextInputs,
        amount: Field,
        secret: Field,
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let pending_shields = storage.pending_shields;

        let mut context = PrivateContext::new(inputs, abi::hash_args([
            amount, secret, owner
        ]));

        let public_note = TransparentNote::new_from_secret(amount, secret);

        // Ensure that the note exists in the tree and remove it.
        pending_shields.assert_contains_and_remove(&mut context, public_note);

        // Mint the tokens
        let balance = storage.balances.at(owner);
        send_note(&mut context, balance, amount, owner);

        context.finish()
    }

    // uint256(keccak256("unshieldTokens(field,field,field)")) >> 224 -> 2423803924
    fn unshieldTokens(
        inputs: PrivateContextInputs,
        from: Field,
        to: Field,
        amount: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            from, to, amount,
        ]));

        // Remove user balance
        let sender_balance = storage.balances.at(from);
        spend_notes(&mut context, sender_balance, amount, from);

        // enqueue a public function to perform the public state update.
        let thisAddress = context.this_address();

        // addUnshieldedBalance selector (in decimal)
        // recompute by: `cast keccak addUnshieldedBalance(field,field)` -> convert to decimal
        let pubEntryPointSelector = 753269941;
        let _callStackItem1 = context.call_public_function(thisAddress, pubEntryPointSelector, [amount, to]);

        context.finish()
    }

    open internal fn addUnshieldedBalance(
        inputs: PublicContextInputs,
        amount: Field,
        to: Field,
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([amount, to]));

        let storage = Storage::init();

        let to_balance = storage.public_balances.at(to);
        let current_balance = to_balance.read();
        let new_balance = current_balance + amount;
        to_balance.write(new_balance);

        context.finish()
    }

    unconstrained fn balance_of(
        owner: Field,
    ) -> Field {
        let storage = Storage::init();
        let owner_balance = storage.balances.at(owner);

        balance_utils::get_balance(owner_balance)
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        if (storage_slot == 2) {
            note_utils::compute_note_hash_and_nullifier(TransparentNoteMethods, note_header, preimage)
        } else {
            note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
        }
    }

    unconstrained fn total_supply() -> Field {
        let storage = Storage::init();
        storage.total_supply.read()
    }

    unconstrained fn public_balance_of(
        owner: Field,
    ) -> Field {
        let storage = Storage::init();
        storage.public_balances.at(owner).read()
    }

    unconstrained fn public_allowance(
        owner: Field,
        spender: Field,
    ) -> Field {
        let storage = Storage::init();
        storage.public_allowances.at(owner).at(spender).read()
    }
}
