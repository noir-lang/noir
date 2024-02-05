// docs:start:easy_private_token_contract
contract EasyPrivateToken {
    use dep::aztec::protocol_types::address::AztecAddress;
    use dep::std::option::Option;
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        state_vars::map::Map,
    };
    use dep::value_note::{
            balance_utils,
            value_note::{
                ValueNote,
                VALUE_NOTE_LEN,
            },
    };
    use dep::easy_private_state::easy_private_state::EasyPrivateUint;

    struct Storage {
        balances: Map<AztecAddress, EasyPrivateUint>,
    }

    /**
     * initialize the contract's initial state variables.
     */
    #[aztec(private)]
    fn constructor(initial_supply: u120, owner: AztecAddress) {
        let balances = storage.balances;

        balances.at(owner).add(initial_supply, owner);
    }

    // Mints `amount` of tokens to `owner`.
    #[aztec(private)]
    fn mint(amount: u120, owner: AztecAddress) {
        let balances = storage.balances;

        balances.at(owner).add(amount, owner);
    }

    // Transfers `amount` of tokens from `sender` to a `recipient`.
    #[aztec(private)]
    fn transfer(amount: u120, sender: AztecAddress, recipient: AztecAddress) {
        let balances = storage.balances;

        balances.at(sender).sub(amount, sender);
        balances.at(recipient).add(amount, recipient);
    }

    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    unconstrained fn getBalance(owner: AztecAddress) -> pub Field {
        let balances = storage.balances;

        // Return the sum of all notes in the set.
        balance_utils::get_balance(balances.at(owner).set)
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; VALUE_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNote::deserialize_content, note_header, serialized_note)
    }
}
// docs:end:easy_private_token_contract
