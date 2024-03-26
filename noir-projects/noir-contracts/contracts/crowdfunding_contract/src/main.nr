contract Crowdfunding {
    mod interfaces;

    use dep::aztec::{
        log::emit_unencrypted_log_from_private,
        protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, traits::Serialize},
        state_vars::{PrivateSet, PublicImmutable, SharedImmutable}
    };
    use dep::value_note::value_note::ValueNote;
    use interfaces::Token;

    #[event]
    struct WithdrawalProcessed {
        who: AztecAddress,
        amount: u64,
    }

    impl Serialize<2> for WithdrawalProcessed {
        fn serialize(self: Self) -> [Field; 2] {
            [self.who.to_field(), self.amount as Field]
        }
    }

    struct Storage {
        // Token used for donations (e.g. DAI)
        donation_token: SharedImmutable<AztecAddress>,
        // Crowdfunding campaign operator
        operator: SharedImmutable<AztecAddress>,
        // End of the crowdfunding campaign after which no more donations are accepted
        deadline: PublicImmutable<u64>,
        // Notes emitted to donors when they donate (later on used to claim rewards in the Claim contract)
        claim_notes: PrivateSet<ValueNote>,
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(donation_token: AztecAddress, operator: AztecAddress, deadline: u64) {
        storage.donation_token.initialize(donation_token);
        storage.operator.initialize(operator);
        storage.deadline.initialize(deadline);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _check_deadline() {
        let deadline = storage.deadline.read();
        assert(context.timestamp() < deadline, "Deadline has passed");
    }

    #[aztec(private)]
    fn donate(amount: u64) {
        // 1) Check that the deadline has not passed
        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("_check_deadline()"),
            []
        );

        // 2) Transfer the donation tokens from donor to this contract
        let donation_token = Token::at(storage.donation_token.read_private());
        donation_token.transfer(
            &mut context,
            context.msg_sender(),
            context.this_address(),
            amount as Field,
            0
        );

        // 3) Create a value note for the donor so that he can later on claim a rewards token in the Claim
        // contract by proving that the hash of this note exists in the note hash tree.
        let mut note = ValueNote::new(amount as Field, context.msg_sender());
        storage.claim_notes.insert(&mut note, true);
    }

    // Withdraws balance to the operator. Requires that msg_sender() is the operator.
    #[aztec(private)]
    fn withdraw(amount: u64) {
        // 1) Check that msg_sender() is the operator
        let operator_address = storage.operator.read_private();
        assert(context.msg_sender() == operator_address, "Not an operator");

        // 2) Transfer the donation tokens from this contract to the operator
        let donation_token = Token::at(storage.donation_token.read_private());
        donation_token.transfer(
            &mut context,
            context.this_address(),
            operator_address,
            amount as Field,
            0
        );

        // 3) Emit an unencrypted event so that anyone can audit how much the operator has withdrawn
        let event = WithdrawalProcessed { amount, who: operator_address };
        emit_unencrypted_log_from_private(&mut context, event.serialize());
    }
}
