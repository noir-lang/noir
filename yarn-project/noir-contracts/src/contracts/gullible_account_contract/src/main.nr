contract GullibleAccount {
    use dep::std;
    use dep::aztec3::entrypoint;
    use dep::aztec3::entrypoint::EntrypointPayload;
    use dep::aztec3::abi;
    use dep::aztec3::abi::PrivateContextInputs;
    use dep::aztec3::abi::CallContext;
    use dep::aztec3::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec3::public_call_stack_item::PublicCallStackItem;
    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::types::vec::BoundedVec;

    fn entrypoint(
        inputs: pub PrivateContextInputs,
        payload: pub EntrypointPayload,
        _signature: pub [u8;64]
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut context = PrivateFunctionContext::new();

        for i in 0..entrypoint::ACCOUNT_MAX_PRIVATE_CALLS {
            let target_address = payload.flattened_targets[i];
            if target_address != 0 {
                let function_selector = payload.flattened_selectors[i];
                let mut args = [0; abi::MAX_ARGS];
                for j in 0..abi::MAX_ARGS {
                    args[j] = payload.flattened_args[i * abi::MAX_ARGS + j];
                }
                let (_callStackItem, updated_context) = PrivateCallStackItem::call(target_address, function_selector, args, context);
                context = updated_context;
            }
        }

        for i in entrypoint::ACCOUNT_MAX_PRIVATE_CALLS..entrypoint::ACCOUNT_MAX_CALLS {
            let target_address = payload.flattened_targets[i];
            if target_address != 0 {
                let function_selector = payload.flattened_selectors[i];
                let mut args = [0; abi::MAX_ARGS];
                for j in 0..abi::MAX_ARGS {
                    args[j] = payload.flattened_args[i * abi::MAX_ARGS + j];
                }
                let (_callStackItem, updated_context) = PublicCallStackItem::call(target_address, function_selector, args, context);
                context = updated_context;
            }
        }

        // // Create args_hash manually since we exceed the MAX_ARGS in PrivateFunctionContext.
        // // Alternatively, we could make PrivateFunctionContext generic on the args size.
        // // ENTRYPOINT_PAYLOAD_SIZE + size_of(signature) = 21 + 64 = 85
        // let mut args: BoundedVec<Field, 85> = BoundedVec::new(0);
        // args = args.push_array(payload.flattened_args);
        // args = args.push_array(payload.flattened_selectors);
        // args = args.push(payload.nonce);
        // for byte in signature {
        //     args = args.push(byte as Field);
        // }
        // TODO(#499): Use hash index
        // let args_hash = dep::std::hash::pedersen(args.storage)[0];
        
        // TODO(#754): Pedersen is throwing a panic in wasm when computing the hash for too many args.
        let args_hash = 0;

        // Finish returns the already serialized payload, so we just tweak the corresponding field
        // Horrible, yes. But we need to rethink the private function context api anyway.
        // TODO(#745): Do not store args, set hash directly in context API
        let mut ret = context.finish(inputs);
        ret[dep::aztec3::abi::CALL_CONTEXT_SIZE] = args_hash;
        ret
    }

    fn constructor(
        inputs: pub PrivateContextInputs,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        PrivateFunctionContext::new().finish(inputs)
    }
}
