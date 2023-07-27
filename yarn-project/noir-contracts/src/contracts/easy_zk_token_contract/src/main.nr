mod storage;

contract EasyZkToken {
    // Libs
    use dep::value_note::{
            balance_utils,
            value_note::{
                ValueNoteInterface,
                VALUE_NOTE_LEN,
            },
    };

    use dep::aztec::{
        abi,
        abi::PrivateContextInputs,
        context::Context,
        log::emit_unencrypted_log,
        note::{
            note_header::NoteHeader,
            utils::compute_note_hash_and_nullifier,
        },
    };

    use crate::storage::Storage;

    // Constructs the contract and sets `initial_supply` which is fully owned by `owner`.
    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: PrivateContextInputs,
        //*********************************/
        initial_supply: u120, 
        owner: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([initial_supply as Field, owner]));
        let storage = Storage::init();
        let balances = storage.balances;

        context = balances.at(owner).add(context, initial_supply, owner);

        context = emit_unencrypted_log(context, "Balance set in constructor");

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
        let mut context = Context::new(inputs, abi::hash_args([amount as Field, owner]));
        let storage = Storage::init();
        let balances = storage.balances;

        context = balances.at(owner).add(context, amount, owner);

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
        amount: u120, 
        sender: Field, 
        recipient: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([amount as Field, sender, recipient]));
        let storage = Storage::init();
        let balances = storage.balances;

        context = balances.at(sender).sub(context, amount, sender);

        context = balances.at(recipient).add(context, amount, recipient);
        
        context = emit_unencrypted_log(context, "Coins transferred");

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
        balance_utils::get_balance(balances.at(owner).storage_slot)
    }

    unconstrained fn stev(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        compute_note_hash_and_nullifier(ValueNoteInterface, note_header, preimage)
    }
}
