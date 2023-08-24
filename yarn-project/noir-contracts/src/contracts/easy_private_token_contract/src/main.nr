// docs:start:easy_private_token_contract
mod storage;

contract EasyPrivateToken {
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
    fn constructor(
        /*********************************/
        inputs: PrivateContextInputs,
        /*********************************/
        initial_supply: u120, 
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([initial_supply as Field, owner]));
        let storage = Storage::init();
        let balances = storage.balances;

        balances.at(owner).add(&mut context, initial_supply, owner);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Mints `amount` of tokens to `owner`.
    fn mint(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: u120, 
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount as Field, owner]));
        let storage = Storage::init();
        let balances = storage.balances;

        balances.at(owner).add(&mut context, amount, owner);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
    }

    // Transfers `amount` of tokens from `sender` to a `recipient`.
    fn transfer(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        amount: u120, 
        sender: Field, 
        recipient: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([amount as Field, sender, recipient]));
        let storage = Storage::init();
        let balances = storage.balances;

        balances.at(sender).sub(&mut context, amount, sender);

        balances.at(recipient).add(&mut context, amount, recipient);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel..
        context.finish()
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
