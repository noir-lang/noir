mod lib;

contract GasToken {
    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::{AztecAddress, EthAddress}};
    use dep::aztec::state_vars::{SharedImmutable, PublicMutable, Map};

    use crate::lib::{calculate_fee, get_bridge_gas_msg_hash};

    #[aztec(storage)]
    struct Storage {
        balances: Map<AztecAddress, PublicMutable<U128>>,
        portal_address: SharedImmutable<EthAddress>,
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(portal_address: EthAddress) {
        storage.portal_address.initialize(portal_address);
    }

    #[aztec(public)]
    fn claim_public(to: AztecAddress, amount: Field, secret: Field, leaf_index: Field) {
        let content_hash = get_bridge_gas_msg_hash(to, amount);

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(
            content_hash,
            secret,
            storage.portal_address.read_public(),
            leaf_index
        );

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

    #[aztec(public)]
    fn pay_fee(fee_limit: Field) -> Field {
        let fee_limit_u128 = U128::from_integer(fee_limit);
        let fee = U128::from_integer(calculate_fee(context));
        dep::aztec::oracle::debug_log::debug_log_format(
            "Gas token: paying fee {0} (limit {1})",
            [fee.to_field(), fee_limit]
        );
        assert(fee <= fee_limit_u128, "Fee too high");

        let sender_new_balance = storage.balances.at(context.msg_sender()).read() - fee;
        storage.balances.at(context.msg_sender()).write(sender_new_balance);

        let recipient_new_balance = storage.balances.at(context.fee_recipient()).read() + fee;
        storage.balances.at(context.fee_recipient()).write(recipient_new_balance);

        let rebate = fee_limit_u128 - fee;
        rebate.to_field()
    }

    // utility function for testing
    unconstrained fn balance_of_public(owner: AztecAddress) -> pub Field {
        storage.balances.at(owner).read().to_field()
    }
}
