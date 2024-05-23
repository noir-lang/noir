mod lib;

contract GasToken {
    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::{AztecAddress, EthAddress}};
    use dep::aztec::state_vars::{SharedImmutable, PublicMutable, Map};

    use crate::lib::{calculate_fee, get_bridge_gas_msg_hash};

    #[aztec(storage)]
    struct Storage {
        // This map is accessed directly by protocol circuits to check balances for fee payment.
        // Do not change this storage layout unless you also update the base rollup circuits.
        balances: Map<AztecAddress, PublicMutable<U128>>,
        portal_address: SharedImmutable<EthAddress>,
    }

    // We purposefully not set this function as an initializer so we do not bind
    // the contract to a specific L1 portal address, since the gas token address
    // is a hardcoded constant in the rollup circuits.
    #[aztec(public)]
    fn set_portal(portal_address: EthAddress) {
        assert(storage.portal_address.read_public().is_zero());
        storage.portal_address.initialize(portal_address);
    }

    #[aztec(private)]
    fn claim(to: AztecAddress, amount: Field, secret: Field) {
        let content_hash = get_bridge_gas_msg_hash(to, amount);
        let portal_address = storage.portal_address.read_private();
        assert(!portal_address.is_zero());

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content_hash, secret, portal_address);

        // TODO(palla/gas) Emit an unencrypted log to announce which L1 to L2 message has been claimed
        // Otherwise, we cannot trace L1 deposits to their corresponding claims on L2

        GasToken::at(context.this_address())._increase_public_balance(to, amount).enqueue(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _increase_public_balance(to: AztecAddress, amount: Field) {
        let new_balance = storage.balances.at(to).read().add(U128::from_integer(amount));
        storage.balances.at(to).write(new_balance);
    }

    // TODO(palla/gas) Remove this function and use the private claim flow only
    #[aztec(public)]
    fn claim_public(to: AztecAddress, amount: Field, secret: Field, leaf_index: Field) {
        let content_hash = get_bridge_gas_msg_hash(to, amount);
        let portal_address = storage.portal_address.read_public();
        assert(!portal_address.is_zero());

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content_hash, secret, portal_address, leaf_index);

        let new_balance = storage.balances.at(to).read() + U128::from_integer(amount);
        storage.balances.at(to).write(new_balance);
    }

    // TODO(@just-mitch): remove this function before mainnet deployment
    // convenience function for testing
    // the true canonical gas token contract will not have this function
    #[aztec(public)]
    fn mint_public(to: AztecAddress, amount: Field) {
        let amount = U128::from_integer(amount);
        let new_balance = storage.balances.at(to).read().add(amount);

        storage.balances.at(to).write(new_balance);
    }

    #[aztec(public)]
    fn check_balance(fee_limit: Field) {
        let fee_limit = U128::from_integer(fee_limit);
        assert(storage.balances.at(context.msg_sender()).read() >= fee_limit, "Balance too low");
    }

    // utility function for testing
    unconstrained fn balance_of_public(owner: AztecAddress) -> pub Field {
        storage.balances.at(owner).read().to_field()
    }
}
