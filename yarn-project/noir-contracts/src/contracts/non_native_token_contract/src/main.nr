mod hash;
mod transparent_note;

// Represents an ERC20 token bridged from L1 to L2 via `l1-contracts/test/portals/TokenPortal.sol`. The bridged user
// balance can be stored in both public and private domain. If the balance is stored in the private domain then it
// is represented as a set of notes in the private data tree. If the balance is stored in the public domain then it is
// represented as a single value in the public data tree.
//
// It is also used to test insertion of a note from public domain into the private data tree. How this works is that
// a user provides a secret hash as an argument on the input of the public function, and the contract then inserts
// a note into the private data tree once the public call is processed. A note can later on be spent by providing
// the secret hash preimage on the input of a private function call.
//
// This flow will be typically used when performing a token swap:
// 1. A calls a private function which enqueues a public call to the DEX contract (with the secret hash on input).
// 2. The sequencer performs the public call to the DEX and the note gets inserted into the private data tree.
// 3. At this point a recipient can spend the note in a private function assuming he/she knows the secret.
contract NonNativeToken {
    // Libs
    use dep::std::option::Option;
    use dep::value_note::{
        balance_utils,
        utils::{increment, decrement},
        value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
    };

    use crate::transparent_note::{
        TransparentNote,
        TransparentNoteMethods,
        TRANSPARENT_NOTE_LEN,
    };

    use crate::hash::{get_mint_content_hash, get_withdraw_content_hash};

    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        oracle::compute_selector::compute_selector,
        state_vars::{map::Map, public_state::PublicState, set::Set},
        types::type_serialisation::field_serialisation::{
            FieldSerialisationMethods, FIELD_SERIALISED_LEN,
        },
    };

    struct Storage {
        balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
        pending_shields: Set<TransparentNote, TRANSPARENT_NOTE_LEN>,
        public_balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
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
                pending_shields: Set::new(context, 2, TransparentNoteMethods),
                public_balances: Map::new(
                    context,
                    3,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            FieldSerialisationMethods,
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

    // Mint Private Function
    // This mint function differs to the typical token mint function as it only allows minting
    // upon consuming valid messages from a token portal contract
    // docs:start:non_native_token_mint
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
    // docs:end:non_native_token_mint

    // Withdraws using user's private balance.
    // @dev Destroys 2 of user's notes and sends a message to the L1 portal contract. That message can then be consumed
    //      by calling the `withdraw` function on the L1 portal contract (assuming the args are set correctly).
    // docs:start:non_native_token_withdraw
    #[aztec(private)]
    fn withdraw(
        amount: Field,
        sender: Field, // TODO: Should verify sender.
        recipient: Field, // ethereum address in the field
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let sender_balance = storage.balances.at(sender);
        decrement(sender_balance, amount, sender);

        let content = get_withdraw_content_hash(amount, recipient, callerOnL1);
        context.message_portal(content);
    }
    // docs:end:non_native_token_withdraw

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

        if (current_sender_balance as u120) > (amount as u120) {
            // User has sufficient balance so we decrement it by `amount`
            let _void1 = sender_balance.write(current_sender_balance - amount);
        }
        // TODO: Revert if there is not enough balance

        let content = get_withdraw_content_hash(amount, recipient, callerOnL1);

        // Emit the l2 to l1 message
        context.message_portal(content);
    }


    // Transfers `amount` of tokens from `sender`'s private balance to a `recipient`.
    // Note: Copied from PrivateToken
    #[aztec(private)]
    fn transfer(
        amount: Field,
        recipient: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));
        let sender = context.msg_sender();

        // Gets the set of sender's notes and picks 2 of those.
        let sender_balance = storage.balances.at(sender);
        decrement(sender_balance, amount, sender);

        let balance = storage.balances.at(recipient);
        increment(balance, amount, recipient);
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

        let public_note = TransparentNote::new_from_secret(amount, secret);

        // Ensure that the note exists in the tree and remove it.
        pending_shields.assert_contains_and_remove_publicly_created(public_note);

        // Mint the tokens
        let balance = storage.balances.at(owner);
        increment(balance, amount, owner);
    }

    #[aztec(private)]
    fn unshieldTokens(
        amount: Field,
        recipient: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));
        let owner = context.msg_sender();

        // Remove user balance
        let sender_balance = storage.balances.at(owner);
        decrement(sender_balance, amount, owner);

        // enqueue a public function to perform the public state update.
        let thisAddress = context.this_address();

        let addUnshieldedBalance = compute_selector("addUnshieldedBalance(Field,Field)");
        let _callStackItem1 = context.call_public_function(thisAddress, addUnshieldedBalance, [amount, recipient]);
    }

    #[aztec(public)]
    fn addUnshieldedBalance(
        amount: Field,
        recipient: Field,
    ) {
        let storage = Storage::init(Context::public(&mut context));

        let recipient_balance = storage.public_balances.at(recipient);
        let current_balance = recipient_balance.read();
        let new_balance = current_balance + amount;
        recipient_balance.write(new_balance);
    }

    unconstrained fn getBalance(
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
        if (storage_slot == 2) {
            note_utils::compute_note_hash_and_nullifier(TransparentNoteMethods, note_header, preimage)
        } else {
            note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
        }
    }

    unconstrained fn publicBalanceOf(
        owner: Field,
    ) -> Field {
        let storage = Storage::init(Context::none());
        storage.public_balances.at(owner).read()
    }
}
