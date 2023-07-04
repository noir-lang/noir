// Since we support account abstraction user accounts are represented by a contract. This contract is one such example
// where the user is authenticated with a Schnorr signature.
contract SchnorrAccount {
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
    use dep::aztec3::types::point::Point;
    use dep::aztec3::oracle::debug_log::debug_log_format;

    // A function which has to be implemented by every account abstraction contract.
    fn entrypoint(
        inputs: pub PrivateContextInputs,
        payload: pub EntrypointPayload, // contains a set of arguments, selectors, targets and a nonce
        owner: pub [u8;64], // x and y coordinates concatenated
        signature: pub [Field;3], // schnorr signature stored in 3 fields
        partialContractAddress: pub Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // 75 = ENTRYPOINT_PAYLOAD_SIZE + 64 + 3 + 1
        let mut args: BoundedVec<Field, 75> = BoundedVec::new(0);
        args = args.push_array(payload.serialize());
        for byte in owner {
            args = args.push(byte as Field);
        }
        args = args.push_array(signature);
        args = args.push(partialContractAddress);
        
        let mut context = PrivateFunctionContext::new(inputs, abi::hash_args(args.storage));

        // Decompress signature from 3 fields to 64 bytes
        let mut decompressed_signature: [u8;64] = [0;64];

        let signature_part_0 = signature[0].to_be_bytes(32);
        for i in 0..31 {
            decompressed_signature[i] = signature_part_0[i + 1];
        }
        let signature_part_1 = signature[1].to_be_bytes(32);
        for i in 0..31 {
            decompressed_signature[i + 31] = signature_part_1[i + 1];
        }
        let signature_part_2 = signature[2].to_be_bytes(32);
        for i in 0..2 {
            decompressed_signature[i + 62] = signature_part_2[i + 1];
        }

        // Verify payload signature
        let payload_bytes: [u8; entrypoint::ENTRYPOINT_PAYLOAD_SIZE_IN_BYTES] = payload.to_be_bytes();
        let payload_hash: [u8; 32] = std::hash::sha256(payload_bytes);

        let mut x: Field = 0;
        let mut y: Field = 0;
        let mut mul: Field = 1;
        for i in 0..32 {
          let bytex: Field = owner[31 - i] as Field;
          x = x + (bytex * mul);
          let bytey: Field = owner[63 - i] as Field;
          y = y + (bytey * mul);
          mul *= 256;
        }
        
        
        // TODO: Find out why this signature verification never fails
        let verification = std::schnorr::verify_signature(x, y, decompressed_signature, payload_hash);
        assert(verification == true);

        // TODO: When we can use pedersen with non-zero generators, get this working        
        //let reproduced_address = dep::std::hash::pedersen([owner, partialContractAddress])[0];
        //constrain reproduced_address == inputs.call_context.storage_contract_address;

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

    // Constructs the contract
    fn constructor(
        inputs: pub PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        PrivateFunctionContext::new(inputs, 0).finish()
    }
}
