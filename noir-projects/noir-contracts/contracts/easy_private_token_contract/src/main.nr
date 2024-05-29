// docs:start:easy_private_token_contract
contract EasyPrivateToken {
    use dep::aztec::prelude::{AztecAddress, NoteHeader, Map};
    use dep::value_note::balance_utils;
    use dep::easy_private_state::EasyPrivateUint;

    #[aztec(storage)]
    struct Storage {
        balances: Map<AztecAddress, EasyPrivateUint>,
    }

    /**
     * initialize the contract's initial state variables.
     */
    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(initial_supply: u64, owner: AztecAddress, outgoing_viewer: AztecAddress) {
        let balances = storage.balances;

        balances.at(owner).add(initial_supply, owner, outgoing_viewer);
    }

    // Mints `amount` of tokens to `owner`.
    #[aztec(private)]
    fn mint(amount: u64, owner: AztecAddress, outgoing_viewer: AztecAddress) {
        let balances = storage.balances;

        balances.at(owner).add(amount, owner, outgoing_viewer);
    }

    // Transfers `amount` of tokens from `sender` to a `recipient`.
    #[aztec(private)]
    fn transfer(
        amount: u64,
        sender: AztecAddress,
        recipient: AztecAddress,
        outgoing_viewer: AztecAddress
    ) {
        let balances = storage.balances;

        balances.at(sender).sub(amount, sender, outgoing_viewer);
        balances.at(recipient).add(amount, recipient, outgoing_viewer);
    }

    // Helper function to get the balance of a user ("unconstrained" is a Noir alternative of Solidity's "view" function).
    unconstrained fn get_balance(owner: AztecAddress) -> pub Field {
        let balances = storage.balances;

        // Return the sum of all notes in the set.
        balance_utils::get_balance(balances.at(owner).set)
    }
}
// docs:end:easy_private_token_contract
