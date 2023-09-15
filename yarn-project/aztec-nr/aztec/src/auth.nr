use crate::context::{PrivateContext, PublicContext};
use crate::oracle::compute_selector::compute_selector;

global IS_VALID_SELECTOR = 0xe86ab4ff;
global IS_VALID_PUBLIC_SELECTOR = 0xf3661153;

fn assert_valid_message_for(context: &mut PrivateContext, whom: Field, message_hash: Field) {
  let result = context.call_private_function(whom, IS_VALID_SELECTOR, [message_hash])[0];
  context.push_new_nullifier(message_hash, 0);
  assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}

fn assert_valid_public_message_for(context: &mut PublicContext, whom: Field, message_hash: Field) {
  let result = context.call_public_function(whom, IS_VALID_PUBLIC_SELECTOR, [message_hash])[0];
  context.push_new_nullifier(message_hash, 0);
  assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}