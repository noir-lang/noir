mod lib;

contract GasToken {
    use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::{AztecAddress, EthAddress}};
    use dep::aztec::{hash::{compute_secret_hash}, state_vars::{PublicMutable, Map}};

    use crate::lib::{calculate_fee, get_bridge_gas_msg_hash};

    struct Storage {
        balances: Map<AztecAddress, PublicMutable<U128>>,
    }

    #[aztec(private)]
    fn constructor() {}

    #[aztec(public)]
    fn claim_public(to: AztecAddress, amount: Field, canceller: EthAddress, msg_key: Field, secret: Field) {
        let content_hash = get_bridge_gas_msg_hash(to, amount, canceller);

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(msg_key, content_hash, secret, context.this_portal_address());

        let new_balance = storage.balances.at(to).read() + U128::from_integer(amount);
        storage.balances.at(to).write(new_balance);
    }

    #[aztec(public)]
    fn check_balance(fee_limit: Field) {
        let fee_limit_u120 = U128::from_integer(fee_limit);
        assert(storage.balances.at(context.msg_sender()).read() >= fee_limit_u120, "Balance too low");
    }

    #[aztec(public)]
    fn pay_fee(fee_limit: Field) -> Field {
        let fee_limit_u128 = U128::from_integer(fee_limit);
        let fee = calculate_fee(context);
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
