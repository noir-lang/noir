// Since we support account abstraction user accounts are represented by a contract. This contract is one such example
// where the user is not authenticated. This implies that anyone can call this contract and execute any function on its
// behalf.
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

    // A function which has to be implemented by every account abstraction contract.
    fn entrypoint(
        inputs: PrivateContextInputs,
        payload: EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        signature: [Field;3]
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // 10 = ENTRYPOINT_PAYLOAD_SIZE + 3
        let mut args: BoundedVec<Field, 10> = BoundedVec::new(0);
        args = args.push_array(payload.serialize());
        args = args.push_array(signature);
        
        let mut context = PrivateFunctionContext::new(inputs, abi::hash_args(args.storage));

        // Call private functions
        for i in 0..entrypoint::ACCOUNT_MAX_PRIVATE_CALLS {
            let target_address = payload.flattened_targets[i];
            if target_address != 0 {
                let function_selector = payload.flattened_selectors[i];
                let args_hash = payload.flattened_args_hashes[i];
                let (_callStackItem, updated_context) = PrivateCallStackItem::call_with_packed_args(target_address, function_selector, args_hash, context);
                context = updated_context;
            }
        }

        // Call public functions
        for i in entrypoint::ACCOUNT_MAX_PRIVATE_CALLS..entrypoint::ACCOUNT_MAX_CALLS {
            let target_address = payload.flattened_targets[i];
            if target_address != 0 {
                let function_selector = payload.flattened_selectors[i];
                let args_hash = payload.flattened_args_hashes[i];
                let (_callStackItem, updated_context) = PublicCallStackItem::call_with_packed_args(target_address, function_selector, args_hash, context);
                context = updated_context;
            }
        }

        context.finish()
    }

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        PrivateFunctionContext::new(inputs, 0).finish()
    }
}
