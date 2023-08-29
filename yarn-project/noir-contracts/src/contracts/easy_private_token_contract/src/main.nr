// docs:start:easy_private_token_contract
mod storage;

contract EasyPrivateToken {
    use dep::aztec::abi::Hasher;
    use dep::value_note::{
            balance_utils,
            value_note::{
                ValueNoteMethods,
                VALUE_NOTE_LEN,
            },
    };

    use dep::aztec::{
        abi,
        abi::PrivateContextInputs,
        context::PrivateContext,
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
    };

    use crate::storage::Storage;

    /**
     * Initialise the contract's initial state variables.
     */ 
    #[aztec(private)]
    fn constructor(
        initial_supply: u120, 
        owner: Field,
    ) {
        let storage = Storage::init();
        let balances = storage.balances;

        balances.at(owner).add(&mut context, initial_supply, owner);
    }

    // Mints `amount` of tokens to `owner`.
    #[aztec(private)]
    fn mint(
        amount: u120, 
        owner: Field,
    )  {
        let storage = Storage::init();
        let balances = storage.balances;

        balances.at(owner).add(&mut context, amount, owner);
    }

    // Transfers `amount` of tokens from `sender` to a `recipient`.
    #[aztec(private)]
    fn transfer(
        amount: u120, 
        sender: Field, 
        recipient: Field,
    ) {
        let storage = Storage::init();
        let balances = storage.balances;

        balances.at(sender).sub(&mut context, amount, sender);
        balances.at(recipient).add(&mut context, amount, recipient);
    }

    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    unconstrained fn getBalance(
        owner: Field,
    ) -> Field {
        let storage = Storage::init();
        let balances = storage.balances;

        // Return the sum of all notes in the set.
        balance_utils::get_balance(balances.at(owner).set)
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, preimage)
    }
}
// docs:end:easy_private_token_contract
