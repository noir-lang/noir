contract Claim {
    mod interfaces;

    use dep::aztec::{
        history::note_inclusion::prove_note_inclusion,
        protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress},
        state_vars::SharedImmutable
    };
    use dep::value_note::value_note::ValueNote;
    use interfaces::Token;

    struct Storage {
      // Address of a contract based on whose notes we distribute the rewards
      target_contract: SharedImmutable<AztecAddress>,
      // Token to be distributed as a reward when claiming
      reward_token: SharedImmutable<AztecAddress>,
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(target_contract: AztecAddress, reward_token: AztecAddress) {
        storage.target_contract.initialize(target_contract);
        storage.reward_token.initialize(reward_token);
    }

    #[aztec(private)]
    fn claim(proof_note: ValueNote, recipient: AztecAddress) {
        // 1) Check that the note corresponds to the target contract and belongs to the sender
        let target_address = storage.target_contract.read_private();
        assert(
            target_address == proof_note.header.contract_address, "Note does not correspond to the target contract"
        );
        assert_eq(proof_note.owner, context.msg_sender(), "Note does not belong to the sender");

        // 2) Prove that the note hash exists in the note hash tree
        prove_note_inclusion(proof_note, context);

        // 3) Compute and emit a nullifier which is unique to the note and this contract to ensure the reward can be
        // claimed only once with the given note.
        // Note: The nullifier is unique to the note and THIS contract because the protocol siloes all nullifiers with
        // the address of a contract it was emitted from.
        context.push_new_nullifier(proof_note.compute_nullifier(&mut context), 0);

        // 4) Finally we mint the reward token to the sender of the transaction
        let reward_token = Token::at(storage.reward_token.read_private());
        reward_token.mint_public(&mut context, recipient, proof_note.value);
    }
}
