use dep::aztec::context::{PrivateContext, PublicContext, Context};
use dep::aztec::state_vars::{map::Map, public_state::PublicState};
use dep::aztec::types::type_serialization::bool_serialization::{BoolSerializationMethods,BOOL_SERIALIZED_LEN};

use crate::entrypoint::EntrypointPayload;
use crate::auth::IS_VALID_SELECTOR;

struct AccountActions {
  context: Context,
  is_valid_impl: fn(&mut PrivateContext, Field) -> bool,
  approved_action: Map<Field, PublicState<bool, BOOL_SERIALIZED_LEN>>,
}

impl AccountActions {
  pub fn init(context: Context, approved_action_storage_slot: Field, is_valid_impl: fn(&mut PrivateContext, Field) -> bool) -> Self {
    AccountActions {
      context: context,
      is_valid_impl: is_valid_impl,
      approved_action: Map::new(
        context,
        approved_action_storage_slot,
        |context, slot| {
          PublicState::new(context, slot, BoolSerializationMethods)
        },
      ),
    }
  }

  pub fn private(context: &mut PrivateContext, approved_action_storage_slot: Field, is_valid_impl: fn(&mut PrivateContext, Field) -> bool) -> Self {
    AccountActions::init(Context::private(context), approved_action_storage_slot, is_valid_impl)
  }

  pub fn public(context: &mut PublicContext, approved_action_storage_slot: Field, is_valid_impl: fn(&mut PrivateContext, Field) -> bool) -> Self {
    AccountActions::init(Context::public(context), approved_action_storage_slot, is_valid_impl)
  }
  
  // docs:start:entrypoint
  pub fn entrypoint(self, payload: EntrypointPayload) {
    let message_hash = payload.hash();
    let valid_fn = self.is_valid_impl;
    let private_context = self.context.private.unwrap();
    assert(valid_fn(private_context, message_hash));
    payload.execute_calls(private_context);
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
    if (value){
        IS_VALID_SELECTOR
    } else {
        0
    }
  }

  pub fn internal_set_is_valid_storage(self, message_hash: Field, value: bool) {
    self.approved_action.at(message_hash).write(value);
  }
}
