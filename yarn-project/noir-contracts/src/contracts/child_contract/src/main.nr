mod storage;

// A contract used along with `Parent` contract to test nested calls.
contract Child {
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PublicContextInputs;
    use dep::aztec::context::Context;
    use crate::storage::Storage;

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        Context::new(inputs, 0).finish()
    }

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values.
    fn value(
        inputs: PrivateContextInputs,
        input: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([input]));

        context.return_values.push(input + inputs.private_global_variables.chain_id + inputs.private_global_variables.version);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    // Returns base_value + 42.
    open fn pubValue(inputs: PublicContextInputs, base_value: Field) -> pub Field {
        base_value + inputs.public_global_variables.chain_id + inputs.public_global_variables.version + inputs.public_global_variables.block_number + inputs.public_global_variables.timestamp
    }

    // Increments `current_value` by `new_value` and returns `new_value` + 1.
    open fn pubStoreValue(_inputs: PublicContextInputs, new_value: Field) -> pub Field {
        let storage = Storage::init();
        let old_value = storage.current_value.read();
        // Compiler complains if we don't assign the result to anything
        let _void1 = storage.current_value.write(old_value + new_value);
        // Compiler fails with "we do not allow private ABI inputs to be returned as public outputs" if we try to
        // return new_value as-is, but then it also complains if we add `pub` to `new_value` in the args, so we
        // just assign it to another variable and tweak it so it's not the same value, and the compiler is happy.
        let ret_value = new_value + 1;
        ret_value
    }
}
