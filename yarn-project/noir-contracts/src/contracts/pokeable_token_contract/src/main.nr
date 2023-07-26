mod storage;
mod address_note;

contract PokeableToken {
    // Libs
    use dep::value_note::{
        balance_utils,
        utils::{send_note, spend_notes},
        value_note::{VALUE_NOTE_LEN, ValueNoteInterface},
        filter::get_2_notes,
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
    use dep::aztec::oracle::get_public_key::get_public_key_non_contract_account;
    use dep::aztec::log::emit_encrypted_log;

    use crate::storage::Storage;
    use crate::address_note::{AddressNote, AddressNoteInterface};

    // Constructs the contract and sets `initial_supply` which is fully owned by `sender`.
    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        initial_supply: Field, 
        sender: Field,
        recipient: Field,
        poker: Field, // Recipient of the address notes.
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = Context::new(inputs, abi::hash_args([initial_supply, sender, recipient, poker]));

        let sender_note = AddressNote::new(sender);
        let recipient_note = AddressNote::new(recipient);

        context = storage.sender.initialise(context, sender_note);
        context = storage.recipient.initialise(context, recipient_note);

        let pokerEncPubKey = get_public_key_non_contract_account(poker);

        let this = inputs.call_context.storage_contract_address;
        context = emit_encrypted_log(
            context,
            this,
            storage.sender.storage_slot,
            poker,
            pokerEncPubKey,
            sender_note.serialise(),
        );

        context = emit_encrypted_log(
            context,
            this,
            storage.recipient.storage_slot,
            poker,
            pokerEncPubKey,
            recipient_note.serialise(),
        );

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let sender_balance = storage.balances.at(sender);
        context = send_note(context, sender_balance, initial_supply, sender);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Transfers full balance of tokens from `sender` to a `recipient`.
    fn poke(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, 0); // Since there are no arguments, we can just 0 as the args hash.
        let storage = Storage::init();

        let (context_1, sender) = storage.sender.get_note(context);
        context = context_1;

        let (context_2, recipient) = storage.recipient.get_note(context);
        context = context_2;

        // Pick from the set of sender's notes.
        let sender_balance = storage.balances.at(sender.address);

        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        let (mut new_context, notes) = sender_balance.get_notes(context, options);
        context = new_context;

        let note1 = notes[0];
        let note2 = notes[1];

        let note_sum = note1.value + note2.value;

        // Removes the 2 notes from the sender's set of notes.
        context = sender_balance.remove(context, note1);
        context = sender_balance.remove(context, note2);

        // Create new note for the recipient.
        let recipient_balance = storage.balances.at(recipient.address);
        context = send_note(context, recipient_balance, note_sum, recipient.address);
        
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    unconstrained fn getBalance(
        sender: Field,
    ) -> pub Field {
        let storage = Storage::init();

        // Get the set of notes owned by the user.
        let sender_balance = storage.balances.at(sender);

        // Return the sum of all notes in the set.
        balance_utils::get_balance(sender_balance.storage_slot)
    }

    // stev function needs to be defined by every contract producing logs. Having it in all the contracts gives us the ability to
    // compute the note hash and nullifier differently for different kind of notes.
    // Note: "stev" is just a placeholder name before we come up with a better one.
    unconstrained fn stev(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> pub [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        if (storage_slot == 1) | (storage_slot == 2) {
            compute_note_hash_and_nullifier(AddressNoteInterface, note_header, preimage)
        } else {
            compute_note_hash_and_nullifier(ValueNoteInterface, note_header, preimage)
        }
    }
}
