use dep::aztec::context::{PrivateContext, PublicContext, Context};
use dep::aztec::state_vars::{map::Map, public_state::PublicState};

use crate::entrypoint::{app::AppPayload, fee::FeePayload};
use crate::auth::IS_VALID_SELECTOR;

struct AccountActions {
  context: Context,
  is_valid_impl: fn(&mut PrivateContext, Field) -> bool,
  approved_action: Map<Field, PublicState<bool>>,
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
          PublicState::new(context, slot)
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

    pub fn is_valid(self, message_hash: Field) -> Field {
        let valid_fn = self.is_valid_impl;
        if (valid_fn(self.context.private.unwrap(), message_hash)) {
            IS_VALID_SELECTOR
        } else {
            0
        }
    }

    pub fn is_valid_public(self, message_hash: Field) -> Field {
        let value = self.approved_action.at(message_hash).read();
        if (value) { IS_VALID_SELECTOR } else { 0 }
    }

    pub fn internal_set_is_valid_storage(self, message_hash: Field, value: bool) {
        self.approved_action.at(message_hash).write(value);
    }
}
