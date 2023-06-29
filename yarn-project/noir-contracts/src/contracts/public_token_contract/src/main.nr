// Noir implementation of a public token contract. It allows for minting, and transfer of tokens. This contract only
// works with public state and for this reason user balances are represented as a value in the public data tree.
contract PublicToken {
    use dep::aztec3::abi;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::abi::PublicContextInputs;
    use dep::aztec3::types::point::Point;
    use dep::aztec3::state_vars::storage_map::StorageMap;
    use dep::aztec3::state_vars::storage_field::StorageField;

    // oracles
    // Note: Emitting encrypted logs in public functions is not supported because public functions can't work with
    // secrets (making it impossible to encrypt logs).
    use dep::aztec3::oracle::logs::emit_unencrypted_log;

    // A map from `owner.x` (Field type) to a value (StorageField type) representing user balance.
    global balances = StorageMap { storage_slot: 1 };

    // Constructs the contract.
    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        PrivateFunctionContext::new().finish(inputs)
    }

    // Mints `amount` of tokens to a `recipient`.
    open fn mint(
        _inputs: PublicContextInputs,
        amount: Field,
        recipient: Point,
        _padding: [Field; abi::MAX_ARGS - 3]
    ) -> pub Field {
        let recipient_balance = balances.at(recipient.x);
        // emit unencrypted event (for testing purposes)
        assert(emit_unencrypted_log("Coins minted") == 0);
        // Increase recipient balance by `amount`
        recipient_balance.write(recipient_balance.read() + amount)
    }

    // Transfers `amount` of tokens from `msg_sender` to `recipient`.
    open fn transfer(
        inputs: PublicContextInputs,
        amount: Field,
        recipient: Point,
        _padding: [Field; abi::MAX_ARGS - 3]
    ) -> pub Field {
        let sender = inputs.call_context.msg_sender;

        let sender_balance = balances.at(sender);
        let recipient_balance = balances.at(recipient.x);

        let current_sender_balance = sender_balance.read();
        let current_recipient_balance = recipient_balance.read();

        // TODO: Handle larger integer values ("long integers are not yet supported")
        if (current_sender_balance as u120) > (amount as u120) {
            // Sender has enough balance to transfer `amount` of tokens so we update sender's and recipient's balances.
            // TODO: Compiler complains if we don't assign the result of the write to anything
            let _void1 = sender_balance.write(current_sender_balance - amount);
            assert(emit_unencrypted_log("Coins transferred") == 0);
            recipient_balance.write(current_recipient_balance + amount)
        } else {
            // TODO: Revert if there is not enough balance
            current_recipient_balance
        }
    }

}
