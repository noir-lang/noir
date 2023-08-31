mod storage;

contract PrivateToken {
    // Libs
    use dep::value_note::{
        balance_utils,
        utils::{increment, decrement},
        value_note::{VALUE_NOTE_LEN, ValueNoteMethods},
    };

    use dep::aztec::note::{
        note_header::NoteHeader,
        utils as note_utils,
    };

    // docs:start:storage-import
    use crate::storage::Storage;
    // docs:end:storage-import

    // docs:start:constructor
    // Constructs the contract and sets `initial_supply` which is fully owned by `owner`.
    #[aztec(private)]
    fn constructor(
        initial_supply: Field, 
        owner: Field
    )  {
        let storage = Storage::init();
        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        if (initial_supply != 0) {
            increment(&mut context, owner_balance, initial_supply, owner);
        }
    }
    // docs:end:constructor

    // docs:start:mint
    // Mints `amount` of tokens to `owner`.
    #[aztec(private)]
    fn mint(
        amount: Field, 
        owner: Field
    )  {
        let storage = Storage::init();

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        increment(&mut context, owner_balance, amount, owner);
    }
    // docs:end:mint

    // docs:start:transfer
    // Transfers `amount` of tokens from msg_sender to a `recipient`.
    #[aztec(private)]
    fn transfer(
        amount: Field, 
        recipient: Field,
    )  {
        let storage = Storage::init();
        let sender = context.msg_sender();

        // Pick from the set of sender's notes to spend amount.
        let sender_balance = storage.balances.at(sender);
        decrement(&mut context, sender_balance, amount, sender);

        // Creates new note for the recipient.
        let recipient_balance = storage.balances.at(recipient);
        increment(&mut context, recipient_balance, amount, recipient);
    }
    // docs:end:transfer

    // docs:start:getBalance
    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    unconstrained fn getBalance(
        owner: Field,
    ) -> Field {
        let storage = Storage::init();

        // Get the set of notes owned by the user.
        let owner_balance = storage.balances.at(owner);

        // Return the sum of all notes in the set.
        balance_utils::get_balance(owner_balance)
    }
    // docs:end:getBalance

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    // docs:start:compute_note_hash_and_nullifier
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
    }
    // docs:end:compute_note_hash_and_nullifier
}
