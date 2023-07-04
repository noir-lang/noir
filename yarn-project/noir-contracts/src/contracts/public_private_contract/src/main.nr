mod balance_utils;
mod notes;
mod storage;

// A contract used to test insertion of a note from public domain into the private data tree. How this works is that
// a user provides a secret hash as an argument on the input of the public function, and the contract then inserts
// a note into the private data tree once the public call is processed. A note can later on be spent by providing
// the secret hash preimage on the input of a private function call.
//
// This flow will be typically used when performing a token swap:
// 1. A calls a private function which enqueues a public call to the DEX contract (with the secret hash on input).
// 2. The sequencer performs the public call to the DEX and the note gets inserted into the private data tree.
// 3. At this point a recipient can spend the note in a private function assuming he/she knows the secret.
contract PublicPrivate {
    use dep::std::hash::pedersen;
    use dep::aztec::abi;
    use dep::aztec::context::Context;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PublicContextInputs;
    use dep::aztec::log::emit_encrypted_log;

    // Private state
    use dep::aztec::types::point::Point;

    // oracles
    use dep::aztec::oracle::create_commitment::create_commitment;
    use dep::aztec::oracle::create_l2_to_l1_message::create_l2_to_l1_message;
    use dep::aztec::oracle::create_nullifier::create_nullifier;
    use dep::aztec::oracle::get_commitment::get_commitment;

    use crate::balance_utils;
    use crate::notes::transparent_note::TransparentNote;
    use crate::notes::value_note::ValueNote;
    use crate::storage::Storage;

    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        Context::new(inputs, 0).finish()
    }

    // A public function which inserts a note to the private data tree.
    open fn mintFromPublicToPrivate(
        _inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
    ) {
        // Create a commitment to the amount
        let note = TransparentNote::new(amount, secretHash);

        // Public oracle call to emit new commitment.
        create_commitment(note.get_commitment());
    }

    // Purely exists for testing
    open fn createL2ToL1MessagePublic(
        _inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
    ) {
        // Create a commitment to the amount
        let note = TransparentNote::new(amount, secretHash);
        
        // Public oracle call to emit new commitment.
        create_l2_to_l1_message(note.get_commitment());
    }

    // Purely exists for testing
    open fn createNullifierPublic(
        _inputs: PublicContextInputs,
        amount: Field,
        secretHash: Field,
    ) -> Field {
    
        // Create a commitment to the amount
        let note = TransparentNote::new(amount, secretHash);
        
        // Public oracle call to emit new commitment.
        create_nullifier(note.get_commitment());
        0
    }
    
    // Consumes the transparent note and creates a standard note.
    fn mintFromPublicMessage(
        inputs: PrivateContextInputs,
        amount: Field,
        secret: Field,
        owner: Point,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let storage = Storage::init();
        let mut context = Context::new(inputs, abi::hash_args([
            amount,
            secret,
            owner.x,
            owner.y,
        ]));

        // Assert that the note exists within the tree
        let public_note = TransparentNote::new_from_secret(amount, secret);
        context = public_note.consume_in_secret(context, inputs.roots.private_data_tree_root, secret);

        // Mint the tokens
        let owner_balance = storage.balances.at(owner.x);
        let note = ValueNote::new(amount, owner);

        // Insert note and emit encrypted preimage
        context = owner_balance.insert(context, note);
        let preimage = note.serialise();
        context = emit_encrypted_log(context, inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, preimage);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
    }

    /// ABI getBalance type "unconstrained"
    fn getBalance(
        owner: Point,
    ) -> pub Field {
        let storage = Storage::init();
        let owner_balance = storage.balances.at(owner.x);
        balance_utils::get_balance(owner_balance.storage_slot)
    }
}
