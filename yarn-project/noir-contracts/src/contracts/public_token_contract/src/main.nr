contract PublicToken {
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::Outputs;
    use dep::aztec3::abi::FinalOutputs;
    use dep::aztec3::abi::CallContext;
    use dep::aztec3::types::point::Point;
    use dep::aztec3::state_vars::storage_map::StorageMap;
    use dep::aztec3::state_vars::storage_field::StorageField;

    global balances = StorageMap { storage_slot: 1 };

    /// ABI constructor []
    fn constructor(
        inputs: pub Inputs,
    ) -> pub FinalOutputs {
        let outputs = Outputs::new();
        outputs.finish(inputs)
    }

    /// ABI mint [{"name":"amount","type":{"kind":"field"},"visibility":"public"},{"name":"recipient","type":{"kind":"struct","fields":[{"name":"x","type":{"kind":"field"}},{"name":"y","type":{"kind":"field"}}],"visibility":"public"}}]
    fn mint(
        _call_context: pub CallContext,
        amount: pub Field,
        recipient: pub Point,
    ) -> pub Field {
        let recipient_balance = balances.at(recipient.x);
        recipient_balance.write(recipient_balance.read() + amount)
    }


    /// ABI transfer [{"name":"amount","type":{"kind":"field"},"visibility":"public"},{"name":"recipient","type":{"kind":"struct","fields":[{"name":"x","type":{"kind":"field"}},{"name":"y","type":{"kind":"field"}}],"visibility":"public"}}]
    fn transfer(
        call_context: pub CallContext,
        amount: pub Field,
        recipient: pub Point,
    ) -> pub Field {
        let sender = call_context.msg_sender;
        
        let sender_balance = balances.at(sender);
        let recipient_balance = balances.at(recipient.x);
        let current_recipient_balance = recipient_balance.read();

        // TODO: Handle larger integer values ("long integers are not yet supported")
        if (sender_balance.read() as u120) > (amount as u120) {
            // TODO: Compiler complains if we don't assign the result of the write to anything
            let _void1 = sender_balance.write(sender_balance.read() - amount);
            recipient_balance.write(current_recipient_balance + amount)
        } else {
            // TODO: Revert if there is not enough balance
            current_recipient_balance
        }
    }

}
