mod subscription_note;
mod dapp_payload;

contract AppSubscription {
    use dep::std;
    use crate::dapp_payload::DAppPayload;

    use dep::aztec::prelude::{
        AztecAddress, FunctionSelector, PrivateContext, NoteHeader, Map, PrivateMutable, PublicMutable,
        SharedImmutable
    };

    use dep::aztec::protocol_types::traits::is_empty;

    use dep::aztec::{context::Context, oracle::get_public_key::get_public_key};
    use dep::authwit::{account::AccountActions, auth_witness::get_auth_witness, auth::assert_current_call_valid_authwit};

    use crate::subscription_note::{SubscriptionNote, SUBSCRIPTION_NOTE_LEN};

    use dep::gas_token::GasToken;
    use dep::token::Token;

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
    fn entrypoint(payload: pub DAppPayload, user_address: pub AztecAddress) {
        assert(context.msg_sender().to_field() == 0);
        assert_current_call_valid_authwit(&mut context, user_address);

        let mut note = storage.subscriptions.at(user_address).get_note(false);
        assert(note.remaining_txs as u64 > 0, "you're out of txs");

        note.remaining_txs -= 1;
        storage.subscriptions.at(user_address).replace(&mut note, true);

        let gas_limit = storage.gas_token_limit_per_tx.read_private();
        GasToken::at(storage.gas_token_address.read_private()).pay_fee(gas_limit).enqueue(&mut context);

        context.end_setup();

        AppSubscription::at(context.this_address()).assert_not_expired(note.expiry_block_number).static_enqueue(&mut context);

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
    fn assert_not_expired(expiry_block_number: Field) {
        assert((context.block_number()) as u64 < expiry_block_number as u64);
    }

    #[aztec(public)]
    #[aztec(internal)]
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

        Token::at(storage.subscription_token_address.read_private()).transfer(
            context.msg_sender(),
            storage.subscription_recipient_address.read_private(),
            storage.subscription_price.read_private(),
            nonce
        ).call(&mut context);

        // Assert that the given expiry_block_number < current_block_number + SUBSCRIPTION_DURATION_IN_BLOCKS.
        AppSubscription::at(context.this_address()).assert_block_number(expiry_block_number).static_enqueue(&mut context);

        let mut subscription_note = SubscriptionNote::new(subscriber_address, expiry_block_number, tx_count);
        if (!is_initialized(subscriber_address)) {
            storage.subscriptions.at(subscriber_address).initialize(&mut subscription_note, true);
        } else {
            storage.subscriptions.at(subscriber_address).replace(&mut subscription_note, true)
        }
    }

    // Compiler bug workaround. You can't call an unconstrained function in another module, unless its from an
    // unconstained function in your module.
    unconstrained fn is_initialized(subscriber_address: AztecAddress) -> pub bool {
        storage.subscriptions.at(subscriber_address).is_initialized()
    }
}
