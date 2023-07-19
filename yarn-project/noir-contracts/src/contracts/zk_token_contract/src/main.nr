mod claim_note;
mod storage;

contract ZkToken {
    // Libs
    use dep::value_note::{
        balance_utils,
        utils::{send_note, spend_notes},
        value_note::{VALUE_NOTE_LEN, ValueNoteInterface},
    };

    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::context::Context;
    use dep::aztec::note::{
        note_getter::NoteGetterOptions,
        note_header::{NoteHeader},
        utils::compute_note_hash_and_nullifier,
    };
    use dep::aztec::types::point::Point;
    use dep::aztec::state_vars::map::Map;
    use dep::aztec::state_vars::set::Set;
    use dep::aztec::log::emit_encrypted_log;
    use dep::aztec::log::emit_unencrypted_log;

    use crate::storage::Storage;
    use crate::claim_note::{ClaimNote, ClaimNoteInterface};


    // Constructs the contract and sets `initial_supply` which is fully owned by `owner`.
    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        initial_supply: Field, 
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = Context::new(inputs, abi::hash_args([initial_supply, owner]));

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        if (initial_supply != 0) {
            context = send_note(context, owner_balance, initial_supply, owner);
            context = emit_unencrypted_log(context, "Balance set in constructor");
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
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = Context::new(inputs, abi::hash_args([amount, owner]));

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        context = send_note(context, owner_balance, amount, owner);
        context = emit_unencrypted_log(context, "Coins minted");

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
        let mut context = Context::new(inputs, abi::hash_args([amount, sender, recipient]));

        // Pick from the set of sender's notes to spend amount.
        let sender_balance = storage.balances.at(sender);
        context = spend_notes(context, sender_balance, amount, sender);

        // Creates new note for the recipient.
        let recipient_balance = storage.balances.at(recipient);
        context = send_note(context, recipient_balance, amount, recipient);
        
        context = emit_unencrypted_log(context, "Coins transferred");

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
        sender: Field, 
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = Context::new(inputs, abi::hash_args([amounts[0], amounts[1], secrets[0], secrets[1], sender]));

        // Pick from the set of sender's notes to spend amount.
        let sender_balance = storage.balances.at(sender);
        let total = amounts[0] + amounts[1];
        context = spend_notes(context, sender_balance, total, sender);

        // Create claim notes.
        let claims = storage.claims;
        let note0 = ClaimNote::new(amounts[0], secrets[0]);
        let note1 = ClaimNote::new(amounts[1], secrets[1]);

        // Insert the new claim notes to the set.
        context = claims.insert(context, note0);
        context = claims.insert(context, note1);

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
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = Context::new(inputs, abi::hash_args([amount, secret, owner]));

        // Remove the claim note if it exists in the set.
        let mut note = ClaimNote::new(amount, secret);
        let (new_context, note_with_header) = storage.claims.has(context, note);
        context = new_context;
        note = note_with_header;
        context = storage.claims.remove(context, note);

        // Send the value note.
        let balance = storage.balances.at(owner);
        context = send_note(context, balance, amount, owner);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
    }

    /// ABI getBalance type "unconstrained"
    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    fn getBalance(
        owner: Field,
    ) -> pub Field {
        let storage = Storage::init();

        // Get the set of notes owned by the user.
        let owner_balance = storage.balances.at(owner);

        // Return the sum of all notes in the set.
        balance_utils::get_balance(owner_balance.storage_slot)
    }

    /// ABI stev type "unconstrained"
    fn stev(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> pub [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        if (storage_slot == 2) {
            compute_note_hash_and_nullifier(ClaimNoteInterface, note_header, preimage)
        } else {
            compute_note_hash_and_nullifier(ValueNoteInterface, note_header, preimage)
        }
    }
}
