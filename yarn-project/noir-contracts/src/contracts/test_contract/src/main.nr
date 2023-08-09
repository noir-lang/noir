// A contract used to test whether constructing a contract works.
contract Test {
    use dep::aztec::{
        abi,
        abi::{
            PublicContextInputs,
            PrivateContextInputs
        }
    };
    use dep::aztec::context::Context;

    use dep::aztec::oracle::{
        create_l2_to_l1_message::create_l2_to_l1_message,
        create_nullifier::create_nullifier,
        get_public_key::get_public_key,
        context::get_portal_address
    };

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        Context::new(inputs, 0).finish()
    }

    fn getPublicKey(
        inputs: PrivateContextInputs,
        address: Field,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([address]));
        let pub_key = get_public_key(address);
        context.return_values.push_array([pub_key.x, pub_key.y]);
        context.finish()
    }

    // Get the portal contract address through an oracle call
    fn getPortalContractAddress(
        inputs: PrivateContextInputs,
        aztec_address: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([]));
        let portal_address = get_portal_address(aztec_address);
        context.return_values.push_array([portal_address]);
        context.finish()
    }

    // Get the address of the l1 portal for this contract (taken from the input context)
    fn getThisPortalAddress(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([]));
        let this_portal_address = context.this_portal_address();
        context.return_values.push_array([this_portal_address]);
        context.finish()
    }

    // Get the address of this contract (taken from the input context)
    fn getThisAddress(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([]));
        let this_address = context.this_address();
        context.return_values.push_array([this_address]);
        context.finish()
    }

    // Purely exists for testing
    open fn createL2ToL1MessagePublic(
        _inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
    ) {
        // Create a commitment to the amount
        let note = DummyNote::new(amount, secretHash);
        
        // Public oracle call to emit new commitment.
        create_l2_to_l1_message(note.get_commitment());
    }

    // Purely exists for testing
    open fn createNullifierPublic(
        _inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
    ) -> pub Field {
    
        // Create a commitment to the amount
        let note = DummyNote::new(amount, secretHash);
        
        // Public oracle call to emit new commitment.
        create_nullifier(note.get_commitment());
        0
    }

    struct DummyNote {
        amount: Field,
        secretHash: Field
    }

    impl DummyNote {
        fn new(amount: Field, secretHash: Field) -> pub Self {
            Self {
                amount: amount,
                secretHash: secretHash
            }
        }

        fn get_commitment(self) -> pub Field {
            dep::std::hash::pedersen([self.amount, self.secretHash])[0]
        }
    }
}
