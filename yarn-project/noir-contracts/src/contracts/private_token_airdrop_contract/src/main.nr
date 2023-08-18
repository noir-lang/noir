mod claim_note;
mod storage;

contract PrivateTokenAirdrop {
    // Libs
    use dep::value_note::{
        balance_utils,
        utils::{send_note, spend_notes, send_notes, spend_one_note},
        value_note::{VALUE_NOTE_LEN, ValueNoteMethods},
    };

    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::context::PrivateContext;
    use dep::aztec::note::{
        note_header::NoteHeader,
        utils as note_utils,
    };
    use dep::aztec::log::emit_unencrypted_log;

    use crate::storage::Storage;
    use crate::claim_note::{ClaimNote, ClaimNoteMethods};


    // Constructs the contract and sets `initial_supply` which is fully owned by `owner`.
    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        initial_supply: Field, 
        owner: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([initial_supply, owner]));

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        if (initial_supply != 0) {
            send_note(&mut context, owner_balance, initial_supply, owner);
            emit_unencrypted_log(&mut context, "Balance set in constructor");
        }

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
        owner: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, owner]));

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        send_note(&mut context, owner_balance, amount, owner);
        emit_unencrypted_log(&mut context, "Coins minted");

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
        sender: Field, 
        recipient: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, sender, recipient]));

        // Pick from the set of sender's notes to spend amount.
        let sender_balance = storage.balances.at(sender);
        spend_notes(&mut context, sender_balance, amount, sender);

        // Creates new note for the recipient.
        let recipient_balance = storage.balances.at(recipient);
        send_note(&mut context, recipient_balance, amount, recipient);
        
        emit_unencrypted_log(&mut context, "Coins transferred");

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
    }

    fn createClaims(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amounts: [Field; 2], 
        secrets: [Field; 2], 
        sender: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([amounts[0], amounts[1], secrets[0], secrets[1], sender]));

        // Pick from the set of sender's notes to spend amount.
        let sender_balance = storage.balances.at(sender);
        let total = amounts[0] + amounts[1];
        spend_notes(&mut context, sender_balance, total, sender);

        // Create claim notes.
        let claims = storage.claims;
        let mut note0 = ClaimNote::new(amounts[0], secrets[0]);
        let mut note1 = ClaimNote::new(amounts[1], secrets[1]);

        // Insert the new claim notes to the set.
        claims.insert(&mut context, &mut note0);
        claims.insert(&mut context, &mut note1);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
    }

    fn claim(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: Field,
        secret: Field,
        owner: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount, secret, owner]));

        // Remove the claim note if it exists in the set.
        let note = ClaimNote::new(amount, secret);
        storage.claims.assert_contains_note_and_remove(&mut context, note);

        // Send the value note.
        let balance = storage.balances.at(owner);
        send_note(&mut context, balance, amount, owner);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
    }

    // Transfers `amounts` of tokens from `sender` to 3 `recipients`. 
    // Aztec only allows `MAX_NEW_COMMITMENTS_PER_CALL = 4` notes per tx =>
    // 1 new note for sender's new balance and 1 note each for 3 recipients (for their new balance)
    fn batchTransfer(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        sender: Field, 
        amounts: [Field; 3], 
        recipients: [Field; 3],
        spend_note_offset: u32,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            sender, 
            amounts[0], 
            amounts[1], 
            amounts[2], 
            recipients[0], 
            recipients[1], 
            recipients[2], 
            spend_note_offset as Field,
        ]));

        // Gets the set of sender's notes and picks 4 of those based on the offset.
        // Spends the first of those 4 notes.
        let sender_balance = storage.balances.at(sender);
        let total = amounts[0] + amounts[1] + amounts[2];
        spend_one_note(&mut context, sender_balance, total, sender, spend_note_offset);

        // Creates new note for the recipient.
        let recipient1_balance = storage.balances.at(recipients[0]);
        let recipient2_balance = storage.balances.at(recipients[1]);
        let recipient3_balance = storage.balances.at(recipients[2]);

        send_notes(&mut context, [recipient1_balance, recipient2_balance, recipient3_balance], amounts, recipients);

        // Also emit an unencrypted log, eg. "Coins transferred"
        // In this example, we emit the first output note's commitment to ensure that the unencrypted log
        // for each call to this function is distinct. This is done to detect any issues while collecting 
        // logs when building a transaction. See: https://github.com/AztecProtocol/aztec-packages/issues/987
        emit_unencrypted_log(&mut context, context.new_commitments.storage[0]);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
    }

    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    unconstrained fn getBalance(
        owner: Field,
    ) -> Field {
        let storage = Storage::init();

        // Get the set of notes owned by the user.
        let owner_balance = storage.balances.at(owner);

        // Return the sum of all notes in the set.
        balance_utils::get_balance(owner_balance.storage_slot)
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
