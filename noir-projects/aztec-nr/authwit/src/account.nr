use dep::aztec::context::{PrivateContext, PublicContext, Context};
use dep::aztec::state_vars::{Map, PublicMutable};
use dep::aztec::protocol_types::{address::AztecAddress, abis::function_selector::FunctionSelector, hash::{pedersen_hash}};

use crate::entrypoint::{app::AppPayload, fee::FeePayload};
use crate::auth::{IS_VALID_SELECTOR, compute_outer_authwit_hash};

struct AccountActions {
  context: Context,
  is_valid_impl: fn(&mut PrivateContext, Field) -> bool,
  approved_action: Map<Field, PublicMutable<bool>>,
}

impl AccountActions {
    pub fn init(
        context: Context,
        approved_action_storage_slot: Field,
        is_valid_impl: fn(&mut PrivateContext, Field) -> bool
    ) -> Self {
        AccountActions {
            context,
            is_valid_impl,
            approved_action: Map::new(
                context,
                approved_action_storage_slot,
                |context, slot| {
          PublicMutable::new(context, slot)
        }
            )
        }
    }

    pub fn private(
        context: &mut PrivateContext,
        approved_action_storage_slot: Field,
        is_valid_impl: fn(&mut PrivateContext, Field) -> bool
    ) -> Self {
        AccountActions::init(
            Context::private(context),
            approved_action_storage_slot,
            is_valid_impl
        )
    }

    pub fn public(
        context: &mut PublicContext,
        approved_action_storage_slot: Field,
        is_valid_impl: fn(&mut PrivateContext, Field) -> bool
    ) -> Self {
        AccountActions::init(
            Context::public(context),
            approved_action_storage_slot,
            is_valid_impl
        )
    }

    // docs:start:entrypoint
    pub fn entrypoint(self, app_payload: AppPayload, fee_payload: FeePayload) {
        let valid_fn = self.is_valid_impl;
        let mut private_context = self.context.private.unwrap();

        let fee_hash = fee_payload.hash();
        assert(valid_fn(private_context, fee_hash));
        fee_payload.execute_calls(private_context);
        private_context.capture_min_revertible_side_effect_counter();

        let app_hash = app_payload.hash();
        assert(valid_fn(private_context, app_hash));
        app_payload.execute_calls(private_context);
    }
    // docs:end:entrypoint

    // docs:start:spend_private_authwit
    pub fn spend_private_authwit(self, inner_hash: Field) -> Field {
        let context = self.context.private.unwrap();
        // The `inner_hash` is "siloed" with the `msg_sender` to ensure that only it can 
        // consume the message.
        // This ensures that contracts cannot consume messages that are not intended for them.
        let message_hash = compute_outer_authwit_hash(context.msg_sender(), context.chain_id(), context.version(), inner_hash);
        let valid_fn = self.is_valid_impl;
        assert(valid_fn(context, message_hash) == true, "Message not authorized by account");
        context.push_new_nullifier(message_hash, 0);
        IS_VALID_SELECTOR
    }
    // docs:end:spend_private_authwit

    // docs:start:spend_public_authwit
    pub fn spend_public_authwit(self, inner_hash: Field) -> Field {
        let context = self.context.public.unwrap();
        // The `inner_hash` is "siloed" with the `msg_sender` to ensure that only it can 
        // consume the message.
        // This ensures that contracts cannot consume messages that are not intended for them.
        let message_hash = compute_outer_authwit_hash(context.msg_sender(), context.chain_id(), context.version(), inner_hash);
        let is_valid = self.approved_action.at(message_hash).read();
        assert(is_valid == true, "Message not authorized by account");
        context.push_new_nullifier(message_hash, 0);
        IS_VALID_SELECTOR
    }
    // docs:end:spend_public_authwit

    // docs:start:approve_public_authwit
    pub fn approve_public_authwit(self, message_hash: Field) {
        self.approved_action.at(message_hash).write(true);
    }
    // docs:end:approve_public_authwit
}
