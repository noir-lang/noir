mod storage;

// A contract used along with `Parent` contract to test nested calls.
contract Child {
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PublicContextInputs;
    use dep::aztec::context::{
        PrivateContext,
        PublicContext
    };
    use crate::storage::Storage;
    use dep::aztec::oracle::logs::emit_unencrypted_log;

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        PrivateContext::new(inputs, 0).finish()
    }

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values.
    fn value(
        inputs: PrivateContextInputs,
        input: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([input]));

        context.return_values.push(input + context.chain_id() + context.version());

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Returns base_value + 42.
    open fn pubGetValue(inputs: PublicContextInputs, base_value: Field) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([base_value]));
        
        let returnValue = base_value + context.chain_id() + context.version() + context.block_number() + context.timestamp();

        context.return_values.push(returnValue);
        context.finish()
    }

    // Sets `current_value` to `new_value`
    open fn pubSetValue(inputs: PublicContextInputs, new_value: Field) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([new_value]));

        let storage = Storage::init();
        storage.current_value.write(new_value);
        let _hash = emit_unencrypted_log(new_value);
        context.return_values.push(new_value);
        context.finish()
    }

    // Increments `current_value` by `new_value`
    open fn pubIncValue(inputs: PublicContextInputs, new_value: Field) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([new_value]));

        let storage = Storage::init();
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        let _hash = emit_unencrypted_log(new_value);
        context.return_values.push(new_value);
        context.finish()
    }
}
