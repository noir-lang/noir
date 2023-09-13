// Testing token that can be bridged in and out.
// TODOS:
// - Add role based access control to mint functions
// - Add function for managing roles
// - Add public self-burn function for users to burn their own tokens
contract NativeToken {
    // Libs
    use dep::std::option::Option;
    use dep::value_note::{
        balance_utils,
        utils::{increment, decrement},
        value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
    };
    use dep::std;
    use dep::aztec::{
        constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD,
        context::{PrivateContext, PublicContext, Context},
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        oracle::{
            compute_selector::compute_selector,
            logs::emit_unencrypted_log,
        },
        state_vars::{
            map::Map,
            public_state::PublicState,
            set::Set,
        },
        types::type_serialisation::field_serialisation::{
            FieldSerialisationMethods,
            FIELD_SERIALISED_LEN,
        },
        auth::assert_valid_message_for,
    };
    use dep::non_native::{
        hash::{get_mint_content_hash, get_withdraw_content_hash},
        transparent_note::{
            TransparentNote,
            TransparentNoteMethods,
            TRANSPARENT_NOTE_LEN,
        },
    };

    struct Storage {
        balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
        total_supply: PublicState<Field, FIELD_SERIALISED_LEN>,
        pending_shields: Set<TransparentNote, TRANSPARENT_NOTE_LEN>,
        public_balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
        public_allowances: Map<Map<PublicState<Field, FIELD_SERIALISED_LEN>>>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                balances: Map::new(
                    context,
                    1, // Storage slot
                    |context, slot| {
                        Set::new(context, slot, ValueNoteMethods)
                    },
                ),
                total_supply: PublicState::new(
                    context,
                    2,
                    FieldSerialisationMethods,
                ),
                pending_shields: Set::new(context, 3, TransparentNoteMethods),
                public_balances: Map::new(
                    context,
                    4,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            FieldSerialisationMethods,
                        )
                    },
                ),
                public_allowances: Map::new(
                    context,
                    5,
                    |context, s1| {
                        Map::new(
                            context,
                            s1,
                            |context, s2| {
                                PublicState::new(
                                    context,
                                    s2,
                                    FieldSerialisationMethods,
                                )
                            },
                        )
                    },
                ),
            }
        }
    }

    #[aztec(private)]
    fn constructor(
        initial_supply: Field,
        owner: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let balance = storage.balances.at(owner);
        increment(balance, initial_supply, owner);
    }

    #[aztec(public)]
    fn owner_mint_pub(
        to: Field,
        amount: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));
        let new_balance = storage.public_balances.at(to).read() + amount;
        storage.public_balances.at(to).write(new_balance);
        storage.total_supply.write(storage.total_supply.read() + amount);
        let _hash = emit_unencrypted_log("Coins minted");

        1
    }

    #[aztec(public)]
    fn owner_mint_priv(
        amount: Field,
        secret_hash: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));
        let pending_shields = storage.pending_shields;

        let mut note = TransparentNote::new(amount, secret_hash);
        pending_shields.insert_from_public(&mut note);

        storage.total_supply.write(storage.total_supply.read() + amount);
    
        1
    }

    // Mint Private Function
    // This mint function differs to the typical token mint function as it only allows minting
    // upon consuming valid messages from a token portal contract
    #[aztec(private)]
    fn mint(
        amount: Field,
        owner: Field,
        // This field should be hidden
        msg_key: Field,
        secret: Field,
        canceller: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let content_hash = get_mint_content_hash(amount, owner, canceller);

        // Get the l1 message from an oracle call
        context.consume_l1_to_l2_message(inputs, msg_key, content_hash, secret);

        let balance = storage.balances.at(owner);
        increment(balance, amount, owner);
    }

    // Withdraws using user's private balance.
    // @dev Destroys 2 of user's notes and sends a message to the L1 portal contract. That message can then be consumed
    //      by calling the `withdraw` function on the L1 portal contract (assuming the args are set correctly).
    #[aztec(private)]
    fn withdraw(
        amount: Field,
        sender: Field,
        recipient: Field, // ethereum address in the field
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let sender_balance = storage.balances.at(sender);
        decrement(sender_balance, amount, sender);

        let content = get_withdraw_content_hash(amount, recipient, callerOnL1);
        context.message_portal(content);
    }

    // Mint Public Function
    // This mint function differs to the typical token mint function as it only allows minting
    // upon consuming valid messages from a token portal contract
    #[aztec(public)]
    fn mintPublic(
        amount: Field,
        owner_address: Field,
        // This field should be hidden
        msg_key: Field,
        secret: Field,
        canceller: Field,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));
        let public_balances = storage.public_balances;

        let content_hash = get_mint_content_hash(amount, owner_address, canceller);

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(msg_key, content_hash, secret);

        // Update the public balance
        let recipient_balance = public_balances.at(owner_address);
        let new_balance = recipient_balance.read() + amount;
        recipient_balance.write(new_balance);

        // Push the return value into the context
        new_balance
    }

    // Withdraws using user's public balance.
    #[aztec(public)]
    fn withdrawPublic(
        amount: Field,
        recipient: Field,
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) {
        let storage = Storage::init(Context::public(&mut context));
        let public_balances = storage.public_balances;

        let sender = context.msg_sender();
        let sender_balance = public_balances.at(sender);

        let current_sender_balance: Field = sender_balance.read();

        assert(current_sender_balance as u120 >= amount as u120);
        let content = get_withdraw_content_hash(amount, recipient, callerOnL1);

        // Emit the l2 to l1 message
        context.message_portal(content);
    }

    #[aztec(public)]
    fn approve(
        spender: Field,
        allowance: Field,
    ) {
        let storage = Storage::init(Context::public(&mut context));
        storage.public_allowances.at(context.msg_sender()).at(spender).write(allowance);
    }

    #[aztec(public)]
    fn transfer_pub(
        to: Field,
        amount: Field,
    ) {
        let storage = Storage::init(Context::public(&mut context));

        // Decrease user's balance.
        let sender = context.msg_sender();
        let sender_balance = storage.public_balances.at(sender);
        let current_sender_balance: Field = sender_balance.read();
        assert(current_sender_balance as u120 >= amount as u120, "Balance too low");

        let to_balance = storage.public_balances.at(to);
        let current_to_balance: Field = to_balance.read();

        // User has sufficient balance so we decrement it by `amount`
        sender_balance.write(current_sender_balance - amount);
        to_balance.write(current_to_balance + amount);

        let _hash = emit_unencrypted_log("Coins transferred");
    }

    #[aztec(public)]
    fn transfer_from_pub(
        from: Field,
        to: Field,
        amount: Field,
    ) {
        let storage = Storage::init(Context::public(&mut context));

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
    }

    // Transfers `amount` of tokens from `sender`'s private balance to a `recipient`.
    // Note: Copied from PrivateToken
    #[aztec(private)]
    fn transfer(
        from: Field,
        to: Field,
        amount: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        // Gets the set of sender's notes and picks 2 of those.
        let sender_balance = storage.balances.at(from);
        decrement(sender_balance, amount, from);

        let balance = storage.balances.at(to);
        increment(balance, amount, to);
    }

    // Shield creates a way for a user to move tokens from the public context into the private context.
    #[aztec(public)]
    fn shield(
        amount: Field,
        secretHash: Field,
    ) {
        let storage = Storage::init(Context::public(&mut context));
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
        pending_shields.insert_from_public(&mut note);
    }

    // The shield function takes a public balance, and creates a commitment containing the amount of tokens
    // in the private context.
    #[aztec(private)]
    fn redeemShield(
        amount: Field,
        secret: Field,
        owner: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));
        let pending_shields = storage.pending_shields;

        let mut public_note = TransparentNote::new_from_secret(amount, secret);

        // Ensure that the note exists in the tree and remove it.
        pending_shields.assert_contains_and_remove_publicly_created(&mut public_note);

        // Mint the tokens
        let balance = storage.balances.at(owner);
        increment(balance, amount, owner);
    }

    #[aztec(private)]
    fn unshieldTokens(
        from: Field,
        to: Field,
        amount: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        // If `from != sender` then we use the is_valid function to check that the message is approved.
        if (from != context.msg_sender()) {
            // Compute the message hash, should follow eip-712 more here.
            // @todo @lherskind, probably need a separate generator index and address of the 
            // @todo @lherskind Currently this can be used multiple times since it is not nullified. 
            //                  We can do a simple nullifier to handle that in here. Spends only 32 bytes onchain.
            // @todo @LHerskind Is to be solved as part of https://github.com/AztecProtocol/aztec-packages/issues/1743
            let message_field: Field = std::hash::pedersen_with_separator([
                    compute_selector("unshieldTokens(Field,Field,Field)"),
                    from, 
                    to, 
                    amount
                ], 
                GENERATOR_INDEX__SIGNATURE_PAYLOAD
            )[0];
            
            assert_valid_message_for(&mut context, from, message_field);
        }

        // Reduce user balance
        let sender_balance = storage.balances.at(from);
        decrement(sender_balance, amount, from);

        // enqueue a public function to perform the public state update.
        let thisAddress = context.this_address();

        let addUnshieldedBalance = compute_selector("addUnshieldedBalance(Field,Field)");
        let _callStackItem1 = context.call_public_function(thisAddress, addUnshieldedBalance, [amount, to]);
    }

    #[aztec(public)]
    internal fn addUnshieldedBalance(
        amount: Field,
        to: Field,
    ) {
        let storage = Storage::init(Context::public(&mut context));

        let to_balance = storage.public_balances.at(to);
        let current_balance = to_balance.read();
        let new_balance = current_balance + amount;
        to_balance.write(new_balance);
    }

    unconstrained fn balance_of(
        owner: Field,
    ) -> Field {
        let storage = Storage::init(Context::none());
        let owner_balance = storage.balances.at(owner);

        balance_utils::get_balance(owner_balance)
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        if (storage_slot == 3) {
            note_utils::compute_note_hash_and_nullifier(TransparentNoteMethods, note_header, preimage)
        } else {
            note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
        }
    }

    unconstrained fn total_supply() -> Field {
        let storage = Storage::init(Context::none());
        storage.total_supply.read()
    }

    unconstrained fn public_balance_of(
        owner: Field,
    ) -> Field {
        let storage = Storage::init(Context::none());
        storage.public_balances.at(owner).read()
    }

    unconstrained fn public_allowance(
        owner: Field,
        spender: Field,
    ) -> Field {
        let storage = Storage::init(Context::none());
        storage.public_allowances.at(owner).at(spender).read()
    }
}
