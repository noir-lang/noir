mod claim_note;
mod interface;

contract PrivateTokenAirdrop {
    // Libs
    use dep::std::option::Option;
    use dep::value_note::{
        balance_utils,
        utils::{increment, decrement, decrement_by_at_most},
        value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
    };
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        state_vars::{map::Map, set::Set},
        note::{
            note_getter_options::NoteGetterOptions,
            note_header::NoteHeader,
            utils as note_utils,
        },
        log::emit_unencrypted_log,
    };

    use crate::claim_note::{ClaimNote, ClaimNoteMethods, CLAIM_NOTE_LEN};
    use crate::interface::PrivateTokenAirdropPrivateContextInterface;


    struct Storage {
        balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
        claims: Set<ClaimNote, CLAIM_NOTE_LEN>,
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
                claims: Set::new(context, 2, ClaimNoteMethods),
            }
        }
    }

    // Constructs the contract and sets `initial_supply` which is fully owned by `owner`.
    #[aztec(private)]
    fn constructor(
        initial_supply: Field, 
        owner: Field
    ) {
        let storage = Storage::init(Context::private(&mut context));

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        if (initial_supply != 0) {
            increment(owner_balance, initial_supply, owner);
        }
    }

    // Mints `amount` of tokens to `owner`.
    #[aztec(private)]
    fn mint(
        amount: Field, 
        owner: Field
    ) {
        let storage = Storage::init(Context::private(&mut context));

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        increment(owner_balance, amount, owner);
    }

    // Burn `amount` of tokens from `owner`'s balance.
    // This is an example of a recursive function. It calls itself until enough notes have been burned to burn the `amount`.
    #[aztec(private)]
    fn burn(
        amount: Field,
        owner: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));
        let msg_sender = context.msg_sender();
        let this_address = context.this_address();

        // If this function is not called by another function in the same contract, the owner must be msg_sender.
        if msg_sender != this_address {
            assert(owner == msg_sender);
        }

        let balance = storage.balances.at(owner);
        let sum = decrement_by_at_most(balance, amount, owner);

        // If sum is 0, there are no notes to be burned.
        assert(sum != 0);

        if sum != amount {
            // The destroyed notes' sum is not enough. Keep burning.
            let amount_to_burn = amount - sum;
            let this = PrivateTokenAirdropPrivateContextInterface::at(this_address);
            let _res = this.burn(&mut context, amount_to_burn, owner);
        }
    }

    // Transfers `amount` of tokens from `sender` to a `recipient`.
    #[aztec(private)]
    fn transfer(
        amount: Field, 
        recipient: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let sender = context.msg_sender();
        let sender_balance = storage.balances.at(sender);

        let sum = decrement_by_at_most(sender_balance, amount, sender);

        // If sum is 0, there are no notes to be used.
        assert(sum != 0);

        if sum != amount {
            // The destroyed notes' sum is not enough for the transfer.
            // Burn the remaining amount.
            // We only call burn() when decrement_by_at_most() didn't destroy enough notes.
            let amount_to_burn = amount - sum;
            let this_address = context.this_address();
            let this = PrivateTokenAirdropPrivateContextInterface::at(this_address);
            let _res = this.burn(&mut context, amount_to_burn, sender);
        }

        // Create a new note for the recipient.
        let recipient_balance = storage.balances.at(recipient);
        increment(recipient_balance, amount, recipient);
    }

    #[aztec(private)]
    fn createClaims(
        amounts: [Field; 2],
        secrets: [Field; 2],
    ) {
        let storage = Storage::init(Context::private(&mut context));
        let sender = context.msg_sender();

        // Pick from the set of sender's notes to spend amount.
        let sender_balance = storage.balances.at(sender);
        let total = amounts[0] + amounts[1];
        decrement(sender_balance, total, sender);

        // Create claim notes.
        let claims = storage.claims;
        let mut note0 = ClaimNote::new(amounts[0], secrets[0]);
        let mut note1 = ClaimNote::new(amounts[1], secrets[1]);

        // Insert the new claim notes to the set.
        claims.insert(&mut note0);
        claims.insert(&mut note1);
    }

    #[aztec(private)]
    fn claim(
        amount: Field,
        secret: Field,
        owner: Field
    ) {
        let storage = Storage::init(Context::private(&mut context));

        // Remove the claim note if it exists in the set.
        let note = ClaimNote::new(amount, secret);
        storage.claims.assert_contains_note_and_remove(note);

        // Send the value note.
        let balance = storage.balances.at(owner);
        increment(balance, amount, owner);
    }

    // Transfers `amounts` of tokens from `sender` to 3 `recipients`. 
    // Aztec only allows `MAX_NEW_COMMITMENTS_PER_CALL = 4` notes per tx =>
    // 1 new note for sender's new balance and 1 note each for 3 recipients (for their new balance)
    #[aztec(private)]
    fn batchTransfer(
        sender: Field,
        amounts: [Field; 3],
        recipients: [Field; 3],
        spend_note_offset: u32,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        // Gets the set of sender's notes and picks 4 of those based on the offset.
        // Spends the first of those 4 notes.
        let sender_balance = storage.balances.at(sender);
        let total = amounts[0] + amounts[1] + amounts[2];

        let options = NoteGetterOptions::new().set_limit(1).set_offset(spend_note_offset);
        let opt_notes = sender_balance.get_notes(options);

        // The note should always exist.
        let note = opt_notes[0].unwrap_unchecked();

        assert(note.owner == sender);

        sender_balance.remove(note);
        
        // Assert that the note chosen to spend has enough funds.
        assert(note.value as u64 >= total as u64);

        // Add the change value back to the owner's balance.
        let change_value = note.value - total;
        increment(sender_balance, change_value, sender);

        // Creates new note for the recipient.
        for i in 0..3 {
            let recipient_balance = storage.balances.at(recipients[i]);
            increment(recipient_balance, amounts[i], recipients[i]);
        }

        // Also emit an unencrypted log, eg. "Coins transferred"
        // In this example, we emit the first output note's commitment to ensure that the unencrypted log
        // for each call to this function is distinct. This is done to detect any issues while collecting 
        // logs when building a transaction. See: https://github.com/AztecProtocol/aztec-packages/issues/987
        emit_unencrypted_log(&mut context, context.new_commitments.storage[0]);
    }

    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    unconstrained fn getBalance(
        owner: Field,
    ) -> Field {
        let storage = Storage::init(Context::none());

        // Get the set of notes owned by the user.
        let owner_balance = storage.balances.at(owner);

        // Return the sum of all notes in the set.
        balance_utils::get_balance(owner_balance)
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        if (storage_slot == 2) {
            note_utils::compute_note_hash_and_nullifier(ClaimNoteMethods, note_header, preimage)
        } else {
            note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
        }
    }
}
