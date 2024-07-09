mod subscription_note;
mod dapp_payload;

contract AppSubscription {
    use crate::{dapp_payload::DAppPayload, subscription_note::{SubscriptionNote, SUBSCRIPTION_NOTE_LEN}};

    use aztec::{
        prelude::{
        AztecAddress, FunctionSelector, PrivateContext, NoteHeader, Map, PrivateMutable, PublicMutable,
        SharedImmutable
    },
        encrypted_logs::encrypted_note_emission::encode_and_encrypt_note
    };
    use authwit::{auth_witness::get_auth_witness, auth::assert_current_call_valid_authwit};
    use gas_token::GasToken;
    use token::Token;

    #[aztec(storage)]
    struct Storage {
        // The following is only needed in private but we use ShareImmutable here instead of PrivateImmutable because
        // the value can be publicly known and SharedImmutable provides us with a better devex here because we don't
        // have to bother with sharing the note between pixies of users.
        target_address: SharedImmutable<AztecAddress>,
        subscription_token_address: SharedImmutable<AztecAddress>,
        subscription_recipient_address: SharedImmutable<AztecAddress>,
        subscription_price: SharedImmutable<Field>,
        subscriptions: Map<AztecAddress, PrivateMutable<SubscriptionNote>>,
        gas_token_address: SharedImmutable<AztecAddress>,
        gas_token_limit_per_tx: SharedImmutable<Field>,
    }

    global SUBSCRIPTION_DURATION_IN_BLOCKS = 5;
    global SUBSCRIPTION_TXS = 5;

    #[aztec(private)]
    fn entrypoint(payload: DAppPayload, user_address: AztecAddress) {
        assert(context.msg_sender().to_field() == 0);
        assert_current_call_valid_authwit(&mut context, user_address);

        let mut note = storage.subscriptions.at(user_address).get_note().note;
        assert(note.remaining_txs as u64 > 0, "you're out of txs");

        note.remaining_txs -= 1;

        // We are emitting both the outgoing and the incoming logs to the subscriber here because passing a separate
        // outgoing_viewer arg to entrypoint function is impractical and the outgoing are not so valuable here.
        storage.subscriptions.at(user_address).replace(&mut note).emit(encode_and_encrypt_note(&mut context, user_address, user_address));

        context.set_as_fee_payer();

        // TODO(palla/gas) Assert gas_token_limit_per_tx is less than this tx gas_limit
        let _gas_limit = storage.gas_token_limit_per_tx.read_private();

        context.end_setup();

        AppSubscription::at(context.this_address()).assert_not_expired(note.expiry_block_number).enqueue_view(&mut context);

        payload.execute_calls(&mut context, storage.target_address.read_private());
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(
        target_address: AztecAddress,
        subscription_recipient_address: AztecAddress,
        subscription_token_address: AztecAddress,
        subscription_price: Field,
        gas_token_address: AztecAddress,
        gas_token_limit_per_tx: Field
    ) {
        storage.target_address.initialize(target_address);
        storage.subscription_token_address.initialize(subscription_token_address);
        storage.subscription_recipient_address.initialize(subscription_recipient_address);
        storage.subscription_price.initialize(subscription_price);
        storage.gas_token_address.initialize(gas_token_address);
        storage.gas_token_limit_per_tx.initialize(gas_token_limit_per_tx);
    }

    #[aztec(public)]
    #[aztec(internal)]
    #[aztec(view)]
    fn assert_not_expired(expiry_block_number: Field) {
        assert((context.block_number()) as u64 < expiry_block_number as u64);
    }

    #[aztec(public)]
    #[aztec(internal)]
    #[aztec(view)]
    fn assert_block_number(expiry_block_number: Field) {
        assert(
            (context.block_number() + SUBSCRIPTION_DURATION_IN_BLOCKS) as u64
            >= expiry_block_number as u64
        );
    }

    #[aztec(private)]
    fn subscribe(
        subscriber_address: AztecAddress,
        nonce: Field,
        expiry_block_number: Field,
        tx_count: Field
    ) {
        assert(tx_count as u64 <= SUBSCRIPTION_TXS as u64);

        Token::at(storage.subscription_token_address.read_private()).transfer_from(
            context.msg_sender(),
            storage.subscription_recipient_address.read_private(),
            storage.subscription_price.read_private(),
            nonce
        ).call(&mut context);

        // Assert that the given expiry_block_number < current_block_number + SUBSCRIPTION_DURATION_IN_BLOCKS.
        AppSubscription::at(context.this_address()).assert_block_number(expiry_block_number).enqueue_view(&mut context);
        let header = context.get_header();
        let subscriber_npk_m_hash = header.get_npk_m_hash(&mut context, subscriber_address);

        let mut subscription_note = SubscriptionNote::new(subscriber_npk_m_hash, expiry_block_number, tx_count);
        storage.subscriptions.at(subscriber_address).initialize_or_replace(&mut subscription_note).emit(encode_and_encrypt_note(&mut context, context.msg_sender(), subscriber_address));
    }

    unconstrained fn is_initialized(subscriber_address: AztecAddress) -> pub bool {
        storage.subscriptions.at(subscriber_address).is_initialized()
    }
}
