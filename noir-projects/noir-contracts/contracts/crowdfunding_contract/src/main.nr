// docs:start:empty-contract
contract Crowdfunding {
    // docs:end:empty-contract

    // docs:start:all-deps
    use dep::aztec::{
        protocol_types::address::AztecAddress,
        encrypted_logs::encrypted_note_emission::encode_and_encrypt_note,
        state_vars::{PrivateSet, PublicImmutable, SharedImmutable}
    };
    use dep::aztec::unencrypted_logs::unencrypted_event_emission::encode_event;
    use dep::value_note::value_note::ValueNote;
    use dep::token::Token;
    // docs:end:all-deps

    #[aztec(event)]
    struct WithdrawalProcessed {
        who: Field,
        amount: Field,
    }

    // docs:start:storage
    #[aztec(storage)]
    struct Storage {
        // Token used for donations (e.g. DAI)
        donation_token: SharedImmutable<AztecAddress>,
        // Crowdfunding campaign operator
        operator: SharedImmutable<AztecAddress>,
        // End of the crowdfunding campaign after which no more donations are accepted
        deadline: PublicImmutable<u64>,
        // Notes emitted to donors when they donate (can be used as proof to obtain rewards, eg in Claim contracts)
        donation_receipts: PrivateSet<ValueNote>,
    }
    // docs:end:storage

    // docs:start:init
    // docs:start:init-header
    // docs:start:init-header-error
    #[aztec(public)]
    #[aztec(initializer)]
    // this-will-error:init-header-error
    fn init(donation_token: AztecAddress, operator: AztecAddress, deadline: u64) {
        // docs:end:init-header
        // docs:end:init-header-error
        storage.donation_token.initialize(donation_token);
        storage.operator.initialize(operator);
        storage.deadline.initialize(deadline);
    }
    // docs:end:init

    // docs:start:deadline
    // docs:start:deadline-header
    #[aztec(public)]
    #[aztec(internal)]
    #[aztec(view)]
    fn _check_deadline() {
        // docs:end:deadline-header
        let deadline = storage.deadline.read();
        assert(context.timestamp() < deadline, "Deadline has passed");
    }
    // docs:end:deadline

    // docs:start:donate
    // docs:start:call-check-deadline
    #[aztec(private)]
    fn donate(amount: u64) {
        // 1) Check that the deadline has not passed
        Crowdfunding::at(context.this_address())._check_deadline().enqueue_view(&mut context);
        // docs:end:call-check-deadline

        // docs:start:do-transfer
        // 2) Transfer the donation tokens from donor to this contract
        let donor = context.msg_sender();
        Token::at(storage.donation_token.read_private()).transfer_from(donor, context.this_address(), amount as Field, 0).call(&mut context);
        // docs:end:do-transfer

        let header = context.get_header();
        // 3) Create a value note for the donor so that he can later on claim a rewards token in the Claim
        // contract by proving that the hash of this note exists in the note hash tree.
        let donor_npk_m_hash = header.get_npk_m_hash(&mut context, donor);
        let mut note = ValueNote::new(amount as Field, donor_npk_m_hash);
        storage.donation_receipts.insert(&mut note).emit(encode_and_encrypt_note(&mut context, donor, donor));
    }
    // docs:end:donate

    // docs:start:operator-withdrawals
    // Withdraws balance to the operator. Requires that msg_sender() is the operator.
    #[aztec(private)]
    fn withdraw(amount: u64) {
        // 1) Check that msg_sender() is the operator
        let operator_address = storage.operator.read_private();
        assert(context.msg_sender() == operator_address, "Not an operator");

        // 2) Transfer the donation tokens from this contract to the operator
        Token::at(storage.donation_token.read_private()).transfer(operator_address, amount as Field).call(&mut context);
        // 3) Emit an unencrypted event so that anyone can audit how much the operator has withdrawn
        Crowdfunding::at(context.this_address())._publish_donation_receipts(amount, operator_address).enqueue(&mut context);
    }
    // docs:end:operator-withdrawals

    #[aztec(public)]
    #[aztec(internal)]
    fn _publish_donation_receipts(amount: u64, to: AztecAddress) {
        WithdrawalProcessed { amount: amount as Field, who: to.to_field() }.emit(encode_event(&mut context));
    }
}
